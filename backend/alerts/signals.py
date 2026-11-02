import logging
from asgiref.sync import async_to_sync
from channels.layers import get_channel_layer
from django.db import connection, transaction
from django.db.models.signals import m2m_changed, post_delete, post_save
from django.dispatch import receiver

from .consumers import LIVE_MAP_GROUP
from .models import Alert, WastePoint
from .serializers import AlertSerializer, WastePointSerializer

logger = logging.getLogger(__name__)


def _send_group_event(payload: dict) -> None:
    try:
        channel_layer = get_channel_layer()
        if channel_layer is not None:
            async_to_sync(channel_layer.group_send)(LIVE_MAP_GROUP, payload)
    except Exception as exc:
        logger.warning("No se pudo difundir evento WebSocket en %s: %s", LIVE_MAP_GROUP, exc)


def broadcast_alert_by_id(alert_id: int) -> None:
    alert = Alert.objects.select_related("reported_by").filter(pk=alert_id).first()
    if not alert:
        return
    confirmed_user_ids = list(alert.confirmed_by.values_list("id", flat=True))
    serialized = AlertSerializer(alert, context={"confirmed_alert_ids": set()}).data
    _send_group_event(
        {
            "type": "map.alert.upserted",
            "alert": serialized,
            "confirmed_user_ids": confirmed_user_ids,
        }
    )


def _schedule_alert_broadcast(alert_id: int) -> None:
    pending = getattr(connection, "_eco_ws_alert_ids", None)
    if pending is None:
        pending = set()
        connection._eco_ws_alert_ids = pending
    if alert_id in pending:
        return
    pending.add(alert_id)

    def _flush():
        pending.discard(alert_id)
        broadcast_alert_by_id(alert_id)

    transaction.on_commit(_flush)


def broadcast_waste_point_by_id(wp_id: int) -> None:
    wp = WastePoint.objects.filter(pk=wp_id).first()
    if not wp:
        return
    serialized = WastePointSerializer(wp).data
    _send_group_event(
        {
            "type": "map.waste.upserted",
            "waste_point": serialized,
        }
    )


@receiver(post_save, sender=Alert)
def on_alert_saved(sender, instance: Alert, **kwargs):
    if instance.pk:
        _schedule_alert_broadcast(instance.pk)


@receiver(post_delete, sender=Alert)
def on_alert_deleted(sender, instance: Alert, **kwargs):
    alert_id = str(instance.pk)
    transaction.on_commit(
        lambda: _send_group_event(
            {
                "type": "map.alert.deleted",
                "id": alert_id,
            }
        )
    )


@receiver(m2m_changed, sender=Alert.confirmed_by.through)
def on_alert_confirmations_changed(sender, instance: Alert, action: str, **kwargs):
    if action in ("post_add", "post_remove", "post_clear") and instance.pk:
        _schedule_alert_broadcast(instance.pk)


@receiver(post_save, sender=WastePoint)
def on_waste_point_saved(sender, instance: WastePoint, **kwargs):
    wp_id = instance.pk
    transaction.on_commit(lambda: broadcast_waste_point_by_id(wp_id))


@receiver(post_delete, sender=WastePoint)
def on_waste_point_deleted(sender, instance: WastePoint, **kwargs):
    wp_id = str(instance.pk)
    transaction.on_commit(
        lambda: _send_group_event(
            {
                "type": "map.waste.deleted",
                "id": wp_id,
            }
        )
    )
