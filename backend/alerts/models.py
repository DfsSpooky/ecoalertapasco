from django.db import models

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

    title = models.CharField(max_length=200, verbose_name="Título")
    description = models.TextField(verbose_name="Descripción")
    category = models.CharField(max_length=20, choices=CATEGORY_CHOICES, verbose_name="Categoría")
    severity = models.CharField(max_length=20, choices=SEVERITY_CHOICES, verbose_name="Gravedad")
    district = models.CharField(max_length=30, choices=DISTRICT_CHOICES, verbose_name="Distrito")
    address = models.CharField(max_length=300, verbose_name="Dirección")
    
    latitude = models.FloatField(verbose_name="Latitud")
    longitude = models.FloatField(verbose_name="Longitud")
    
    status = models.CharField(max_length=20, choices=STATUS_CHOICES, default='pending', verbose_name="Estado")
    image_url = models.URLField(max_length=500, blank=True, null=True, verbose_name="URL de Imagen")
    
    # Campos de moderación y resolución
    resolution_comment = models.TextField(blank=True, null=True, verbose_name="Comentario de Resolución")
    resolution_image_url = models.URLField(max_length=500, blank=True, null=True, verbose_name="URL de Imagen de Resolución")
    
    created_at = models.DateTimeField(auto_now_add=True, verbose_name="Fecha de Creación")
    resolved_at = models.DateTimeField(blank=True, null=True, verbose_name="Fecha de Resolución")
    history_log = models.TextField(blank=True, null=True, verbose_name="Historial de Modificaciones")

    class Meta:
        verbose_name = "Alerta Ambiental"
        verbose_name_plural = "Alertas Ambientales"

    def __str__(self):
        return f"{self.title} - {self.get_status_display()} ({self.district})"


class CollectorRoutePoint(models.Model):
    latitude = models.FloatField(verbose_name="Latitud")
    longitude = models.FloatField(verbose_name="Longitud")
    order = models.IntegerField(default=0, help_text="Orden de los puntos para dibujar la ruta.", verbose_name="Orden de Visita")

    class Meta:
        ordering = ['order']
        verbose_name = "Punto de Ruta de Recolector"
        verbose_name_plural = "Puntos de Ruta de Recolector"

    def __str__(self):
        return f"Punto {self.order} ({self.latitude}, {self.longitude})"


class MunicipalDump(models.Model):
    FILL_LEVEL_CHOICES = [
        ('low', 'Bajo'),
        ('medium', 'Medio'),
        ('full', 'Lleno'),
    ]

    TYPE_CHOICES = [
        ('general', 'General'),
        ('organic', 'Orgánico'),
        ('recyclable', 'Reciclable'),
        ('municipalDump', 'Botadero Municipal'),
    ]

    name = models.CharField(max_length=200, verbose_name="Nombre del Punto")
    description = models.TextField(verbose_name="Descripción")
    latitude = models.FloatField(verbose_name="Latitud")
    longitude = models.FloatField(verbose_name="Longitud")
    image = models.ImageField(upload_to='dumps/', blank=True, null=True, verbose_name="Imagen Oficial")
    schedule = models.CharField(max_length=200, help_text="Horario de recolección/limpieza", verbose_name="Horario")
    fill_level = models.CharField(max_length=20, choices=FILL_LEVEL_CHOICES, default='medium', verbose_name="Nivel de Llenado")
    type = models.CharField(max_length=20, choices=TYPE_CHOICES, default='municipalDump', verbose_name="Tipo de Punto")

    class Meta:
        verbose_name = "Punto de Residuos / Botadero"
        verbose_name_plural = "Puntos de Residuos / Botaderos"

    def __str__(self):
        return self.name

