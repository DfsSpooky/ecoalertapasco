import math

from django.core.management.base import BaseCommand
from django.db import transaction

from alerts.models import CollectionRoute, RouteStop, WastePoint

# Ruta inicial que antes vivía fija en el frontend (TRUCK_ROUTE_POINTS)
INITIAL_ROUTES = [
    {
        'name': 'Ruta Centro Yanacancha - Chaupimarca',
        'description': 'Recorrido base del camión compactador municipal entre la Plaza de Yanacancha y Quiulacocha.',
        'district': 'yanacancha',
        'waste_type': 'general',
        'color': '#16a34a',
        'schedule': 'Lunes a Sábado 06:00 - 09:00',
        'stops': [
            ('Plaza Yanacancha', -10.6625, -76.2555, 0),
            ('Av. Los Próceres', -10.6640, -76.2530, 8),
            ('Hospital Huariaca', -10.6685, -76.2580, 18),
            ('Av. Bolívar Central', -10.6705, -76.2600, 26),
            ('Plaza Quiulacocha', -10.6720, -76.2625, 35),
        ],
    },
]

LINK_RADIUS_M = 80.0


def _distance_m(lat1, lon1, lat2, lon2):
    r = 6371000.0
    p1, p2 = math.radians(lat1), math.radians(lat2)
    dp, dl = math.radians(lat2 - lat1), math.radians(lon2 - lon1)
    a = math.sin(dp / 2) ** 2 + math.cos(p1) * math.cos(p2) * math.sin(dl / 2) ** 2
    return 2 * r * math.asin(math.sqrt(a))


def _nearest_waste_point(lat, lng, points):
    best, best_d = None, LINK_RADIUS_M
    for wp in points:
        d = _distance_m(lat, lng, wp.latitude, wp.longitude)
        if d <= best_d:
            best, best_d = wp, d
    return best


class Command(BaseCommand):
    help = 'Registra la ruta base de recolección (antes fija en el frontend) si no existe ninguna ruta.'

    def add_arguments(self, parser):
        parser.add_argument(
            '--if-empty',
            action='store_true',
            help='Solo inserta si la tabla de rutas está vacía (seguro para producción).',
        )

    @transaction.atomic
    def handle(self, *args, **options):
        if CollectionRoute.objects.exists():
            self.stdout.write(self.style.NOTICE(
                f'Ya existen {CollectionRoute.objects.count()} rutas de recolección. Sin cambios.'
            ))
            return

        waste_points = list(WastePoint.objects.all())
        for spec in INITIAL_ROUTES:
            stops = spec['stops']
            route = CollectionRoute.objects.create(
                name=spec['name'],
                description=spec['description'],
                district=spec['district'],
                waste_type=spec['waste_type'],
                color=spec['color'],
                schedule=spec['schedule'],
                path=[[lat, lng] for _, lat, lng, _ in stops],
            )
            for order, (name, lat, lng, eta) in enumerate(stops):
                RouteStop.objects.create(
                    route=route,
                    order=order,
                    name=name,
                    latitude=lat,
                    longitude=lng,
                    eta_offset_minutes=eta,
                    waste_point=_nearest_waste_point(lat, lng, waste_points),
                )
            self.stdout.write(self.style.SUCCESS(f'Ruta creada: {route.name} ({len(stops)} paradas)'))
