from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import (
    AlertViewSet,
    WastePointViewSet,
    CollectionRouteViewSet,
    TruckViewSet,
    RouteRunViewSet,
    SyncMapDataView,
    SystemSettingsView,
    TestGeminiView,
    AnalyzeEvidenceAIView,
    UserProfileView,
    PublicAlertVerificationView,
    HealthCheckView,
    GoogleAuthView,
    GoogleAuthConfigView,
)

router = DefaultRouter()
router.register(r'alerts', AlertViewSet, basename='alert')
router.register(r'waste-points', WastePointViewSet, basename='waste-point')
router.register(r'routes', CollectionRouteViewSet, basename='collection-route')
router.register(r'trucks', TruckViewSet, basename='truck')
router.register(r'runs', RouteRunViewSet, basename='route-run')

urlpatterns = [
    path('sync/', SyncMapDataView.as_view(), name='sync_map_data'),
    path('health/', HealthCheckView.as_view(), name='health_check'),
    path('auth/profile/', UserProfileView.as_view(), name='user_profile'),
    path('auth/google/', GoogleAuthView.as_view(), name='alert_google_auth'),
    path('auth/google/config/', GoogleAuthConfigView.as_view(), name='alert_google_auth_config'),
    path('alerts/<int:pk>/verify/', PublicAlertVerificationView.as_view(), name='alert_verify'),
    path('system/settings/', SystemSettingsView.as_view(), name='system_settings'),
    path('system/test-gemini/', TestGeminiView.as_view(), name='test_gemini'),
    path('ai/analyze-evidence/', AnalyzeEvidenceAIView.as_view(), name='analyze_evidence_ai'),
    path('', include(router.urls)),
]
