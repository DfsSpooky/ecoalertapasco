from django.contrib import admin
from unfold.admin import ModelAdmin
from .models import Alert

@admin.register(Alert)
class AlertAdmin(ModelAdmin):
    list_display = ('title', 'category', 'severity', 'district', 'status_badge', 'created_at', 'resolved_at')
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

    @admin.display(description="Estado")
    def status_badge(self, obj):
        from django.utils.html import format_html
        colors = {
            'pending': ('bg-amber-100 text-amber-800 dark:bg-amber-950/50 dark:text-amber-300 border border-amber-200 dark:border-amber-900', 'Pendiente'),
            'verified': ('bg-blue-100 text-blue-800 dark:bg-blue-950/50 dark:text-blue-300 border border-blue-200 dark:border-blue-900', 'Verificada'),
            'solved': ('bg-emerald-100 text-emerald-800 dark:bg-emerald-950/50 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-900', 'Solucionada'),
            'dismissed': ('bg-slate-100 text-slate-800 dark:bg-slate-800/50 dark:text-slate-400 border border-slate-200 dark:border-slate-700', 'Descartada'),
        }
        class_str, label = colors.get(obj.status, ('bg-slate-100 text-slate-800', obj.status))
        return format_html(
            '<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-semibold {}">{}</span>',
            class_str,
            label
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
class CollectorRoutePointAdmin(ModelAdmin):
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
class MunicipalDumpAdmin(ModelAdmin):
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


from .models import SiteConfiguration

@admin.register(SiteConfiguration)
class SiteConfigurationAdmin(ModelAdmin):
    list_display = ('__str__', 'is_maintenance_mode')
    
    def has_add_permission(self, request):
        return not SiteConfiguration.objects.exists()
        
    def has_delete_permission(self, request, obj=None):
        return False

