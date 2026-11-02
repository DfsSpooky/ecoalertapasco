from django.apps import AppConfig


class AlertsConfig(AppConfig):
    default_auto_field = "django.db.models.BigAutoField"
    name = "alerts"
    verbose_name = "EcoAlerta - Gestión Ambiental de Pasco"

    def ready(self):
        import alerts.signals  # noqa: F401
