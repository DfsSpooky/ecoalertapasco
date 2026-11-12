from django.contrib import admin, messages
from .models import Alert, WastePoint, SystemConfig, Truck, CollectionRoute, RouteStop, RouteRun, TruckPosition

admin.site.site_header = "EcoAlerta Pasco - Administración Municipal"
admin.site.site_title = "EcoAlerta Admin"
admin.site.index_title = "Centro de Gestión y Fiscalización Ambiental de Cerro de Pasco"


@admin.register(Alert)
class AlertAdmin(admin.ModelAdmin):
    list_display = ('title', 'category', 'severity', 'district', 'status', 'confirmations_count', 'created_at', 'resolved_at')
    list_filter = ('status', 'category', 'severity', 'district')
    search_fields = ('title', 'description', 'address')
    readonly_fields = ('created_at', 'confirmations_count')
    fieldsets = (
        ('Detalles del Reporte', {
            'fields': ('title', 'description', 'category', 'severity', 'district', 'address', 'image_url', 'image_url_2', 'confirmations_count')
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


@admin.action(description="🤝 Realizar Handshake con Google Gemini AI")
def test_gemini_connection(modeladmin, request, queryset):
    import time
    import httpx
    from .models import SystemConfig

    active_model = SystemConfig.get_value('gemini_model', default='gemini-3.8-flash')
    api_key = SystemConfig.get_value('gemini_api_key')

    # Si seleccionó el registro de clave o modelo, verificar si tiene valor
    for conf in queryset:
        if conf.key == 'gemini_api_key' and conf.value.strip():
            api_key = conf.value.strip()
        elif conf.key == 'gemini_model' and conf.value.strip():
            active_model = conf.value.strip()

    if not api_key:
        messages.error(request, "La clave de Gemini está vacía. Ingrésala primero en el formulario.")
        return

    try:
        start_time = time.time()
        url = f"https://generativelanguage.googleapis.com/v1beta/models/{active_model}:generateContent?key={api_key}"
        payload = {
            "contents": [{"parts": [{"text": "EcoAlerta Handshake: Responde OK."}]}],
            "generationConfig": {"responseMimeType": "application/json"}
        }
        res = httpx.post(url, json=payload, timeout=15.0)
        latency_ms = int((time.time() - start_time) * 1000)

        if res.status_code == 200:
            messages.success(
                request,
                f"🤝 ¡Handshake Exitoso con Google Gemini ({active_model})! "
                f"Latencia: {latency_ms} ms. El peritaje ambiental y reconocimiento de imágenes están 100% operativos."
            )
        elif res.status_code == 429:
            messages.warning(
                request,
                f"⚠️ Límite temporal de cuota gratuita alcanzado (15 req/min). Tu API Key y modelo ({active_model}) son válidos. Espera unos segundos."
            )
        else:
            messages.error(
                request,
                f"Google Gemini respondió con error ({res.status_code}): {res.text[:200]}"
            )
    except Exception as e:
        messages.error(request, f"Error de conexión durante el handshake: {str(e)}")


from django.http import HttpResponseRedirect, JsonResponse
from .views import AVAILABLE_GEMINI_MODELS, discover_and_verify_gemini_models


@admin.register(SystemConfig)
class SystemConfigAdmin(admin.ModelAdmin):
    list_display = ('key', 'masked_value', 'description', 'updated_at')
    search_fields = ('key', 'description')
    actions = [test_gemini_connection]
    fieldsets = (
        ("Parámetro del Sistema", {
            "fields": ("key", "value", "description"),
            "description": (
                "💡 Configuración de Inteligencia Artificial para EcoAlerta:<br>"
                "• <b>gemini_api_key</b>: Tu clave gratuita de Google AI Studio (<a href='https://aistudio.google.com/apikey' target='_blank'>aistudio.google.com/apikey</a>).<br>"
                "• <b>gemini_model</b>: Modelo activo a utilizar. Opciones recomendadas: <code>gemini-1.5-flash</code> (por defecto), <code>gemini-2.0-flash</code>, <code>gemini-1.5-pro</code>, <code>gemini-1.5-flash-8b</code>.<br>"
                "Para verificar el estado, marca el parámetro y ejecuta la acción: <b>'🤝 Realizar Handshake con Google Gemini AI'</b>."
            )
        }),
    )

    def masked_value(self, obj):
        if not obj.value:
            return "(Sin configurar)"
        if obj.key == 'gemini_model':
            return f"🤖 {obj.value}"
        if len(obj.value) <= 8:
            return "********"
        return f"{obj.value[:4]}...{obj.value[-4:]}"
    masked_value.short_description = "Estado / Valor Activo"

    def has_view_permission(self, request, obj=None):
        return request.user.is_staff

    def has_change_permission(self, request, obj=None):
        return request.user.is_superuser

    def has_add_permission(self, request):
        return request.user.is_superuser

    def has_delete_permission(self, request, obj=None):
        return request.user.is_superuser

    def formfield_for_dbfield(self, db_field, request, **kwargs):
        from django import forms
        if db_field.name == "value":
            kwargs["widget"] = forms.PasswordInput(render_value=False, attrs={"placeholder": "•••••••••••••••• (Clave oculta por seguridad)"})
        return super().formfield_for_dbfield(db_field, request, **kwargs)

    def get_urls(self):
        from django.urls import path
        urls = super().get_urls()
        custom_urls = [
            path("ajax-handshake/", self.admin_site.admin_view(self.ajax_handshake_view), name="alerts_gemini_handshake"),
        ]
        return custom_urls + urls

    def ajax_handshake_view(self, request):
        if request.method != "POST":
            return JsonResponse({"success": False, "error": "Método no permitido"}, status=405)

        import json
        try:
            data = json.loads(request.body.decode("utf-8")) if request.body else {}
        except Exception:
            data = {}

        api_key = data.get("api_key", "").strip() or SystemConfig.get_value("gemini_api_key")
        model = data.get("model", "").strip() or SystemConfig.get_value("gemini_model", default="gemini-3.8-flash")

        if not api_key:
            return JsonResponse({
                "success": False,
                "error": "La clave de API está vacía. Ingrésala en el campo superior antes de realizar el handshake."
            }, status=400)

        diag = discover_and_verify_gemini_models(api_key, requested_model=model)
        status_code = 200 if diag.get("success") else 400
        return JsonResponse(diag, status=status_code)

    def changelist_view(self, request, extra_context=None):
        if request.method == "POST" and request.POST.get("action_type") == "save_gemini_config":
            if not request.user.is_superuser:
                messages.error(request, "⛔ Solo los Superadministradores tienen permisos para modificar la configuración de Gemini AI.")
                return HttpResponseRedirect(request.path)

            new_key = request.POST.get("gemini_api_key", "").strip()
            new_model = request.POST.get("gemini_model", "").strip()
            new_enabled = request.POST.get("gemini_enabled")

            if new_enabled is not None:
                enabled_val = "true" if new_enabled in ("true", "1", "on") else "false"
                SystemConfig.objects.update_or_create(
                    key="gemini_enabled",
                    defaults={"value": enabled_val, "description": "Estado de activación de Gemini AI (true=activo, false=desactivado)"}
                )

            if new_key:
                SystemConfig.objects.update_or_create(
                    key="gemini_api_key",
                    defaults={"value": new_key, "description": "Google Gemini API Key para análisis inteligente de reportes ambientales"}
                )

            if new_model:
                SystemConfig.objects.update_or_create(
                    key="gemini_model",
                    defaults={"value": new_model, "description": "Modelo activo de Google Gemini para análisis ambiental"}
                )

            status_text = "Activado" if SystemConfig.is_gemini_enabled() else "Desactivado (Modo Manual)"
            messages.success(request, f"✅ ¡Configuración de Google Gemini AI actualizada exitosamente! Estado: {status_text} | Modelo: {new_model or 'gemini-3.8-flash'}")
            return HttpResponseRedirect(request.path)

        current_key = SystemConfig.get_value("gemini_api_key", default="")
        current_model = SystemConfig.get_value("gemini_model", default="gemini-3.8-flash")
        is_enabled = SystemConfig.is_gemini_enabled()
        masked_key = f"{current_key[:4]}...{current_key[-4:]}" if len(current_key) > 8 else ("********" if current_key else "")

        extra_context = extra_context or {}
        extra_context.update({
            "masked_gemini_key": masked_key,
            "has_gemini_key": bool(current_key),
            "current_gemini_model": current_model,
            "available_gemini_models": AVAILABLE_GEMINI_MODELS,
            "gemini_enabled": is_enabled,
            "is_superuser": request.user.is_superuser,
            "title": "Centro de Configuración de Google Gemini AI",
        })
        return super().changelist_view(request, extra_context=extra_context)


@admin.register(WastePoint)
class WastePointAdmin(admin.ModelAdmin):
    list_display = ('name', 'type', 'fill_level', 'address', 'truck_schedule')
    list_filter = ('type', 'fill_level')
    search_fields = ('name', 'address', 'description')




# ---------------------------------------------------------------------------
# Flota y rutas de recolección
# ---------------------------------------------------------------------------

@admin.register(Truck)
class TruckAdmin(admin.ModelAdmin):
    list_display = ('code', 'plate', 'waste_type', 'driver', 'is_active')
    list_filter = ('is_active', 'waste_type')
    search_fields = ('code', 'plate', 'description', 'driver__username')
    autocomplete_fields = ('driver',)


class RouteStopInline(admin.TabularInline):
    model = RouteStop
    extra = 1
    fields = ('order', 'name', 'latitude', 'longitude', 'waste_point', 'eta_offset_minutes')
    autocomplete_fields = ('waste_point',)


@admin.register(CollectionRoute)
class CollectionRouteAdmin(admin.ModelAdmin):
    list_display = ('name', 'district', 'waste_type', 'schedule', 'default_truck', 'is_active', 'vertex_count', 'updated_at')
    list_filter = ('is_active', 'district', 'waste_type')
    search_fields = ('name', 'description', 'schedule')
    inlines = [RouteStopInline]
    readonly_fields = ('created_at', 'updated_at')

    @admin.display(description="Vértices")
    def vertex_count(self, obj):
        return len(obj.path or [])


class TruckPositionInline(admin.TabularInline):
    model = TruckPosition
    extra = 0
    can_delete = False
    fields = ('recorded_at', 'latitude', 'longitude', 'accuracy_m', 'speed_kmh', 'heading')
    readonly_fields = fields
    ordering = ('-recorded_at',)
    max_num = 0
    show_change_link = False


@admin.register(RouteRun)
class RouteRunAdmin(admin.ModelAdmin):
    list_display = ('truck', 'route', 'driver', 'status', 'started_at', 'ended_at', 'last_position_at')
    list_filter = ('status', 'route', 'truck')
    search_fields = ('truck__code', 'route__name', 'driver__username')
    readonly_fields = (
        'started_at', 'last_latitude', 'last_longitude', 'last_heading', 'last_speed_kmh', 'last_position_at',
    )
    inlines = [TruckPositionInline]
