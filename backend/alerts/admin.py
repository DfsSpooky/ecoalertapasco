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

    class Media:
        css = {
            'all': ('https://unpkg.com/leaflet@1.9.4/dist/leaflet.css',)
        }
        js = (
            'https://unpkg.com/leaflet@1.9.4/dist/leaflet.js',
            'js/admin_map_widget.js',
        )


from .models import CollectorRoutePoint, MunicipalDump

@admin.register(CollectorRoutePoint)
class CollectorRoutePointAdmin(admin.ModelAdmin):
    list_display = ('order', 'latitude', 'longitude')
    ordering = ('order',)

    class Media:
        css = {
            'all': ('https://unpkg.com/leaflet@1.9.4/dist/leaflet.css',)
        }
        js = (
            'https://unpkg.com/leaflet@1.9.4/dist/leaflet.js',
            'js/admin_map_widget.js',
        )


@admin.register(MunicipalDump)
class MunicipalDumpAdmin(admin.ModelAdmin):
    list_display = ('name', 'type', 'fill_level', 'schedule', 'latitude', 'longitude')
    list_filter = ('type', 'fill_level')
    search_fields = ('name', 'description')

    class Media:
        css = {
            'all': ('https://unpkg.com/leaflet@1.9.4/dist/leaflet.css',)
        }
        js = (
            'https://unpkg.com/leaflet@1.9.4/dist/leaflet.js',
            'js/admin_map_widget.js',
        )

