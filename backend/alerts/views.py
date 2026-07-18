from rest_framework import viewsets, status, permissions
from rest_framework.views import APIView
from rest_framework.response import Response
from rest_framework.parsers import MultiPartParser, FormParser
from rest_framework.authtoken.models import Token
from django.contrib.auth import authenticate, get_user_model
from django.utils import timezone
from django.conf import settings
from django.core.files.storage import default_storage
import os
import uuid
import urllib.request
import urllib.parse
import json

from .models import Alert
from .serializers import AlertSerializer

User = get_user_model()

def get_user_district(user):
    username = user.username.lower() if user and user.username else ""
    if 'yanacancha' in username:
        return 'yanacancha'
    elif 'bolivar' in username or 'simon' in username:
        return 'simonBolivar'
    return 'chaupimarca'  # Supervisor provincial (Pasco / Chaupimarca) por defecto

def send_telegram_notification(alert):
    token = os.environ.get("TELEGRAM_BOT_TOKEN")
    chat_id = os.environ.get("TELEGRAM_CHAT_ID")
    
    if not token or not chat_id:
        return
        
    district_names = {
        'yanacancha': 'Yanacancha 🏔️',
        'chaupimarca': 'Chaupimarca 🏛️',
        'simonBolivar': 'Simón Bolívar ⛏️'
    }
    district_str = district_names.get(alert.district, alert.district)
    severity_str = alert.get_severity_display() if hasattr(alert, 'get_severity_display') else alert.severity
    category_str = alert.get_category_display() if hasattr(alert, 'get_category_display') else alert.category
    
    message = (
        f"🚨 *ALERTA AMBIENTAL CRÍTICA DETECTADA*\n\n"
        f"📌 *Título:* {alert.title}\n"
        f"📂 *Categoría:* {category_str}\n"
        f"⚠️ *Gravedad:* {severity_str} 💀\n"
        f"📍 *Distrito:* {district_str}\n"
        f"🏠 *Dirección:* {alert.address}\n"
        f"📝 *Detalles:* {alert.description}\n"
        f"🗺️ *Coordenadas:* `{alert.latitude}, {alert.longitude}`\n"
        f"🔗 [Ver en el Mapa](https://ecoalerta.tudominio.com)\n"
    )
    
    try:
        url = f"https://api.telegram.org/bot{token}/sendMessage"
        data = urllib.parse.urlencode({
            'chat_id': chat_id,
            'text': message,
            'parse_mode': 'Markdown'
        }).encode('utf-8')
        
        req = urllib.request.Request(url, data=data, method='POST')
        req.add_header('Content-Type', 'application/x-www-form-urlencoded')
        
        # Enviar petición con timeout de 3 segundos para no congelar la API
        with urllib.request.urlopen(req, timeout=3.0) as response:
            pass
    except Exception as e:
        import logging
        logger = logging.getLogger(__name__)
        logger.error(f"Error enviando notificación de Telegram: {e}")

