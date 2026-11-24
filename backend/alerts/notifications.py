import html
import json
import logging
import os
import sys
import threading

import httpx
from cryptography.hazmat.primitives import serialization
from django.db import close_old_connections
from django.db.models import Q
from django.utils import timezone
from py_vapid import Vapid
from py_vapid.utils import b64urlencode
from pywebpush import WebPushException, webpush

from .models import Alert, PushSubscription, SystemConfig, is_serenazgo

logger = logging.getLogger(__name__)

_VAPID_LOCK = threading.Lock()


def get_or_create_vapid_keys() -> tuple[str, str]:
    """
    Obtiene el par de llaves VAPID (privada, pública) desde SystemConfig o variables de entorno.
    Si aún no existen, las genera automáticamente (ECDSA P-256) y las persiste en SystemConfig.
    """
    priv = SystemConfig.get_value("vapid_private_key", os.environ.get("VAPID_PRIVATE_KEY", "")).strip()
    pub = SystemConfig.get_value("vapid_public_key", os.environ.get("VAPID_PUBLIC_KEY", "")).strip()
    if priv and pub:
        return priv, pub

    with _VAPID_LOCK:
        priv = SystemConfig.get_value("vapid_private_key", os.environ.get("VAPID_PRIVATE_KEY", "")).strip()
        pub = SystemConfig.get_value("vapid_public_key", os.environ.get("VAPID_PUBLIC_KEY", "")).strip()
        if priv and pub:
            return priv, pub

        vapid = Vapid()
        vapid.generate_keys()
        priv_bytes = vapid.private_key.private_numbers().private_value.to_bytes(32, "big")
        pub_bytes = vapid.public_key.public_bytes(
            serialization.Encoding.X962,
            serialization.PublicFormat.UncompressedPoint,
        )
        priv = b64urlencode(priv_bytes)
        pub = b64urlencode(pub_bytes)
        SystemConfig.set_value(
            "vapid_private_key",
            priv,
            "Llave privada VAPID (P-256) para Web Push a Serenazgo y autoridades",
        )
        SystemConfig.set_value(
            "vapid_public_key",
            pub,
            "Llave pública VAPID (base64url) para suscripciones Web Push",
        )
        return priv, pub


def get_vapid_claim_sub() -> str:
    raw = SystemConfig.get_value(
        "vapid_claim_email",
        os.environ.get("VAPID_CLAIM_EMAIL", "mailto:serenazgo@ecoalerta.pasco.gob.pe"),
    ).strip()
    if not raw:
        return "mailto:serenazgo@ecoalerta.pasco.gob.pe"
    if not raw.startswith(("mailto:", "https://", "http://")):
        return f"mailto:{raw}"
    return raw


def get_telegram_settings() -> dict:
    enabled_raw = SystemConfig.get_value(
        "telegram_enabled",
        os.environ.get("TELEGRAM_ENABLED", "false"),
    ).strip().lower()
    token = SystemConfig.get_value(
        "telegram_bot_token",
        os.environ.get("TELEGRAM_BOT_TOKEN", ""),
    ).strip()
    chat_id = SystemConfig.get_value(
        "telegram_chat_id",
        os.environ.get("TELEGRAM_CHAT_ID", ""),
    ).strip()
    public_url = SystemConfig.get_value(
        "public_app_url",
        os.environ.get("PUBLIC_APP_URL", os.environ.get("FRONTEND_URL", "")),
    ).strip().rstrip("/")
    return {
        "enabled": enabled_raw in ("true", "1", "yes", "si", "on"),
        "bot_token": token,
        "chat_id": chat_id,
        "public_app_url": public_url,
    }


def build_alert_push_payload(alert: Alert) -> dict:
    """Construye el payload Web Push sin exponer identidad ni evidencia privada."""
    is_ciudadana = alert.domain == "ciudadana"
    prefix = "🚨 Serenazgo Pasco" if is_ciudadana else "🌿 Alerta Crítica"
    sev_label = alert.get_severity_display()
    dist_label = alert.get_district_display()
    cat_label = alert.get_category_display()

    title = f"{prefix} · {cat_label} ({sev_label})"
    body = f"{alert.title}\n📍 {alert.address} ({dist_label})"
    vista = "ciudadana" if is_ciudadana else "ambiental"

    return {
        "type": "new_alert",
        "alert_id": str(alert.id),
        "domain": alert.domain,
        "severity": alert.severity,
        "category": alert.category,
        "district": alert.district,
        "title": title,
        "body": body,
        "address": alert.address,
        "url": f"/?vista={vista}&alert_id={alert.id}",
        "created_at": alert.created_at.isoformat() if alert.created_at else timezone.now().isoformat(),
    }


