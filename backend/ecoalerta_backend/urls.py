from django.contrib import admin
from django.urls import path, re_path, include
from django.conf import settings
from django.views.static import serve
from alerts.views import LoginView, LogoutView, RegisterView, ImageUploadView, GoogleAuthView, GoogleAuthConfigView

urlpatterns = [
    path("admin/", admin.site.urls),
    path("api/auth/login/", LoginView.as_view(), name="token_login"),
    path("api/auth/logout/", LogoutView.as_view(), name="token_logout"),
    path("api/auth/register/", RegisterView.as_view(), name="token_register"),
    path("api/auth/google/", GoogleAuthView.as_view(), name="google_auth"),
    path("api/auth/google/config/", GoogleAuthConfigView.as_view(), name="google_auth_config"),
    path("api/upload/", ImageUploadView.as_view(), name="image_upload"),
    path("api/", include("alerts.urls")),
    re_path(r"^media/(?P<path>.*)$", serve, {"document_root": settings.MEDIA_ROOT}),
]