class AlertViewSet(viewsets.ModelViewSet):
    queryset = Alert.objects.all().order_by('-created_at')
    serializer_class = AlertSerializer

    def get_queryset(self):
        queryset = Alert.objects.all().order_by('-created_at')
        
        # Filtrado de distrito para autoridades distritales
        user = self.request.user
        if user.is_authenticated and user.is_staff:
            user_district = get_user_district(user)
            if user_district != 'chaupimarca':
                queryset = queryset.filter(district=user_district)
                
        lat_str = self.request.query_params.get('lat')
        lng_str = self.request.query_params.get('lng')
        radius_str = self.request.query_params.get('radius') # en metros
        
        if lat_str and lng_str and radius_str:
            try:
                lat = float(lat_str)
                lng = float(lng_str)
                radius = float(radius_str)
                
                from django.db import connection
                if connection.vendor == 'postgresql':
                    # Uso de PostGIS nativo y optimizado por índice espacial
                    queryset = queryset.extra(
                        where=[
                            "ST_DistanceSphere(ST_MakePoint(longitude, latitude), ST_MakePoint(%s, %s)) <= %s"
                        ],
                        params=[lng, lat, radius]
                    )
                else:
                    # Fallback matemático para SQLite en desarrollo
                    lat_delta = radius / 111000.0
                    import math
                    lng_delta = radius / (111000.0 * math.cos(math.radians(lat)))
                    
                    min_lat, max_lat = lat - lat_delta, lat + lat_delta
                    min_lng, max_lng = lng - lng_delta, lng + lng_delta
                    
                    queryset = queryset.filter(
                        latitude__range=(min_lat, max_lat),
                        longitude__range=(min_lng, max_lng)
                    )
                    
                    def distance_meters(alat, alng):
                        R = 6371000.0
                        dlat = math.radians(alat - lat)
                        dlng = math.radians(alng - lng)
                        a = math.sin(dlat/2)**2 + math.cos(math.radians(lat)) * math.cos(math.radians(alat)) * math.sin(dlng/2)**2
                        c = 2 * math.atan2(math.sqrt(a), math.sqrt(1-a))
                        return R * c
                    
                    matching_ids = [alert.id for alert in queryset if distance_meters(alert.latitude, alert.longitude) <= radius]
                    queryset = queryset.filter(id__in=matching_ids)
            except ValueError:
                pass
        return queryset

    def get_throttles(self):
        if self.action == 'create':
            self.throttle_scope = 'alerts_create'
        else:
            self.throttle_scope = None
        return super().get_throttles()

    def get_permissions(self):
        if self.action == 'create':
            return [permissions.IsAuthenticated()]
        if self.action in ['partial_update', 'update', 'destroy']:
            return [permissions.IsAdminUser()]
        return [permissions.AllowAny()]

    def perform_create(self, serializer):
        user_str = self.request.user.username if self.request.user.is_authenticated else "Ciudadano"
        now_str = timezone.now().isoformat()
        history_log = f"{now_str}|Reportado por {user_str}"
        alert = serializer.save(history_log=history_log)
        
        # Disparar alerta en Telegram si la severidad es crítica
        if alert.severity == 'critico':
            send_telegram_notification(alert)

    def perform_update(self, serializer):
        new_status = serializer.validated_data.get('status')
        new_district = serializer.validated_data.get('district')
        
        user = self.request.user
        if not user.is_staff:
            from rest_framework.exceptions import PermissionDenied
            raise PermissionDenied("Solo las autoridades pueden realizar esta acción.")

        instance = serializer.instance
        user_district = get_user_district(user)
        is_supervisor = (user_district == 'chaupimarca')
        
        # 1. Autoridad distrital solo gestiona reportes de su distrito
        if not is_supervisor and instance.district != user_district:
            from rest_framework.exceptions import PermissionDenied
            raise PermissionDenied("No tienes permisos para modificar alertas de otros distritos.")

        # 2. Solo el supervisor provincial (Pasco) puede transferir alertas de distrito
        if new_district is not None and new_district != instance.district and not is_supervisor:
            from rest_framework.exceptions import PermissionDenied
            raise PermissionDenied("Solo la autoridad provincial (Pasco) puede transferir reportes de distrito.")

        old_status = instance.status
        old_district = instance.district

        # Guardar cambios
        new_instance = serializer.save()
        
        # Sincronizar resolved_at
        if new_instance.status == 'solved' and not new_instance.resolved_at:
            new_instance.resolved_at = timezone.now()
            new_instance.save()
        elif new_instance.status != 'solved' and new_instance.resolved_at:
            new_instance.resolved_at = None
            new_instance.save()

        # Bitácora
        log_entries = []
        user_str = user.username
        now_str = timezone.now().isoformat()

        if old_status != new_instance.status:
            if new_instance.status == 'solved':
                log_entries.append(f"{now_str}|Solucionado por {user_str}. Evidencia: {new_instance.resolution_comment}")
            elif new_instance.status == 'dismissed':
                log_entries.append(f"{now_str}|Descartado por {user_str}. Motivo: {new_instance.resolution_comment}")
            else:
                log_entries.append(f"{now_str}|Estado cambiado a {new_instance.status} por {user_str}")

        if old_district != new_instance.district:
            log_entries.append(f"{now_str}|Transferido de {old_district} a {new_instance.district} por {user_str}")

        if log_entries:
            current_log = new_instance.history_log or ""
            new_log = current_log + "\n" + "\n".join(log_entries) if current_log else "\n".join(log_entries)
            new_instance.history_log = new_log
            new_instance.save()

class LoginView(APIView):
    authentication_classes = []
    permission_classes = []
    throttle_scope = 'auth'

    def post(self, request):
        username = request.data.get('username')
        password = request.data.get('password')
        if not username or not password:
            return Response({'error': 'Por favor ingresa usuario y contraseña'}, status=status.HTTP_400_BAD_REQUEST)
        
        user = authenticate(username=username, password=password)
        if user:
            token, created = Token.objects.get_or_create(user=user)
            return Response({
                'token': token.key,
                'username': user.username,
                'is_staff': user.is_staff,
                'is_superuser': user.is_superuser
            })
        return Response({'error': 'Credenciales inválidas'}, status=status.HTTP_401_UNAUTHORIZED)