def should_notify_push_for_alert(alert: Alert, sub: PushSubscription) -> bool:
    if not sub.is_active or not sub.user or not sub.user.is_active:
        return False

    if sub.district_filter and sub.district_filter != alert.district:
        return False

    if alert.domain == "ciudadana":
        if not sub.notify_ciudadana:
            return False
        return bool(sub.user.is_superuser or is_serenazgo(sub.user))

    if not sub.notify_ambiental:
        return False
    if not (sub.user.is_superuser or sub.user.is_staff):
        return False
    return alert.severity == "critico"


def send_webpush_to_subscription(
    sub: PushSubscription,
    payload: dict,
    priv_key: str,
    claim_sub: str,
) -> bool:
    """Envía una notificación Web Push a una suscripción individual."""
    subscription_info = {
        "endpoint": sub.endpoint,
        "keys": {
            "p256dh": sub.p256dh,
            "auth": sub.auth,
        },
    }
    try:
        webpush(
            subscription_info=subscription_info,
            data=json.dumps(payload, ensure_ascii=False),
            vapid_private_key=priv_key,
            vapid_claims={"sub": claim_sub},
            ttl=3600,
            timeout=8.0,
        )
        PushSubscription.objects.filter(pk=sub.pk).update(last_notified_at=timezone.now())
        return True
    except WebPushException as exc:
        status_code = getattr(getattr(exc, "response", None), "status_code", None)
        if status_code in (404, 410):
            logger.info("Desactivando suscripción Web Push expirada (%s): %s", status_code, sub.pk)
            PushSubscription.objects.filter(pk=sub.pk).update(is_active=False)
        else:
            logger.warning("Error Web Push para suscripción %s (status=%s): %s", sub.pk, status_code, exc)
        return False
    except Exception as exc:
        logger.warning("Error inesperado enviando Web Push a %s: %s", sub.pk, exc)
        return False


def send_push_for_alert(alert_id: int) -> int:
    """Envía Web Push a todos los dispositivos autorizados (Serenazgo / Staff) para una denuncia."""
    alert = Alert.objects.filter(pk=alert_id).first()
    if not alert:
        return 0

    subs = list(
        PushSubscription.objects.select_related("user")
        .filter(is_active=True, user__is_active=True)
        .filter(
            Q(user__is_superuser=True)
            | Q(user__is_staff=True)
            | Q(user__groups__name="serenazgo")
        )
        .distinct()
    )
    if not subs:
        return 0

    eligible = [s for s in subs if should_notify_push_for_alert(alert, s)]
    if not eligible:
        return 0

    priv_key, _ = get_or_create_vapid_keys()
    claim_sub = get_vapid_claim_sub()
    payload = build_alert_push_payload(alert)

    sent_count = 0
    for sub in eligible:
        if send_webpush_to_subscription(sub, payload, priv_key, claim_sub):
            sent_count += 1
    return sent_count


def should_notify_telegram_for_alert(alert: Alert) -> bool:
    return alert.domain == "ciudadana" or alert.severity == "critico"


