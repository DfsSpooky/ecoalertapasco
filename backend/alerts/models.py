from django.db import models
from django.conf import settings

class Alert(models.Model):
    CATEGORY_CHOICES = [
        ('mineria', 'Minería'),
        ('basura', 'Basura'),
        ('agua', 'Agua'),
        ('aire', 'Aire'),
    ]
    
    SEVERITY_CHOICES = [
        ('critico', 'Crítico'),
        ('medio', 'Medio'),
        ('bajo', 'Bajo'),
    ]
    
    DISTRICT_CHOICES = [
        ('yanacancha', 'Yanacancha'),
        ('chaupimarca', 'Chaupimarca'),
        ('simonBolivar', 'Simón Bolívar'),
    ]
    
    STATUS_CHOICES = [
        ('pending', 'Pendiente'),
        ('verified', 'Verificada'),
        ('solved', 'Solucionada'),
        ('dismissed', 'Descartada'),
    ]

    title = models.CharField(max_length=200)
    description = models.TextField()
    category = models.CharField(max_length=20, choices=CATEGORY_CHOICES)
    severity = models.CharField(max_length=20, choices=SEVERITY_CHOICES)
    district = models.CharField(max_length=30, choices=DISTRICT_CHOICES)
    address = models.CharField(max_length=300)
    
    latitude = models.FloatField()
    longitude = models.FloatField()
    
    status = models.CharField(max_length=20, choices=STATUS_CHOICES, default='pending')
    image_url = models.URLField(max_length=500, blank=True, null=True, verbose_name="Primera Foto de Evidencia")
    image_url_2 = models.URLField(max_length=500, blank=True, null=True, verbose_name="Segunda Foto de Evidencia")
    
    # Usuario que reportó la alerta (requerido para dar trazabilidad al ciudadano)
    reported_by = models.ForeignKey(
        settings.AUTH_USER_MODEL,
        on_delete=models.SET_NULL,
        null=True,
        blank=True,
        related_name='reported_alerts'
    )
    
    # Validación y apoyo comunitario (+1 ciudadano)
    confirmed_by = models.ManyToManyField(
        settings.AUTH_USER_MODEL,
        blank=True,
        related_name='confirmed_alerts'
    )
    confirmations_count = models.PositiveIntegerField(default=0)

    # Campos de moderación y resolución
    resolution_comment = models.TextField(blank=True, null=True)
    resolution_image_url = models.URLField(max_length=500, blank=True, null=True)
    
    created_at = models.DateTimeField(auto_now_add=True)
    resolved_at = models.DateTimeField(blank=True, null=True)
    history_log = models.TextField(blank=True, null=True)

    def __str__(self):
        return f"{self.title} - {self.get_status_display()} ({self.district})"

    class Meta:
        verbose_name = "Alerta Ambiental"
        verbose_name_plural = "Alertas Ambientales"


class WastePoint(models.Model):
    WASTE_TYPE_CHOICES = [
        ('general', 'General'),
        ('organic', 'Orgánico'),
        ('recyclable', 'Reciclable'),
    ]

    FILL_LEVEL_CHOICES = [
        ('low', 'Bajo'),
        ('medium', 'Medio'),
        ('full', 'Lleno'),
    ]

    name = models.CharField(max_length=200, verbose_name="Nombre del Punto")
    description = models.TextField(verbose_name="Descripción")
    address = models.CharField(max_length=255, blank=True, default='', verbose_name="Dirección")
    latitude = models.FloatField(verbose_name="Latitud")
    longitude = models.FloatField(verbose_name="Longitud")
    type = models.CharField(max_length=20, choices=WASTE_TYPE_CHOICES, default='general', verbose_name="Tipo de Residuo")
    fill_level = models.CharField(max_length=20, choices=FILL_LEVEL_CHOICES, default='low', blank=True, verbose_name="Nivel de Llenado")
    truck_schedule = models.CharField(max_length=200, verbose_name="Horario de Recolección")
    image_url = models.CharField(max_length=500, blank=True, default='', verbose_name="URL de Imagen")
    recommendations = models.TextField(blank=True, default='', verbose_name="Recomendaciones Ciudadanas")

    class Meta:
        verbose_name = "Punto de Acopio y Reciclaje"
        verbose_name_plural = "Puntos de Acopio y Reciclaje"

    def __str__(self):
        return f"{self.name} ({self.get_type_display()})"