class RegisterView(APIView):
    authentication_classes = []
    permission_classes = []
    throttle_scope = 'auth'

    def post(self, request):
        username = request.data.get('username')
        email = request.data.get('email')
        password = request.data.get('password')

        if not username or not email or not password:
            return Response({'error': 'Por favor completa todos los campos'}, status=status.HTTP_400_BAD_REQUEST)

        if User.objects.filter(username=username).exists():
            return Response({'error': 'El usuario ya existe'}, status=status.HTTP_400_BAD_REQUEST)

        if User.objects.filter(email=email).exists():
            return Response({'error': 'El correo electrónico ya está registrado'}, status=status.HTTP_400_BAD_REQUEST)

        try:
            user = User.objects.create_user(username=username, email=email, password=password)
            token, created = Token.objects.get_or_create(user=user)
            return Response({
                'token': token.key,
                'username': user.username,
                'is_staff': user.is_staff,
                'is_superuser': user.is_superuser
            }, status=status.HTTP_201_CREATED)
        except Exception as e:
            return Response({'error': str(e)}, status=status.HTTP_400_BAD_REQUEST)

class ImageUploadView(APIView):
    permission_classes = [permissions.IsAuthenticated]
    parser_classes = [MultiPartParser, FormParser]
    throttle_scope = 'uploads'

    def post(self, request):
        file_obj = request.FILES.get('image')
        if not file_obj:
            return Response({'error': 'No se cargó ninguna foto'}, status=status.HTTP_400_BAD_REQUEST)
        
        # 1. Validar tamaño (máximo 5 MB)
        max_size = 5 * 1024 * 1024
        if file_obj.size > max_size:
            return Response({'error': 'El tamaño de la imagen no debe exceder los 5 MB'}, status=status.HTTP_400_BAD_REQUEST)
        
        # 2. Validar extensión de archivo
        ext = os.path.splitext(file_obj.name)[1].lower()
        allowed_extensions = ['.jpg', '.jpeg', '.png', '.webp', '.gif']
        if ext not in allowed_extensions:
            return Response({'error': 'Formato de archivo no permitido. Solo se permiten imágenes (jpg, jpeg, png, webp, gif).'}, status=status.HTTP_400_BAD_REQUEST)

        # 3. Generar un nombre de archivo seguro y único (UUID)
        safe_filename = f"{uuid.uuid4()}{ext}"
        
        file_name = default_storage.save(os.path.join('uploads', safe_filename), file_obj)
        file_url = request.build_absolute_uri(settings.MEDIA_URL + file_name)
        return Response({'url': file_url})

import time
from django.http import StreamingHttpResponse

def alerts_sse_stream(request):
    def event_stream():
        last_id = None
        while True:
            try:
                latest = Alert.objects.latest('id')
                if last_id != latest.id:
                    last_id = latest.id
                    yield "data: refresh\n\n"
            except Alert.DoesNotExist:
                pass
            time.sleep(2)
            
    response = StreamingHttpResponse(event_stream(), content_type='text/event-stream')
    response['Cache-Control'] = 'no-cache'
    response['X-Accel-Buffering'] = 'no' # Evitar buffer en Nginx/Traefik
    response['Access-Control-Allow-Origin'] = '*'
    return response


from rest_framework import viewsets
from .models import CollectorRoutePoint, MunicipalDump
from .serializers import CollectorRoutePointSerializer, MunicipalDumpSerializer

class CollectorRoutePointViewSet(viewsets.ModelViewSet):
    queryset = CollectorRoutePoint.objects.all().order_by('order')
    serializer_class = CollectorRoutePointSerializer
    permission_classes = [permissions.AllowAny]


class MunicipalDumpViewSet(viewsets.ModelViewSet):
    queryset = MunicipalDump.objects.all()
    serializer_class = MunicipalDumpSerializer
    permission_classes = [permissions.AllowAny]


from rest_framework.decorators import api_view
from .models import SiteConfiguration
from .serializers import SiteConfigurationSerializer

@api_view(['GET'])
def get_site_settings(request):
    config, created = SiteConfiguration.objects.get_or_create(id=1)
    serializer = SiteConfigurationSerializer(config, context={'request': request})
    return Response(serializer.data)

