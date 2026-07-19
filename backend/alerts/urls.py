from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import AlertViewSet, alerts_sse_stream, CollectorRoutePointViewSet, MunicipalDumpViewSet, get_site_settings, health_check

router = DefaultRouter()
router.register(r'alerts', AlertViewSet, basename='alert')
router.register(r'collector-route', CollectorRoutePointViewSet, basename='collector-route')
router.register(r'municipal-dumps', MunicipalDumpViewSet, basename='municipal-dump')

urlpatterns = [
    path('health/', health_check, name='health-check'),
    path('alerts/sse/', alerts_sse_stream, name='alerts-sse'),
    path('site-settings/', get_site_settings, name='site-settings'),
    path('', include(router.urls)),
]
