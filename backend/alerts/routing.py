from django.urls import re_path
from .consumers import MapLiveConsumer

websocket_urlpatterns = [
    re_path(r"^ws/map/?$", MapLiveConsumer.as_asgi()),
]
