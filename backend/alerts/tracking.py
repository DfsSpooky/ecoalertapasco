"""
Seguimiento en vivo de camiones recolectores (Fase 2).

Ingesta de posiciones GPS enviadas por el teléfono del conductor, filtrado de lecturas
erróneas, muestreo del historial y difusión en tiempo real al mapa ciudadano.
"""
import logging
import math
from datetime import datetime, timedelta, timezone as dt_timezone

from asgiref.sync import async_to_sync
from channels.layers import get_channel_layer
from django.db import transaction
from django.utils import timezone

from .consumers import LIVE_MAP_GROUP
from .models import RouteRun, TruckPosition

logger = logging.getLogger(__name__)

# Lecturas con precisión peor que esto (metros) se descartan: suelen ser triangulación por antenas
MAX_ACCURACY_M = 60.0
# Velocidad máxima plausible para un compactador en ciudad (salto de GPS si se supera)
MAX_SPEED_KMH = 110.0
# Intervalo mínimo entre posiciones aceptadas (segundos)
MIN_INTERVAL_S = 2.0
# Historial en BD: una muestra cada N segundos o cada N metros recorridos
PERSIST_INTERVAL_S = 15.0
PERSIST_MIN_DISTANCE_M = 40.0
# Lote máximo por envío (buffer offline del teléfono)
MAX_BATCH = 200
# Tolerancia de reloj del teléfono hacia el futuro
MAX_FUTURE_SKEW_S = 120.0
# Turnos abiertos sin señal más allá de esto no se muestran a la ciudadanía
OPEN_RUN_MAX_AGE = timedelta(hours=16)


def haversine_m(lat1, lon1, lat2, lon2) -> float:
    r = 6371000.0
    p1, p2 = math.radians(lat1), math.radians(lat2)
    dp, dl = math.radians(lat2 - lat1), math.radians(lon2 - lon1)
    a = math.sin(dp / 2) ** 2 + math.cos(p1) * math.cos(p2) * math.sin(dl / 2) ** 2
    return 2 * r * math.asin(math.sqrt(min(1.0, a)))


def _parse_ts(raw):
    """Acepta epoch en milisegundos (Date.now() del navegador) o ISO-8601."""
    if isinstance(raw, (int, float)) and not isinstance(raw, bool):
        return datetime.fromtimestamp(float(raw) / 1000.0, tz=dt_timezone.utc)
    if isinstance(raw, str) and raw.strip():
        try:
            parsed = datetime.fromisoformat(raw.strip().replace('Z', '+00:00'))
        except ValueError:
            return None
        if timezone.is_naive(parsed):
            parsed = timezone.make_aware(parsed, dt_timezone.utc)
        return parsed
    return None


def _opt_float(value):
    if value is None or isinstance(value, bool):
        return None
    try:
        f = float(value)
    except (TypeError, ValueError):
        return None
    return f if math.isfinite(f) else None


def _normalize_point(raw):
    """Convierte un punto crudo del cliente en dict validado, o None si es inválido."""
    if not isinstance(raw, dict):
        return None
    lat = _opt_float(raw.get('latitude', raw.get('lat')))
    lng = _opt_float(raw.get('longitude', raw.get('lng')))
    ts = _parse_ts(raw.get('ts', raw.get('recorded_at')))
    if lat is None or lng is None or ts is None:
        return None
    if not (-90.0 <= lat <= 90.0) or not (-180.0 <= lng <= 180.0):
        return None
    accuracy = _opt_float(raw.get('accuracy'))
    # El navegador entrega velocidad en m/s; se almacena en km/h
    speed_ms = _opt_float(raw.get('speed'))
    heading = _opt_float(raw.get('heading'))
    if heading is not None and not (0.0 <= heading <= 360.0):
        heading = None
    return {
        'latitude': lat,
        'longitude': lng,
        'ts': ts,
        'accuracy': accuracy,
        'speed_kmh': round(speed_ms * 3.6, 1) if speed_ms is not None and speed_ms >= 0 else None,
        'heading': heading,
    }


def public_run_payload(run: RouteRun) -> dict:
    """Datos de un turno visibles para la ciudadanía (sin conductor ni placa)."""
    return {
        'id': run.id,
        'route': run.route_id,
        'route_name': run.route.name,
        'route_color': run.route.color,
        'truck_code': run.truck.code,
        'waste_type': run.truck.waste_type,
        'status': run.status,
        'started_at': run.started_at.isoformat() if run.started_at else None,
        'last_latitude': run.last_latitude,
        'last_longitude': run.last_longitude,
        'last_heading': run.last_heading,
        'last_speed_kmh': run.last_speed_kmh,
        'last_position_at': run.last_position_at.isoformat() if run.last_position_at else None,
    }