class SystemConfig(models.Model):
    """Almacena configuraciones del sistema como la API Key de Gemini."""
    key = models.CharField(max_length=100, unique=True, verbose_name="Parámetro de Configuración")
    value = models.TextField(blank=True, default='', verbose_name="Valor / Clave de API")
    description = models.CharField(max_length=255, blank=True, default='', verbose_name="Descripción")
    updated_at = models.DateTimeField(auto_now=True, verbose_name="Última Modificación")

    class Meta:
        verbose_name = "Configuración del Sistema (API Key Gemini)"
        verbose_name_plural = "Configuraciones del Sistema (API Key Gemini)"

    def __str__(self):
        return f"{self.key}: {'*' * min(len(self.value), 8)}"

    @classmethod
    def get_value(cls, key: str, default: str = '') -> str:
        import os
        try:
            obj = cls.objects.filter(key=key).first()
            if obj and obj.value.strip():
                return obj.value.strip()
        except Exception:
            pass
        return os.environ.get(key.upper(), default)

    @classmethod
    def is_gemini_enabled(cls) -> bool:
        val = cls.get_value('gemini_enabled', default='true').strip().lower()
        return val in ('true', '1', 'yes', 'si', 'on')

    @classmethod
    def set_value(cls, key: str, value: str, description: str = ''):
        obj, _ = cls.objects.get_or_create(key=key)
        obj.value = value
        if description:
            obj.description = description
        obj.save()
        return obj



# ---------------------------------------------------------------------------
# Flota de recolección municipal: camiones, rutas, turnos y posiciones GPS
# ---------------------------------------------------------------------------

COLLECTOR_GROUP_NAME = 'recolectores'


def is_collector(user) -> bool:
    """True si el usuario pertenece al grupo de conductores recolectores."""
    if not user or not user.is_authenticated:
        return False
    return user.groups.filter(name=COLLECTOR_GROUP_NAME).exists()


class Truck(models.Model):
    """Camión compactador / recolector de la flota municipal."""

    code = models.CharField(
        max_length=20, unique=True, verbose_name="Código Público",
        help_text="Identificador visible para la ciudadanía (ej. R-03). Nunca se muestra el nombre del conductor.",
    )
    plate = models.CharField(max_length=15, unique=True, verbose_name="Placa")
    description = models.CharField(max_length=200, blank=True, default='', verbose_name="Descripción")
    waste_type = models.CharField(
        max_length=20, choices=WastePoint.WASTE_TYPE_CHOICES, default='general', verbose_name="Tipo de Residuo",
    )
    driver = models.ForeignKey(
        settings.AUTH_USER_MODEL,
        on_delete=models.SET_NULL,
        null=True,
        blank=True,
        related_name='assigned_trucks',
        verbose_name="Conductor Asignado",
    )
    is_active = models.BooleanField(default=True, verbose_name="Operativo")
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        verbose_name = "Camión Recolector"
        verbose_name_plural = "Camiones Recolectores"
        ordering = ['code']

    def __str__(self):
        return f"{self.code} ({self.plate})"


class CollectionRoute(models.Model):
    """Ruta planificada de recolección, trazada sobre el mapa por la autoridad."""

    name = models.CharField(max_length=150, verbose_name="Nombre de la Ruta")
    description = models.TextField(blank=True, default='', verbose_name="Descripción")
    district = models.CharField(
        max_length=30, choices=Alert.DISTRICT_CHOICES, default='yanacancha', verbose_name="Distrito",
    )
    waste_type = models.CharField(
        max_length=20, choices=WastePoint.WASTE_TYPE_CHOICES, default='general', verbose_name="Tipo de Residuo",
    )
    color = models.CharField(max_length=7, default='#16a34a', verbose_name="Color en el Mapa (hex)")
    schedule = models.CharField(
        max_length=200, blank=True, default='', verbose_name="Días y Horario",
        help_text="Ej. Lunes, Miércoles y Viernes 06:00 - 09:00",
    )
    # Trazo de la ruta como lista ordenada de pares [lat, lng] (sin depender de PostGIS)
    path = models.JSONField(default=list, verbose_name="Trazo de la Ruta [[lat, lng], ...]")
    # Puntos de control que dibuja la autoridad en el editor; `path` se recalcula a partir de ellos
    waypoints = models.JSONField(default=list, blank=True, verbose_name="Puntos de Control del Editor")
    snap_to_roads = models.BooleanField(
        default=False, verbose_name="Ajustada a Calles",
        help_text="El trazo sigue la red vial calculada con OSRM a partir de los puntos de control.",
    )
    default_truck = models.ForeignKey(
        Truck, on_delete=models.SET_NULL, null=True, blank=True,
        related_name='default_routes', verbose_name="Camión Habitual",
    )
    is_active = models.BooleanField(default=True, verbose_name="Publicada")
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        verbose_name = "Ruta de Recolección"
        verbose_name_plural = "Rutas de Recolección"
        ordering = ['name']

    def __str__(self):
        return f"{self.name} ({self.get_district_display()})"