def format_telegram_alert_message(alert: Alert, public_app_url: str = "") -> str:
    is_ciudadana = alert.domain == "ciudadana"
    header = "🚨 <b>NUEVA DENUNCIA · SERENAZGO PASCO</b>" if is_ciudadana else "🌿 <b>ALERTA AMBIENTAL CRÍTICA · ECOALERTA</b>"
    sev_badge = {
        "critico": "🔴 CRÍTICO",
        "medio": "🟠 MEDIO",
        "bajo": "🟡 BAJO",
    }.get(alert.severity, alert.severity.upper())

    title = html.escape(alert.title or "")
    cat = html.escape(alert.get_category_display())
    dist = html.escape(alert.get_district_display())
    addr = html.escape(alert.address or "Sin dirección")
    desc_raw = (alert.description or "").strip()
    if len(desc_raw) > 240:
        desc_raw = desc_raw[:237] + "..."
    desc = html.escape(desc_raw)

    maps_url = f"https://maps.google.com/?q={alert.latitude:.6f},{alert.longitude:.6f}"
    lines = [
        header,
        "",
        f"<b>Incidente:</b> {title}",
        f"<b>Categoría:</b> {cat}  |  <b>Nivel:</b> {sev_badge}",
        f"<b>Distrito:</b> {dist}",
        f"<b>Dirección:</b> {addr}",
    ]
    if desc:
        lines.append(f"<b>Detalle:</b> {desc}")
    lines.append("")
    lines.append(f"📍 <a href=\"{maps_url}\">Abrir ubicación en Google Maps</a>")

    if public_app_url:
        vista = "ciudadana" if is_ciudadana else "ambiental"
        app_link = f"{public_app_url}/?vista={vista}&alert_id={alert.id}"
        lines.append(f"🛡️ <a href=\"{html.escape(app_link)}\">Ver denuncia en EcoAlerta</a>")

    return "\n".join(lines)


def send_telegram_for_alert(alert_id: int) -> bool:
    cfg = get_telegram_settings()
    if not cfg["enabled"] or not cfg["bot_token"] or not cfg["chat_id"]:
        return False

    alert = Alert.objects.filter(pk=alert_id).first()
    if not alert or not should_notify_telegram_for_alert(alert):
        return False

    text = format_telegram_alert_message(alert, cfg["public_app_url"])
    url = f"https://api.telegram.org/bot{cfg['bot_token']}/sendMessage"
    try:
        resp = httpx.post(
            url,
            json={
                "chat_id": cfg["chat_id"],
                "text": text,
                "parse_mode": "HTML",
                "disable_web_page_preview": True,
            },
            timeout=10.0,
        )
        if resp.status_code != 200:
            logger.warning("Telegram respondió %s: %s", resp.status_code, resp.text[:250])
            return False
        return True
    except Exception as exc:
        logger.warning("Error enviando alerta a Telegram: %s", exc)
        return False


def send_telegram_test_message(bot_token: str, chat_id: str) -> tuple[bool, str]:
    url = f"https://api.telegram.org/bot{bot_token}/sendMessage"
    try:
        resp = httpx.post(
            url,
            json={
                "chat_id": chat_id,
                "text": (
                    "🛡️ <b>Prueba de Conexión · EcoAlerta Serenazgo Pasco</b>\n\n"
                    "✅ El canal de notificaciones inmediatas está operativo. "
                    "Las denuncias de Seguridad Ciudadana y emergencias críticas llegarán aquí al instante."
                ),
                "parse_mode": "HTML",
                "disable_web_page_preview": True,
            },
            timeout=10.0,
        )
        if resp.status_code == 200:
            return True, "Mensaje de prueba enviado correctamente al grupo/chat de Telegram."
        detail = resp.text[:200]
        try:
            detail = resp.json().get("description", detail)
        except Exception:
            pass
        return False, f"Telegram rechazó el envío ({resp.status_code}): {detail}"
    except Exception as exc:
        return False, f"No se pudo conectar con Telegram API: {exc}"


def notify_authorities_on_new_alert(alert_id: int) -> None:
    """Despacha notificaciones Web Push y Telegram para una nueva denuncia."""
    try:
        send_push_for_alert(alert_id)
    except Exception as exc:
        logger.warning("Fallo en envío Web Push para alerta %s: %s", alert_id, exc)

    try:
        send_telegram_for_alert(alert_id)
    except Exception as exc:
        logger.warning("Fallo en envío Telegram para alerta %s: %s", alert_id, exc)


def schedule_new_alert_notifications(alert_id: int) -> None:
    """Programa el despacho de notificaciones en segundo plano fuera de la transacción."""
    if "test" in sys.argv:
        # Durante pruebas automatizadas evitamos hilos sueltos contra red externa;
        # los tests de notificaciones invocan explícitamente los helpers con mocks.
        return

    def _worker():
        close_old_connections()
        try:
            notify_authorities_on_new_alert(alert_id)
        finally:
            close_old_connections()

    threading.Thread(
        target=_worker,
        name=f"ecoalerta-notify-{alert_id}",
        daemon=True,
    ).start()
