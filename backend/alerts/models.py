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