class RouteStop(models.Model):
    """Parada ordenada dentro de una ruta (opcionalmente enlazada a un punto de acopio)."""

    route = models.ForeignKey(CollectionRoute, on_delete=models.CASCADE, related_name='stops')
    order = models.PositiveIntegerField(default=0, verbose_name="Orden")
    name = models.CharField(max_length=150, verbose_name="Nombre de la Parada")
    latitude = models.FloatField(verbose_name="Latitud")
    longitude = models.FloatField(verbose_name="Longitud")
    waste_point = models.ForeignKey(
        WastePoint, on_delete=models.SET_NULL, null=True, blank=True,
        related_name='route_stops', verbose_name="Punto de Acopio Enlazado",
    )
    eta_offset_minutes = models.PositiveIntegerField(
        default=0, verbose_name="Minutos desde el inicio",
        help_text="Tiempo estimado de llegada desde que arranca la ruta.",
    )

    class Meta:
        verbose_name = "Parada de Ruta"
        verbose_name_plural = "Paradas de Ruta"
        ordering = ['route', 'order', 'id']

    def __str__(self):
        return f"{self.order}. {self.name}"


class RouteRun(models.Model):
    """Turno de recolección: un camión recorriendo una ruta con su conductor."""

    STATUS_CHOICES = [
        ('active', 'En Recorrido'),
        ('paused', 'Pausado'),
        ('finished', 'Finalizado'),
    ]

    route = models.ForeignKey(CollectionRoute, on_delete=models.PROTECT, related_name='runs', verbose_name="Ruta")
    truck = models.ForeignKey(Truck, on_delete=models.PROTECT, related_name='runs', verbose_name="Camión")
    driver = models.ForeignKey(
        settings.AUTH_USER_MODEL, on_delete=models.SET_NULL, null=True, blank=True,
        related_name='route_runs', verbose_name="Conductor",
    )
    status = models.CharField(max_length=20, choices=STATUS_CHOICES, default='active', verbose_name="Estado")
    started_at = models.DateTimeField(auto_now_add=True, verbose_name="Inicio")
    ended_at = models.DateTimeField(null=True, blank=True, verbose_name="Fin")

    # Última posición conocida (cache desnormalizada para el snapshot del mapa)
    last_latitude = models.FloatField(null=True, blank=True)
    last_longitude = models.FloatField(null=True, blank=True)
    last_heading = models.FloatField(null=True, blank=True)
    last_speed_kmh = models.FloatField(null=True, blank=True)
    last_position_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        verbose_name = "Turno de Recolección"
        verbose_name_plural = "Turnos de Recolección"
        ordering = ['-started_at']
        constraints = [
            # Un camión no puede tener dos turnos abiertos a la vez
            models.UniqueConstraint(
                fields=['truck'],
                condition=models.Q(status__in=['active', 'paused']),
                name='unique_open_run_per_truck',
            ),
        ]

    def __str__(self):
        return f"{self.truck.code} · {self.route.name} ({self.get_status_display()})"


class TruckPosition(models.Model):
    """Muestra histórica de GPS de un turno (para auditoría y reproducción del recorrido)."""

    run = models.ForeignKey(RouteRun, on_delete=models.CASCADE, related_name='positions')
    latitude = models.FloatField()
    longitude = models.FloatField()
    accuracy_m = models.FloatField(null=True, blank=True)
    speed_kmh = models.FloatField(null=True, blank=True)
    heading = models.FloatField(null=True, blank=True)
    recorded_at = models.DateTimeField(verbose_name="Hora del GPS")
    received_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        verbose_name = "Posición GPS"
        verbose_name_plural = "Posiciones GPS"
        ordering = ['run', 'recorded_at']
        indexes = [models.Index(fields=['run', 'recorded_at'])]

    def __str__(self):
        return f"{self.run_id} @ {self.recorded_at:%H:%M:%S}"
