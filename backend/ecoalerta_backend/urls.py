"""
URL configuration for ecoalerta_backend project.

The `urlpatterns` list routes URLs to views. For more information please see:
    https://docs.djangoproject.com/en/6.0/topics/http/urls/
Examples:
Function views
    1. Add an import:  from my_app import views
    2. Add a URL to urlpatterns:  path('', views.home, name='home')
Class-based views
    1. Add an import:  from other_app.views import Home
    2. Add a URL to urlpatterns:  path('', Home.as_view(), name='home')
Including another URLconf
    1. Import the include() function: from django.urls import include, path
    2. Add a URL to urlpatterns:  path('blog/', include('blog.urls'))
"""

from django.contrib import admin
from django.urls import path, include
from django.conf import settings
from django.conf.urls.static import static
from alerts.views import LoginView, RegisterView, ImageUploadView
from rest_framework_simplejwt.views import TokenRefreshView

import os

ADMIN_PATH = os.environ.get('DJANGO_ADMIN_PATH', 'ecoalerta-secret-admin-portal').strip('/')
if ADMIN_PATH:
    ADMIN_PATH = f"{ADMIN_PATH}/"
else:
    ADMIN_PATH = "admin/"

from django.views.static import serve
from django.urls import re_path

urlpatterns = [
    path(ADMIN_PATH, admin.site.urls),
    path("api/auth/login/", LoginView.as_view(), name="token_login"),
    path("api/auth/register/", RegisterView.as_view(), name="token_register"),
    path("api/auth/token/refresh/", TokenRefreshView.as_view(), name="token_refresh"),
    path("api/upload/", ImageUploadView.as_view(), name="image_upload"),
    path("api/", include("alerts.urls")),
    re_path(r'^media/(?P<path>.*)$', serve, {'document_root': settings.MEDIA_ROOT}),
]


