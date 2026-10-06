from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import (
    AlertViewSet,
    WastePointViewSet,
    SystemSettingsView,
    TestGeminiView,
    AnalyzeEvidenceAIView,
)

router = DefaultRouter()
router.register(r'alerts', AlertViewSet, basename='alert')
router.register(r'waste-points', WastePointViewSet, basename='waste-point')

urlpatterns = [
    path('system/settings/', SystemSettingsView.as_view(), name='system_settings'),
    path('system/test-gemini/', TestGeminiView.as_view(), name='test_gemini'),
    path('ai/analyze-evidence/', AnalyzeEvidenceAIView.as_view(), name='analyze_evidence_ai'),
    path('', include(router.urls)),
]
