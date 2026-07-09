from django.contrib import admin
from .models import Alert

@admin.register(Alert)
class AlertAdmin(admin.ModelAdmin):
    list_display = ('title', 'category', 'severity', 'district', 'status', 'created_at', 'resolved_at')
    list_filter = ('status', 'category', 'severity', 'district')
    search_fields = ('title', 'description', 'address')
    readonly_fields = ('created_at',)
    fieldsets = (
        ('Detalles del Reporte', {
            'fields': ('title', 'description', 'category', 'severity', 'district', 'address', 'image_url')
        }),
        ('Ubicación Geográfica', {
            'fields': ('latitude', 'longitude')
        }),
        ('Moderación e Incidente', {
            'fields': ('status', 'created_at')
        }),
        ('Evidencia de Resolución (Autoridades)', {
            'fields': ('resolution_comment', 'resolution_image_url', 'resolved_at'),
            'description': 'Llenar estos campos cuando el incidente sea solucionado.'
        }),
    )
