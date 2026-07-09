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

    title = models.CharField(max_length=200)
    description = models.TextField()
    category = models.CharField(max_length=20, choices=CATEGORY_CHOICES)
    severity = models.CharField(max_length=20, choices=SEVERITY_CHOICES)
    district = models.CharField(max_length=30, choices=DISTRICT_CHOICES)
    address = models.CharField(max_length=300)
    
    latitude = models.FloatField()
    longitude = models.FloatField()
    
    status = models.CharField(max_length=20, choices=STATUS_CHOICES, default='pending')
    image_url = models.URLField(max_length=500, blank=True, null=True)
    
    # Campos de moderación y resolución
    resolution_comment = models.TextField(blank=True, null=True)
    resolution_image_url = models.URLField(max_length=500, blank=True, null=True)
    
    created_at = models.DateTimeField(auto_now_add=True)
    resolved_at = models.DateTimeField(blank=True, null=True)
    history_log = models.TextField(blank=True, null=True)

    def __str__(self):
        return f"{self.title} - {self.get_status_display()} ({self.district})"
