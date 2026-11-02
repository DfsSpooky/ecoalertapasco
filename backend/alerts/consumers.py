import logging
from urllib.parse import parse_qs, urlparse

from channels.db import database_sync_to_async
from channels.generic.websocket import AsyncJsonWebsocketConsumer
from django.conf import settings
from django.utils import timezone
from rest_framework.authtoken.models import Token

from .models import Alert, WastePoint
from .serializers import AlertSerializer, WastePointSerializer

logger = logging.getLogger(__name__)

LIVE_MAP_GROUP = "ecoalerta_live"


def _is_origin_allowed(headers: list[tuple[bytes, bytes]]) -> bool:
    """
    Valida la cabecera Origin en conexiones WebSocket de producción soportando
    arquitecturas con subdominios separados (ej. ecoalerta.gob.pe -> wss://api.ecoalerta.gob.pe).
    """
    if settings.DEBUG or getattr(settings, "CORS_ALLOW_ALL_ORIGINS", False):
        return True

    origin = None
    for k, v in headers:
        if k.lower() == b"origin":
            origin = v.decode("latin1").strip()
            break

    # Clientes nativos / healthchecks sin cabecera Origin
    if not origin:
        return True

    allowed_origins = set(getattr(settings, "CORS_ALLOWED_ORIGINS", [])) | set(
        getattr(settings, "CSRF_TRUSTED_ORIGINS", [])
    )
    if origin.rstrip("/") in {o.rstrip("/") for o in allowed_origins}:
        return True

    parsed = urlparse(origin)
    origin_host = (parsed.hostname or "").lower()
    for host_pattern in getattr(settings, "ALLOWED_HOSTS", []):
        hp = host_pattern.strip().lower()
        if hp == "*" or hp == origin_host:
            return True
        if hp.startswith(".") and (origin_host == hp[1:] or origin_host.endswith(hp)):
            return True

    return False


@database_sync_to_async
def resolve_token_user_id(raw_token: str) -> tuple[int | None, bool]:
    """
    Resuelve el ID de usuario autenticado a partir de un Token DRF.
    Retorna `(user_id, is_invalid_token)`.
    """
    cleaned = (raw_token or "").strip()
    if not cleaned:
        return None, False

    token_obj = Token.objects.select_related("user").filter(key=cleaned).first()
    if not token_obj or not token_obj.user.is_active:
        return None, True
    return token_obj.user_id, False


@database_sync_to_async
def build_map_snapshot(user_id: int | None) -> dict:
    """
    Construye el estado inicial completo del mapa personalizado para el usuario conectado,
    sin consultas N+1.
    """
    confirmed_ids: set[int] = set()
    if user_id is not None:
        confirmed_ids = set(
            Alert.confirmed_by.through.objects.filter(user_id=user_id).values_list(
                "alert_id", flat=True
            )
        )

    alerts_qs = Alert.objects.select_related("reported_by").order_by("-created_at")
    waste_qs = WastePoint.objects.all().order_by("id")

    alerts_data = AlertSerializer(
        alerts_qs,
        many=True,
        context={"confirmed_alert_ids": confirmed_ids},
    ).data
    waste_data = WastePointSerializer(waste_qs, many=True).data

    return {
        "type": "initial_snapshot",
        "alerts": alerts_data,
        "waste_points": waste_data,
        "server_time": timezone.now().isoformat(),
    }


class MapLiveConsumer(AsyncJsonWebsocketConsumer):
    """
    Canal WebSocket full-duplex de tiempo real para EcoAlerta Cerro de Pasco (`ws/map/`).
    Difunde instantáneamente nuevas denuncias, cambios de estado municipal, respaldos (+1)
    y niveles de llenado de contenedores a todos los ciudadanos y autoridades conectadas.
    """

    async def connect(self):
        if not _is_origin_allowed(self.scope.get("headers", [])):
            await self.close(code=4403)
            return

        query_bytes = self.scope.get("query_string", b"")
        params = parse_qs(query_bytes.decode("utf-8", errors="ignore"))
        raw_token = (params.get("token") or [""])[0]

        user_id, token_invalid = await resolve_token_user_id(raw_token)
        self.user_id: int | None = user_id
        self.last_snapshot_time: float = 0.0

        await self.channel_layer.group_add(LIVE_MAP_GROUP, self.channel_name)
        await self.accept()

        if token_invalid:
            await self.send_json({"type": "auth_invalid"})

        snapshot = await build_map_snapshot(self.user_id)
        await self.send_json(snapshot)

    async def disconnect(self, close_code):
        try:
            await self.channel_layer.group_discard(LIVE_MAP_GROUP, self.channel_name)
        except Exception:
            pass

    async def receive_json(self, content, **kwargs):
        if not isinstance(content, dict):
            return

        msg_type = content.get("type")
        if msg_type == "ping":
            await self.send_json(
                {
                    "type": "pong",
                    "server_time": timezone.now().isoformat(),
                }
            )
        elif msg_type == "auth":
            raw_token = str(content.get("token") or "")
            user_id, token_invalid = await resolve_token_user_id(raw_token)
            self.user_id = user_id
            if token_invalid:
                await self.send_json({"type": "auth_invalid"})
            snapshot = await build_map_snapshot(self.user_id)
            await self.send_json(snapshot)
        elif msg_type == "request_snapshot":
            import time
            now = time.monotonic()
            last_time = getattr(self, "last_snapshot_time", 0.0)
            if now - last_time < 3.0:
                await self.send_json(
                    {
                        "type": "throttled",
                        "message": "Por favor espera unos segundos antes de solicitar otro snapshot completo.",
                    }
                )
                return
            self.last_snapshot_time = now
            snapshot = await build_map_snapshot(self.user_id)
            await self.send_json(snapshot)

    async def map_alert_upserted(self, event):
        alert_payload = dict(event["alert"])
        confirmed_user_ids = event.get("confirmed_user_ids") or []
        alert_payload["has_user_confirmed"] = bool(
            self.user_id is not None and self.user_id in confirmed_user_ids
        )
        await self.send_json(
            {
                "type": "alert_upserted",
                "alert": alert_payload,
            }
        )

    async def map_alert_deleted(self, event):
        await self.send_json(
            {
                "type": "alert_deleted",
                "id": str(event["id"]),
            }
        )

    async def map_waste_upserted(self, event):
        await self.send_json(
            {
                "type": "waste_point_upserted",
                "waste_point": event["waste_point"],
            }
        )

    async def map_waste_deleted(self, event):
        await self.send_json(
            {
                "type": "waste_point_deleted",
                "id": str(event["id"]),
            }
        )
