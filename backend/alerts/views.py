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

from .models import Alert
from .serializers import AlertSerializer

User = get_user_model()

class AlertViewSet(viewsets.ModelViewSet):
    queryset = Alert.objects.all().order_by('-created_at')
    serializer_class = AlertSerializer

    def get_queryset(self):
        queryset = Alert.objects.all().order_by('-created_at')
        lat_str = self.request.query_params.get('lat')
        lng_str = self.request.query_params.get('lng')
        radius_str = self.request.query_params.get('radius') # en metros
        
        if lat_str and lng_str and radius_str:
            try:
                lat = float(lat_str)
                lng = float(lng_str)
                radius = float(radius_str)
                
                # Aproximación del bounding box (1 grado latitud ~ 111,000 metros)
                lat_delta = radius / 111000.0
                import math
                lng_delta = radius / (111000.0 * math.cos(math.radians(lat)))
                
                min_lat, max_lat = lat - lat_delta, lat + lat_delta
                min_lng, max_lng = lng - lng_delta, lng + lng_delta
                
                # Bounding box filter (compatible con SQLite y Postgres)
                queryset = queryset.filter(
                    latitude__range=(min_lat, max_lat),
                    longitude__range=(min_lng, max_lng)
                )
                
                # Circular filter (Haversine)
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

    def get_permissions(self):
        if self.action in ['create', 'partial_update', 'update']:
            return [permissions.IsAuthenticated()]
        return [permissions.AllowAny()]

    def perform_create(self, serializer):
        user_str = self.request.user.username if self.request.user.is_authenticated else "Ciudadano"
        now_str = timezone.now().isoformat()
        history_log = f"{now_str}|Reportado por {user_str}"
        serializer.save(history_log=history_log)

    def perform_update(self, serializer):
        new_status = serializer.validated_data.get('status')
        new_district = serializer.validated_data.get('district')
        
        if (new_status in ['solved', 'dismissed'] or new_district is not None) and not self.request.user.is_staff:
            from rest_framework.exceptions import PermissionDenied
            raise PermissionDenied("Solo las autoridades pueden realizar esta acción.")

        instance = serializer.instance
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
        user_str = self.request.user.username if self.request.user.is_authenticated else "Usuario"
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
                'is_staff': user.is_staff
            })
        return Response({'error': 'Credenciales inválidas'}, status=status.HTTP_401_UNAUTHORIZED)

class RegisterView(APIView):
    authentication_classes = []
    permission_classes = []

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
                'is_staff': user.is_staff
            }, status=status.HTTP_201_CREATED)
        except Exception as e:
            return Response({'error': str(e)}, status=status.HTTP_400_BAD_REQUEST)

class ImageUploadView(APIView):
    authentication_classes = []
    permission_classes = []
    parser_classes = [MultiPartParser, FormParser]

    def post(self, request):
        file_obj = request.FILES.get('image')
        if not file_obj:
            return Response({'error': 'No se cargó ninguna foto'}, status=status.HTTP_400_BAD_REQUEST)
        
        file_name = default_storage.save(os.path.join('uploads', file_obj.name), file_obj)
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