def public_open_runs_queryset():
    cutoff = timezone.now() - OPEN_RUN_MAX_AGE
    return (
        RouteRun.objects.filter(status__in=['active', 'paused'], started_at__gte=cutoff)
        .select_related('route', 'truck')
        .order_by('truck__code')
    )


def _send(payload: dict) -> None:
    try:
        layer = get_channel_layer()
        if layer is not None:
            async_to_sync(layer.group_send)(LIVE_MAP_GROUP, payload)
    except Exception as exc:  # pragma: no cover - Redis caído no debe tumbar la API
        logger.warning("No se pudo difundir evento de flota: %s", exc)


def broadcast_run_state(run_id: int) -> None:
    """Difunde inicio/pausa/reanudación (upserted) o fin (ended) de un turno."""
    def _flush():
        run = RouteRun.objects.select_related('route', 'truck').filter(pk=run_id).first()
        if run is None or run.status == 'finished':
            _send({'type': 'map.truck.run.ended', 'id': str(run_id)})
        else:
            _send({'type': 'map.truck.run.upserted', 'run': public_run_payload(run)})
    transaction.on_commit(_flush)


def ingest_positions(run: RouteRun, raw_points) -> dict:
    """
    Procesa un lote de posiciones del conductor.
    Devuelve contadores para que el cliente sepa qué se aceptó.
    """
    if not isinstance(raw_points, list):
        raw_points = [raw_points]
    raw_points = raw_points[:MAX_BATCH]

    now = timezone.now()
    points = [p for p in (_normalize_point(r) for r in raw_points) if p is not None]
    invalid = len(raw_points) - len(points)
    points.sort(key=lambda p: p['ts'])

    last_lat, last_lng, last_ts = run.last_latitude, run.last_longitude, run.last_position_at
    persisted_last = run.positions.order_by('-recorded_at').values('latitude', 'longitude', 'recorded_at').first()

    accepted = []
    to_persist = []
    rejected = invalid
    for p in points:
        ts = p['ts']
        if ts < run.started_at - timedelta(seconds=30) or (ts - now).total_seconds() > MAX_FUTURE_SKEW_S:
            rejected += 1
            continue
        if p['accuracy'] is not None and p['accuracy'] > MAX_ACCURACY_M:
            rejected += 1
            continue
        if last_ts is not None:
            dt = (ts - last_ts).total_seconds()
            if dt < MIN_INTERVAL_S:
                rejected += 1
                continue
            dist = haversine_m(last_lat, last_lng, p['latitude'], p['longitude'])
            if (dist / dt) * 3.6 > MAX_SPEED_KMH:
                rejected += 1
                continue
        accepted.append(p)
        last_lat, last_lng, last_ts = p['latitude'], p['longitude'], ts

        if (
            persisted_last is None
            or (ts - persisted_last['recorded_at']).total_seconds() >= PERSIST_INTERVAL_S
            or haversine_m(persisted_last['latitude'], persisted_last['longitude'], p['latitude'], p['longitude'])
            >= PERSIST_MIN_DISTANCE_M
        ):
            to_persist.append(TruckPosition(
                run=run,
                latitude=p['latitude'],
                longitude=p['longitude'],
                accuracy_m=p['accuracy'],
                speed_kmh=p['speed_kmh'],
                heading=p['heading'],
                recorded_at=ts,
            ))
            persisted_last = {'latitude': p['latitude'], 'longitude': p['longitude'], 'recorded_at': ts}

    if accepted:
        latest = accepted[-1]
        with transaction.atomic():
            TruckPosition.objects.bulk_create(to_persist)
            RouteRun.objects.filter(pk=run.pk).update(
                last_latitude=latest['latitude'],
                last_longitude=latest['longitude'],
                last_heading=latest['heading'],
                last_speed_kmh=latest['speed_kmh'],
                last_position_at=latest['ts'],
            )
        run.last_latitude = latest['latitude']
        run.last_longitude = latest['longitude']
        run.last_heading = latest['heading']
        run.last_speed_kmh = latest['speed_kmh']
        run.last_position_at = latest['ts']

        # Solo la posición más reciente del lote viaja al mapa ciudadano
        if run.status == 'active':
            payload = {
                'type': 'map.truck.position',
                'position': {
                    'run_id': run.id,
                    'latitude': latest['latitude'],
                    'longitude': latest['longitude'],
                    'heading': latest['heading'],
                    'speed_kmh': latest['speed_kmh'],
                    'at': latest['ts'].isoformat(),
                },
            }
            transaction.on_commit(lambda: _send(payload))

    return {
        'accepted': len(accepted),
        'rejected': rejected,
        'stored': len(to_persist),
        'last_position_at': run.last_position_at.isoformat() if run.last_position_at else None,
    }
