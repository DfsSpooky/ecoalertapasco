"""
Ajuste de rutas a la red vial con OSRM (Open Source Routing Machine).

El editor de rutas envía los puntos de control que marcó la autoridad y el backend
devuelve el trazo siguiendo las calles. Se consulta desde el servidor para:
- poder cambiar de servidor OSRM sin recompilar el frontend (SystemConfig `osrm_url` o env `OSRM_URL`),
- cachear resultados y no golpear el servidor en cada arrastre de un punto.

IMPORTANTE: el servidor público `router.project-osrm.org` es solo de demostración
(sin garantías, con límite de uso). En producción conviene levantar un OSRM propio
con el extracto de Perú de OpenStreetMap.
"""
import hashlib
import json
import logging

import httpx
from django.core.cache import cache

from .models import SystemConfig

logger = logging.getLogger(__name__)

DEFAULT_OSRM_URL = 'https://router.project-osrm.org'
CACHE_TTL_S = 24 * 3600
REQUEST_TIMEOUT_S = 12.0
MAX_OUTPUT_VERTICES = 2000


class SnapError(Exception):
    """Error presentable al usuario (servidor caído, sin ruta posible, etc.)."""


def osrm_base_url() -> str:
    return SystemConfig.get_value('osrm_url', default=DEFAULT_OSRM_URL).rstrip('/')


def _perpendicular_distance(pt, start, end):
    (y, x), (y1, x1), (y2, x2) = pt, start, end
    dx, dy = x2 - x1, y2 - y1
    if dx == 0 and dy == 0:
        return ((x - x1) ** 2 + (y - y1) ** 2) ** 0.5
    t = max(0.0, min(1.0, ((x - x1) * dx + (y - y1) * dy) / (dx * dx + dy * dy)))
    px, py = x1 + t * dx, y1 + t * dy
    return ((x - px) ** 2 + (y - py) ** 2) ** 0.5


def simplify(points, tolerance):
    """Ramer–Douglas–Peucker iterativo (tolerancia en grados; 0.00001 ≈ 1.1 m)."""
    if len(points) < 3:
        return list(points)
    keep = [False] * len(points)
    keep[0] = keep[-1] = True
    stack = [(0, len(points) - 1)]
    while stack:
        a, b = stack.pop()
        best_d, best_i = 0.0, None
        for i in range(a + 1, b):
            d = _perpendicular_distance(points[i], points[a], points[b])
            if d > best_d:
                best_d, best_i = d, i
        if best_i is not None and best_d > tolerance:
            keep[best_i] = True
            stack.append((a, best_i))
            stack.append((best_i, b))
    return [p for p, k in zip(points, keep) if k]


def _fit_vertex_budget(points):
    out = simplify(points, 0.000005)
    tolerance = 0.00001
    while len(out) > MAX_OUTPUT_VERTICES:
        out = simplify(points, tolerance)
        tolerance *= 2
    return out


def snap_waypoints(waypoints):
    """
    Devuelve `{"path": [[lat, lng], ...], "distance_m": float, "duration_s": float}`
    siguiendo las calles entre los puntos de control (en orden).
    """
    key = 'osrm_snap:' + hashlib.sha256(
        (osrm_base_url() + json.dumps(waypoints)).encode('utf-8')
    ).hexdigest()
    cached = cache.get(key)
    if cached:
        return cached

    coords = ';'.join(f'{lng:.6f},{lat:.6f}' for lat, lng in waypoints)
    url = f'{osrm_base_url()}/route/v1/driving/{coords}'
    params = {'overview': 'full', 'geometries': 'geojson', 'continue_straight': 'true'}
    try:
        res = httpx.get(url, params=params, timeout=REQUEST_TIMEOUT_S, headers={'User-Agent': 'EcoAlerta-RouteEditor/1.0'})
    except httpx.HTTPError as exc:
        logger.warning('OSRM no disponible: %s', exc)
        raise SnapError('No se pudo contactar al servidor de calles (OSRM). Intenta de nuevo o desactiva "Ajustar a calles".')

    try:
        data = res.json()
    except ValueError:
        data = {}
    if res.status_code != 200 or data.get('code') != 'Ok' or not data.get('routes'):
        code = data.get('code') or f'HTTP {res.status_code}'
        if code == 'NoRoute':
            raise SnapError('No hay una calle que conecte esos puntos. Mueve el punto de control más cerca de una vía.')
        if res.status_code == 429:
            raise SnapError('El servidor de calles está saturado; espera unos segundos.')
        raise SnapError(f'El servidor de calles no pudo calcular la ruta ({code}).')

    route = data['routes'][0]
    path = [[round(lat, 6), round(lng, 6)] for lng, lat in route['geometry']['coordinates']]
    result = {
        'path': _fit_vertex_budget(path),
        'distance_m': round(float(route.get('distance') or 0.0), 1),
        'duration_s': round(float(route.get('duration') or 0.0), 1),
    }
    cache.set(key, result, CACHE_TTL_S)
    return result
