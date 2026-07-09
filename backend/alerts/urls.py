from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import AlertViewSet, alerts_sse_stream

router = DefaultRouter()
router.register(r'alerts', AlertViewSet, basename='alert')

urlpatterns = [
    path('alerts/sse/', alerts_sse_stream, name='alerts-sse'),
    path('', include(router.urls)),
]
