import re

from django.db import transaction
from rest_framework import serializers
from .models import Alert, WastePoint, SystemConfig, Truck, CollectionRoute, RouteStop, RouteRun

class AlertSerializer(serializers.ModelSerializer):
    reporter_name = serializers.CharField(source='reported_by.username', read_only=True, default='Ciudadano')
    has_user_confirmed = serializers.SerializerMethodField()

    class Meta:
        model = Alert
        exclude = ('confirmed_by',)
        read_only_fields = ('created_at', 'history_log', 'reported_by', 'resolved_at', 'confirmations_count')

    def get_has_user_confirmed(self, obj):
        confirmed_ids = self.context.get('confirmed_alert_ids')
        if confirmed_ids is not None:
            return obj.id in confirmed_ids
        request = self.context.get('request')
        if request and hasattr(request, 'user') and request.user.is_authenticated:
            return obj.confirmed_by.filter(id=request.user.id).exists()
        return False


class WastePointSerializer(serializers.ModelSerializer):
    type_display = serializers.CharField(source='get_type_display', read_only=True)
    fill_level_display = serializers.CharField(source='get_fill_level_display', read_only=True)

    class Meta:
        model = WastePoint
        fields = '__all__'


# ---------------------------------------------------------------------------
# Flota y rutas de recolección
# ---------------------------------------------------------------------------

MAX_ROUTE_VERTICES = 2000
HEX_COLOR_RE = re.compile(r'^#[0-9a-fA-F]{6}$')


def _validate_lat_lng(lat, lng):
    if not (-90.0 <= lat <= 90.0) or not (-180.0 <= lng <= 180.0):
        raise serializers.ValidationError("Coordenada fuera de rango.")


MAX_ROUTE_WAYPOINTS = 100


def clean_lat_lng_list(value, min_len, max_len, label):
    """Valida y normaliza una lista [[lat, lng], ...] (6 decimales ≈ 11 cm)."""
    if not isinstance(value, list):
        raise serializers.ValidationError("Debe ser una lista de pares [lat, lng].")
    if len(value) < min_len:
        raise serializers.ValidationError(f"Se necesitan al menos {min_len} {label}.")
    if len(value) > max_len:
        raise serializers.ValidationError(f"Máximo {max_len} {label}.")
    cleaned = []
    for pt in value:
        if not isinstance(pt, (list, tuple)) or len(pt) != 2:
            raise serializers.ValidationError("Cada punto debe ser [lat, lng].")
        try:
            lat, lng = float(pt[0]), float(pt[1])
        except (TypeError, ValueError):
            raise serializers.ValidationError("Las coordenadas deben ser numéricas.")
        if lat != lat or lng != lng:
            raise serializers.ValidationError("Las coordenadas deben ser numéricas.")
        _validate_lat_lng(lat, lng)
        cleaned.append([round(lat, 6), round(lng, 6)])
    return cleaned


class TruckSerializer(serializers.ModelSerializer):
    """Vista completa para autoridades (incluye placa y conductor)."""
    driver_username = serializers.CharField(source='driver.username', read_only=True, default=None)

    class Meta:
        model = Truck
        fields = ('id', 'code', 'plate', 'description', 'waste_type', 'driver', 'driver_username', 'is_active', 'created_at')
        read_only_fields = ('created_at',)


class PublicTruckSerializer(serializers.ModelSerializer):
    """Vista ciudadana: solo el código público, nunca placa ni conductor."""

    class Meta:
        model = Truck
        fields = ('id', 'code', 'waste_type', 'is_active')


class RouteStopSerializer(serializers.ModelSerializer):
    id = serializers.IntegerField(read_only=True)
    waste_point = serializers.PrimaryKeyRelatedField(
        queryset=WastePoint.objects.all(), allow_null=True, required=False,
    )

    class Meta:
        model = RouteStop
        fields = ('id', 'order', 'name', 'latitude', 'longitude', 'waste_point', 'eta_offset_minutes')

    def validate(self, attrs):
        _validate_lat_lng(attrs.get('latitude', 0.0), attrs.get('longitude', 0.0))
        return attrs


class CollectionRouteSerializer(serializers.ModelSerializer):
    stops = RouteStopSerializer(many=True, required=False)
    default_truck_code = serializers.CharField(source='default_truck.code', read_only=True, default=None)
    district_display = serializers.CharField(source='get_district_display', read_only=True)

    class Meta:
        model = CollectionRoute
        fields = (
            'id', 'name', 'description', 'district', 'district_display', 'waste_type', 'color',
            'schedule', 'path', 'waypoints', 'snap_to_roads', 'default_truck', 'default_truck_code', 'is_active', 'stops', 'updated_at',
        )
        read_only_fields = ('updated_at',)

    def validate_color(self, value):
        if not HEX_COLOR_RE.match(value or ''):
            raise serializers.ValidationError("Usa un color hexadecimal de 6 dígitos, ej. #16a34a.")
        return value.lower()

    def validate_path(self, value):
        return clean_lat_lng_list(value, min_len=2, max_len=MAX_ROUTE_VERTICES, label="vértices")

    def validate_waypoints(self, value):
        return clean_lat_lng_list(value, min_len=0, max_len=MAX_ROUTE_WAYPOINTS, label="puntos de control")

    def _replace_stops(self, route, stops_data):
        route.stops.all().delete()
        RouteStop.objects.bulk_create([
            RouteStop(route=route, **{**stop, 'order': stop.get('order', idx)})
            for idx, stop in enumerate(stops_data)
        ])

    @transaction.atomic
    def create(self, validated_data):
        stops_data = validated_data.pop('stops', [])
        route = CollectionRoute.objects.create(**validated_data)
        self._replace_stops(route, stops_data)
        return route

    @transaction.atomic
    def update(self, instance, validated_data):
        stops_data = validated_data.pop('stops', None)
        for attr, val in validated_data.items():
            setattr(instance, attr, val)
        instance.save()
        # Si no se envían paradas (PATCH parcial) se conservan las actuales
        if stops_data is not None:
            self._replace_stops(instance, stops_data)
        return instance


class RouteRunSerializer(serializers.ModelSerializer):
    route_name = serializers.CharField(source='route.name', read_only=True)
    truck_code = serializers.CharField(source='truck.code', read_only=True)
    driver_username = serializers.CharField(source='driver.username', read_only=True, default=None)

    class Meta:
        model = RouteRun
        fields = (
            'id', 'route', 'route_name', 'truck', 'truck_code', 'driver', 'driver_username', 'status',
            'started_at', 'ended_at', 'last_latitude', 'last_longitude', 'last_heading',
            'last_speed_kmh', 'last_position_at',
        )
        read_only_fields = fields
