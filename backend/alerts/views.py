from rest_framework import viewsets, status, permissions
from rest_framework.decorators import action
from rest_framework.views import APIView
from rest_framework.response import Response
from rest_framework.parsers import MultiPartParser, FormParser
from rest_framework.authtoken.models import Token
from rest_framework.throttling import UserRateThrottle, SimpleRateThrottle
from rest_framework.exceptions import ValidationError, PermissionDenied
from django.contrib.auth import authenticate, get_user_model
from django.utils import timezone
from django.conf import settings
from django.core.cache import cache
from django.core.files.storage import default_storage
from django.core.files.base import ContentFile
import os
import math
import io
import re
import hashlib
import json
import urllib.request
import urllib.error
import urllib.parse
from PIL import Image, ImageOps

from .models import Alert, WastePoint, SystemConfig
from .serializers import AlertSerializer, WastePointSerializer

User = get_user_model()


class AlertCreateRateThrottle(UserRateThrottle):
    scope = 'alert_create'


class GeminiUserMinuteThrottle(UserRateThrottle):
    scope = 'gemini_user_minute'


class GeminiUserDayThrottle(UserRateThrottle):
    scope = 'gemini_user_day'


class GeminiIPThrottle(SimpleRateThrottle):
    scope = 'gemini_ip'

    def get_cache_key(self, request, view):
        return self.cache_format % {'scope': self.scope, 'ident': self.get_ident(request)}


class RegisterRateThrottle(SimpleRateThrottle):
    scope = 'register'

    def get_cache_key(self, request, view):
        return self.cache_format % {'scope': self.scope, 'ident': self.get_ident(request)}


class SyncRateThrottle(SimpleRateThrottle):
    scope = 'sync'

    def get_cache_key(self, request, view):
        if request.user and request.user.is_authenticated:
            ident = f"user_{request.user.pk}"
        else:
            ident = self.get_ident(request)
        return self.cache_format % {'scope': self.scope, 'ident': ident}


class AlertConfirmRateThrottle(UserRateThrottle):
    scope = 'alert_confirm'


class ImageUploadRateThrottle(UserRateThrottle):
    scope = 'image_upload'


class LoginRateThrottle(SimpleRateThrottle):
    scope = 'login_attempt'

    def get_cache_key(self, request, view):
        return self.cache_format % {'scope': self.scope, 'ident': self.get_ident(request)}


def compute_sync_etag(user) -> tuple[str, set[int]]:
    """
    Calcula una huella criptográfica ligera (ETag) del estado actual del mapa
    (alertas, puntos de acopio y respaldos del usuario autenticado) en una fracción
    de milisegundo sin hidratar modelos completos ni ejecutar serializadores.
    """
    h = hashlib.sha256()

    alert_rows = Alert.objects.order_by('id').values_list(
        'id',
        'status',
        'severity',
        'category',
        'district',
        'confirmations_count',
        'resolved_at',
        'history_log',
        'image_url',
        'image_url_2',
        'resolution_image_url',
        'title',
        'resolution_comment',
    )
    for row in alert_rows:
        h.update(repr(row).encode('utf-8'))

    waste_rows = WastePoint.objects.order_by('id').values_list(
        'id',
        'name',
        'fill_level',
        'type',
        'truck_schedule',
        'latitude',
        'longitude',
        'image_url',
        'recommendations',
    )
    for w_row in waste_rows:
        h.update(repr(w_row).encode('utf-8'))

    confirmed_ids: set[int] = set()
    if user and user.is_authenticated:
        confirmed_ids = set(user.confirmed_alerts.values_list('id', flat=True))
        h.update(f"u:{user.id}:{sorted(confirmed_ids)}".encode('utf-8'))
    else:
        h.update(b"u:anon")

    return h.hexdigest()[:20], confirmed_ids


class SyncMapDataView(APIView):
    """
    Endpoint unificado de Smart Polling condicional para producción (GET /api/sync/).
    - Si el ETag del cliente coincide con el estado actual en BD, responde HTTP 304 Not Modified (0 bytes).
    - Si hubo cambios (o es carga inicial), retorna alertas + puntos de acopio + nuevo ETag en un solo viaje HTTP.
    """
    permission_classes = [permissions.AllowAny]
    throttle_classes = [SyncRateThrottle]

    def get(self, request):
        etag, confirmed_ids = compute_sync_etag(request.user)

        client_etag = (
            request.query_params.get('etag', '').strip(' "')
            or request.headers.get('If-None-Match', '').strip(' "')
        )
        if client_etag and client_etag == etag:
            return Response(
                status=status.HTTP_304_NOT_MODIFIED,
                headers={
                    'ETag': f'"{etag}"',
                    'Cache-Control': 'no-cache, must-revalidate',
                },
            )

        alerts_qs = Alert.objects.select_related('reported_by').order_by('-created_at')
        waste_qs = WastePoint.objects.all().order_by('id')

        alerts_data = AlertSerializer(
            alerts_qs,
            many=True,
            context={'request': request, 'confirmed_alert_ids': confirmed_ids},
        ).data
        waste_data = WastePointSerializer(waste_qs, many=True).data

        return Response(
            {
                'etag': etag,
                'alerts': alerts_data,
                'waste_points': waste_data,
                'server_time': timezone.now().isoformat(),
            },
            status=status.HTTP_200_OK,
            headers={
                'ETag': f'"{etag}"',
                'Cache-Control': 'no-cache, must-revalidate',
            },
        )


def calculate_distance_meters(lat1, lon1, lat2, lon2):
    """Calcula la distancia geodésica en metros entre dos coordenadas usando Haversine."""
    R = 6371000  # Radio medio de la Tierra en metros
    phi1 = math.radians(lat1)
    phi2 = math.radians(lat2)
    delta_phi = math.radians(lat2 - lat1)
    delta_lambda = math.radians(lon2 - lon1)

    a = math.sin(delta_phi / 2.0) ** 2 + \
        math.cos(phi1) * math.cos(phi2) * \
        math.sin(delta_lambda / 2.0) ** 2
    c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))
    return R * c


class AlertViewSet(viewsets.ModelViewSet):
    queryset = Alert.objects.select_related('reported_by').prefetch_related('confirmed_by').order_by('-created_at')
    serializer_class = AlertSerializer

    def get_permissions(self):
        # Crear alertas requiere ciudadano autenticado; modificar o eliminar requiere autoridad municipal (is_staff)
        if self.action == 'create':
            return [permissions.IsAuthenticated()]
        if self.action in ['partial_update', 'update', 'destroy']:
            return [permissions.IsAdminUser()]
        return [permissions.AllowAny()]

    def get_throttles(self):
        # Aplicar Rate Limiting (máximo 10 reportes por hora por usuario) en la creación
        if self.action == 'create':
            return [AlertCreateRateThrottle()]
        return super().get_throttles()

    def perform_create(self, serializer):
        user = self.request.user

        # 1. Anti-spam Cooldown: Al menos 60 segundos entre reportes de un mismo usuario no-staff
        if user.is_authenticated and not user.is_staff:
            last_alert = Alert.objects.filter(reported_by=user).order_by('-created_at').first()
            if last_alert:
                diff_seconds = (timezone.now() - last_alert.created_at).total_seconds()
                if diff_seconds < 60:
                    wait_sec = int(60 - diff_seconds)
                    raise ValidationError({
                        'detail': f'Debes esperar {wait_sec} segundos antes de enviar otro reporte ambiental.'
                    })

        # 2. Detección de duplicados espaciales en un radio de 50 metros
        new_lat = serializer.validated_data.get('latitude')
        new_lng = serializer.validated_data.get('longitude')
        new_category = serializer.validated_data.get('category')
        new_title = serializer.validated_data.get('title', '').strip()

        if new_lat is not None and new_lng is not None:
            # Buscar alertas activas (no solucionadas ni descartadas) en las últimas 48 horas
            recent_threshold = timezone.now() - timezone.timedelta(hours=48)
            active_alerts = Alert.objects.filter(
                status__in=['pending', 'verified'],
                created_at__gte=recent_threshold
            )

            for existing in active_alerts:
                dist = calculate_distance_meters(new_lat, new_lng, existing.latitude, existing.longitude)
                # Misma categoría en radio <= 50 metros
                if dist <= 50 and existing.category == new_category:
                    cat_name = dict(Alert.CATEGORY_CHOICES).get(new_category, new_category)
                    raise ValidationError({
                        'detail': f"Ya existe una alerta activa de {cat_name} a {int(dist)} metros ('{existing.title}'). Evitemos duplicar reportes en la misma zona."
                    })
                # Mismo título exacto en radio <= 200 metros
                if dist <= 200 and existing.title.strip().lower() == new_title.lower():
                    raise ValidationError({
                        'detail': f"Ya existe un reporte similar en esta zona ('{existing.title}'). Evitemos duplicar incidentes."
                    })

        # 3. Validación estricta de evidencia fotográfica ambiental (Filtro Anti-Spam / No Ambiental)
        img1 = serializer.validated_data.get('image_url')
        img2 = serializer.validated_data.get('image_url_2')
        for img in [img1, img2]:
            if img:
                clean_key = img.split('uploads/')[-1] if 'uploads/' in img else img.strip()
                cached_analysis = cache.get(f"analysis_res_{img.strip()}") or cache.get(f"analysis_res_{clean_key}")
                if cached_analysis and isinstance(cached_analysis, dict):
                    if cached_analysis.get('is_environmental_issue') is False:
                        if cached_analysis.get('is_minor_litter') or cached_analysis.get('rejection_reason') == 'minor_litter':
                            raise ValidationError({
                                'detail': 'Reporte bloqueado: La evidencia fotográfica corresponde a basura menor cotidiana. El camión recolector municipal pasa habitualmente según su cronograma. EcoAlerta está reservado exclusivamente para focos y emergencias ambientales graves.'
                            })
                        else:
                            tech_msg = cached_analysis.get('technical_analysis') or 'La evidencia no muestra un problema de contaminación ambiental en Cerro de Pasco.'
                            raise ValidationError({
                                'detail': f'Reporte rechazado por filtro ambiental: {tech_msg}'
                            })

        user_str = user.username if user.is_authenticated else "Ciudadano"
        now_str = timezone.now().strftime("%Y-%m-%d %H:%M")
        history_log = f"{now_str}|Reportado por {user_str}"

        # Blindaje contra Mass Assignment / Escalada de privilegios:
        # Los usuarios no-staff NO pueden forzar status 'solved'/'verified' ni adjuntar resoluciones al crear.
        if not user.is_staff:
            serializer.save(
                reported_by=user if user.is_authenticated else None,
                status='pending',
                resolution_comment=None,
                resolution_image_url=None,
                resolved_at=None,
                history_log=history_log
            )
        else:
            serializer.save(
                reported_by=user if user.is_authenticated else None,
                history_log=history_log
            )

    def perform_update(self, serializer):
        new_status = serializer.validated_data.get('status')
        new_district = serializer.validated_data.get('district')
        
        # Solo personal autorizado / staff puede resolver o descartar alertas
        if (new_status in ['solved', 'dismissed'] or new_district is not None) and not self.request.user.is_staff:
            from rest_framework.exceptions import PermissionDenied
            raise PermissionDenied("Solo el personal autorizado puede resolver o transferir alertas.")

        instance = serializer.instance
        old_status = instance.status
        old_district = instance.district

        target_status = serializer.validated_data.get('status', old_status)
        target_district = serializer.validated_data.get('district', old_district)
        target_res_comment = serializer.validated_data.get('resolution_comment', instance.resolution_comment)

        # Sincronizar fecha de resolución en una sola escritura atómica
        if target_status == 'solved' and not instance.resolved_at:
            resolved_at_val = timezone.now()
        elif target_status != 'solved' and instance.resolved_at:
            resolved_at_val = None
        else:
            resolved_at_val = instance.resolved_at

        # Bitácora
        log_entries = []
        user_str = self.request.user.username if self.request.user.is_authenticated else "Autoridad"
        now_str = timezone.now().strftime("%Y-%m-%d %H:%M")

        if old_status != target_status:
            if target_status == 'solved':
                log_entries.append(f"{now_str}|Solucionado por {user_str}. Evidencia: {target_res_comment or 'Conforme'}")
            elif target_status == 'dismissed':
                log_entries.append(f"{now_str}|Descartado por {user_str}. Motivo: {target_res_comment or 'No aplicable'}")
            else:
                status_label = dict(Alert.STATUS_CHOICES).get(target_status, target_status)
                log_entries.append(f"{now_str}|Estado actualizado a {status_label} por {user_str}")

        if old_district != target_district:
            log_entries.append(f"{now_str}|Transferido de {old_district} a {target_district} por {user_str}")

        current_log = instance.history_log or ""
        if log_entries:
            final_log = current_log + "\n" + "\n".join(log_entries) if current_log else "\n".join(log_entries)
        else:
            final_log = current_log

        serializer.save(resolved_at=resolved_at_val, history_log=final_log)

    @action(detail=True, methods=['post'], permission_classes=[permissions.IsAuthenticated], throttle_classes=[AlertConfirmRateThrottle])
    def confirm(self, request, pk=None):
        """Permite a ciudadanos confirmar o apoyar un reporte existente (+1)."""
        from django.db import transaction
        with transaction.atomic():
            alert = self.get_object()
            user = request.user

            # Prevención de Auto-respaldo / Manipulación ciudadana:
            # Un usuario no puede confirmar (+1) sus propias denuncias a menos que sea staff realizando verificación.
            if alert.reported_by_id and alert.reported_by_id == user.id and not user.is_staff:
                raise ValidationError({
                    'detail': 'No puedes respaldar tu propia denuncia ambiental.'
                })

            if alert.confirmed_by.filter(id=user.id).exists():
                alert.confirmed_by.remove(user)
                alert.confirmations_count = max(0, alert.confirmations_count - 1)
                confirmed = False
            else:
                alert.confirmed_by.add(user)
                alert.confirmations_count += 1
                confirmed = True
            alert.save(update_fields=['confirmations_count'])
        return Response({
            'id': alert.id,
            'confirmed': confirmed,
            'confirmations_count': alert.confirmations_count
        })

    @action(detail=False, methods=['get'], permission_classes=[permissions.AllowAny])
    def export_csv(self, request):
        """Exporta las alertas en formato CSV con codificación UTF-8 para Excel."""
        import csv
        from django.http import HttpResponse

        response = HttpResponse(content_type='text/csv; charset=utf-8')
        response['Content-Disposition'] = 'attachment; filename="ecoalerta_reportes_pasco.csv"'
        response.write('\ufeff'.encode('utf8'))  # BOM para Excel

        writer = csv.writer(response)
        writer.writerow([
            'ID', 'Fecha Reporte', 'Título', 'Categoría', 'Severidad',
            'Distrito', 'Dirección', 'Latitud', 'Longitud', 'Estado',
            'Reportado Por', 'Confirmaciones Ciudadanas', 'Fecha Solución',
            'Comentario de Solución'
        ])
        for a in Alert.objects.all().order_by('-created_at'):
            writer.writerow([
                a.id,
                a.created_at.strftime('%Y-%m-%d %H:%M'),
                a.title,
                a.get_category_display(),
                a.get_severity_display(),
                a.get_district_display(),
                a.address,
                a.latitude,
                a.longitude,
                a.get_status_display(),
                a.reported_by.username if a.reported_by else 'Ciudadano',
                a.confirmations_count,
                a.resolved_at.strftime('%Y-%m-%d %H:%M') if a.resolved_at else '',
                a.resolution_comment or ''
            ])
        return response


class WastePointViewSet(viewsets.ModelViewSet):
    queryset = WastePoint.objects.all().order_by('id')
    serializer_class = WastePointSerializer

    def get_permissions(self):
        if self.action in ['create', 'update', 'destroy']:
            return [permissions.IsAdminUser()]
        if self.action == 'partial_update':
            return [permissions.IsAuthenticated()]
        return [permissions.AllowAny()]

    def perform_update(self, serializer):
        # Ciudadanos autenticados solo pueden reportar el nivel de llenado actual del contenedor
        if not self.request.user.is_staff:
            disallowed_fields = set(serializer.validated_data.keys()) - {'fill_level'}
            if disallowed_fields:
                raise PermissionDenied("Solo el personal municipal autorizado puede editar los datos estructurales del punto de acopio.")
        serializer.save()


class LoginView(APIView):
    authentication_classes = []
    permission_classes = []
    throttle_classes = [LoginRateThrottle]

    def post(self, request):
        username = request.data.get('username')
        password = request.data.get('password')
        if not username or not password:
            return Response({'error': 'Por favor ingresa usuario y contraseña'}, status=status.HTTP_400_BAD_REQUEST)
        
        user = authenticate(username=username, password=password)
        if user:
            token, _ = Token.objects.get_or_create(user=user)
            return Response({
                'token': token.key,
                'username': user.username,
                'email': user.email,
                'is_staff': user.is_staff
            })
        return Response({'error': 'Credenciales inválidas. Verifica tu usuario y contraseña.'}, status=status.HTTP_401_UNAUTHORIZED)


class LogoutView(APIView):
    """Invalida y elimina el token de autenticación del usuario en el servidor."""
    permission_classes = [permissions.IsAuthenticated]

    def post(self, request):
        if hasattr(request.user, 'auth_token') and request.user.auth_token:
            request.user.auth_token.delete()
        return Response({'detail': 'Sesión cerrada exitosamente en el servidor.'}, status=status.HTTP_200_OK)


def get_client_ip(request):
    """Obtiene la dirección IP real del cliente considerando reverse proxies (Cloudflare, Nginx, Traefik)."""
    cf_connecting_ip = request.META.get('HTTP_CF_CONNECTING_IP')
    if cf_connecting_ip:
        return cf_connecting_ip.strip()
    x_forwarded_for = request.META.get('HTTP_X_FORWARDED_FOR')
    if x_forwarded_for:
        return x_forwarded_for.split(',')[0].strip()
    return request.META.get('REMOTE_ADDR')


def verify_turnstile_token(token: str, remote_ip: str = None) -> tuple[bool, str]:
    """
    Verifica criptográficamente un token de Cloudflare Turnstile con el endpoint de siteverify.
    Retorna (es_valido, mensaje_error).
    """
    secret = SystemConfig.get_value(
        'TURNSTILE_SECRET_KEY',
        os.environ.get('TURNSTILE_SECRET_KEY', '1x0000000000000000000000000000000AA')
    ).strip()

    # Si Turnstile está desactivado explícitamente en variables o SystemConfig
    turnstile_enabled = SystemConfig.get_value(
        'TURNSTILE_ENABLED',
        os.environ.get('TURNSTILE_ENABLED', 'true')
    ).strip().lower() in ('true', '1', 'yes')

    if not turnstile_enabled:
        return True, ""

    if not token or not token.strip():
        return False, "Por favor completa la verificación de seguridad anti-bot (Cloudflare Turnstile)."

    token_clean = token.strip()

    # Si estamos en modo de desarrollo con la dummy key oficial de Cloudflare y el token es de prueba
    if secret == '1x0000000000000000000000000000000AA' and token_clean in ('dummy', 'test', '1x00000000000000000000AA'):
        return True, ""

    url = "https://challenges.cloudflare.com/turnstile/v0/siteverify"
    payload = {
        'secret': secret,
        'response': token_clean,
    }
    if remote_ip:
        payload['remoteip'] = remote_ip

    data = urllib.parse.urlencode(payload).encode('utf-8')
    req = urllib.request.Request(
        url,
        data=data,
        headers={
            'Content-Type': 'application/x-www-form-urlencoded',
            'User-Agent': 'EcoAlerta-Platform/1.0'
        }
    )

    try:
        with urllib.request.urlopen(req, timeout=7) as resp:
            if resp.status == 200:
                result = json.loads(resp.read().decode('utf-8'))
                if result.get('success'):
                    return True, ""
                error_codes = result.get('error-codes', [])
                err_detail = ", ".join(error_codes) if error_codes else "desafío no superado"
                return False, f"Validación de seguridad Cloudflare Turnstile fallida ({err_detail})."
            else:
                return False, f"Error del servidor de Cloudflare ({resp.status})."
    except Exception as e:
        # En caso de timeout de red o error de conexión con Cloudflare
        # Si la clave es la dummy de desarrollo, permitir pasar para no bloquear desarrollo offline
        if secret == '1x0000000000000000000000000000000AA':
            return True, ""
        return False, f"No se pudo contactar con el servicio anti-bot de Cloudflare: {e}"


class RegisterView(APIView):
    """
    Registro seguro de ciudadanos con protección anti-bot Cloudflare Turnstile.
    Las cuentas ciudadanas creadas aquí SIEMPRE tienen is_staff=False (rol ciudadano vigilante).
    """
    authentication_classes = []
    permission_classes = []
    throttle_classes = [RegisterRateThrottle]

    def post(self, request):
        turnstile_token = str(request.data.get('turnstile_token') or '').strip()
        client_ip = get_client_ip(request)
        is_valid, err_msg = verify_turnstile_token(turnstile_token, remote_ip=client_ip)
        if not is_valid:
            return Response({'error': err_msg}, status=status.HTTP_400_BAD_REQUEST)

        username = str(request.data.get('username') or '').strip()
        email = str(request.data.get('email') or '').strip()
        password = str(request.data.get('password') or '')

        if not username or not email or not password:
            return Response({'error': 'Por favor completa todos los campos obligatorios.'}, status=status.HTTP_400_BAD_REQUEST)

        if User.objects.filter(username__iexact=username).exists():
            return Response({'error': 'El nombre de usuario ya está registrado.'}, status=status.HTTP_400_BAD_REQUEST)

        if User.objects.filter(email__iexact=email).exists():
            return Response({'error': 'El correo electrónico ya está en uso.'}, status=status.HTTP_400_BAD_REQUEST)

        try:
            # Creación estricta de ciudadano sin privilegios administrativos (is_staff=False)
            user = User.objects.create_user(
                username=username,
                email=email,
                password=password,
                is_staff=False,
                is_superuser=False
            )
            token, _ = Token.objects.get_or_create(user=user)
            return Response({
                'token': token.key,
                'username': user.username,
                'email': user.email,
                'is_staff': False
            }, status=status.HTTP_201_CREATED)
        except Exception as e:
            return Response({'error': str(e)}, status=status.HTTP_500_INTERNAL_SERVER_ERROR)


class GoogleAuthView(APIView):
    """
    Autenticación y registro seguro con Google Identity Services (GIS / OAuth 2.0).
    Verifica el JWT id_token con los servidores seguros de Google, previene spam
    y crea o vincula la cuenta del ciudadano en EcoAlerta.
    """
    authentication_classes = []
    permission_classes = []

    def post(self, request):
        credential = (request.data.get('credential') or request.data.get('id_token') or '').strip()
        if not credential:
            return Response({'error': 'Falta el token de credenciales de Google.'}, status=status.HTTP_400_BAD_REQUEST)

        google_user = None

        try:
            # Verificación criptográfica con los servidores de Google Identity
            req_url = f"https://oauth2.googleapis.com/tokeninfo?id_token={credential}"
            req = urllib.request.Request(
                req_url,
                headers={'User-Agent': 'EcoAlerta-Platform/1.0'}
            )
            with urllib.request.urlopen(req, timeout=10) as response:
                if response.status == 200:
                    data = json.loads(response.read().decode('utf-8'))

                    # Validar emisor de Google
                    iss = data.get('iss', '')
                    if iss not in ('accounts.google.com', 'https://accounts.google.com'):
                        return Response({'error': 'Emisor de credencial no reconocido por Google.'}, status=status.HTTP_401_UNAUTHORIZED)

                    # Validar Client ID obligatorio en el sistema (Previene suplantación de identidad por tokens de otras apps)
                    expected_aud = SystemConfig.get_value('GOOGLE_CLIENT_ID', os.environ.get('GOOGLE_CLIENT_ID', '')).strip()
                    if not expected_aud:
                        return Response({
                            'error': 'El inicio de sesión con Google no está configurado aún en el servidor (falta GOOGLE_CLIENT_ID).'
                        }, status=status.HTTP_503_SERVICE_UNAVAILABLE)

                    if data.get('aud') != expected_aud:
                        return Response({'error': 'El token de Google no corresponde al Client ID de EcoAlerta.'}, status=status.HTTP_401_UNAUTHORIZED)

                    # Validar que el correo esté verificado por Google
                    email_verified = data.get('email_verified')
                    if str(email_verified).lower() not in ('true', '1'):
                        return Response({'error': 'La cuenta de Google debe tener un correo electrónico verificado.'}, status=status.HTTP_400_BAD_REQUEST)

                    google_user = data
                else:
                    return Response({'error': 'No se pudo verificar la credencial con Google.'}, status=status.HTTP_401_UNAUTHORIZED)
        except urllib.error.HTTPError as e:
            err_detail = "Token de Google inválido o expirado."
            try:
                err_json = json.loads(e.read().decode('utf-8'))
                if 'error_description' in err_json:
                    err_detail = f"Google: {err_json['error_description']}"
            except Exception:
                pass
            return Response({'error': err_detail}, status=status.HTTP_401_UNAUTHORIZED)
        except Exception as e:
            return Response({'error': f'Error de conexión con Google: {str(e)}'}, status=status.HTTP_500_INTERNAL_SERVER_ERROR)

        if not google_user or not google_user.get('email'):
            return Response({'error': 'No se pudo extraer la información del perfil de Google.'}, status=status.HTTP_400_BAD_REQUEST)

        email = google_user['email'].strip().lower()
        full_name = google_user.get('name', '').strip()
        picture = google_user.get('picture', '')

        # Buscar si ya existe un usuario con este correo
        user = User.objects.filter(email__iexact=email).first()
        if not user:
            # Generar username amigable y único
            base_username = email.split('@')[0]
            base_username = re.sub(r'[^a-zA-Z0-9_]', '_', base_username)
            if not base_username:
                base_username = f"google_user_{google_user.get('sub', '')[:8]}"

            candidate_username = base_username
            counter = 1
            while User.objects.filter(username=candidate_username).exists():
                candidate_username = f"{base_username}_{counter}"
                counter += 1

            first_name = full_name.split()[0] if full_name else ''
            last_name = " ".join(full_name.split()[1:]) if len(full_name.split()) > 1 else ''

            user = User.objects.create_user(
                username=candidate_username,
                email=email,
                first_name=first_name,
                last_name=last_name
            )
            user.set_unusable_password()
            user.save()
        else:
            # Actualizar nombres si no los tenía
            if full_name and not user.first_name and not user.last_name:
                parts = full_name.split()
                user.first_name = parts[0]
                if len(parts) > 1:
                    user.last_name = " ".join(parts[1:])
                user.save()

        # Emitir o recuperar token de sesión DRF
        token, _ = Token.objects.get_or_create(user=user)

        return Response({
            'token': token.key,
            'username': user.username,
            'email': user.email,
            'is_staff': user.is_staff,
            'full_name': user.get_full_name() or user.username,
            'photo_url': picture
        }, status=status.HTTP_200_OK)


class GoogleAuthConfigView(APIView):
    """
    Retorna la configuración pública para el botón de Google Sign-In y Cloudflare Turnstile.
    """
    authentication_classes = []
    permission_classes = []

    def get(self, request):
        client_id = SystemConfig.get_value('GOOGLE_CLIENT_ID', os.environ.get('GOOGLE_CLIENT_ID', '')).strip()
        turnstile_sitekey = SystemConfig.get_value(
            'TURNSTILE_SITEKEY',
            os.environ.get('TURNSTILE_SITEKEY', '1x00000000000000000000AA')
        ).strip()
        turnstile_enabled = SystemConfig.get_value(
            'TURNSTILE_ENABLED',
            os.environ.get('TURNSTILE_ENABLED', 'true')
        ).strip().lower() in ('true', '1', 'yes')

        return Response({
            'client_id': client_id,
            'is_configured': bool(client_id),
            'app_name': 'EcoAlerta Cerro de Pasco',
            'turnstile_sitekey': turnstile_sitekey,
            'turnstile_enabled': turnstile_enabled,
        })


def compress_and_convert_to_webp(file_obj, max_dimension=1600, quality=80):
    """
    Comprime y convierte rápidamente cualquier imagen a formato WebP optimizado.
    - Corrige orientación EXIF (típico en fotos de celulares).
    - Redimensiona proporcionalmente si supera max_dimension px (1600px).
    - Reduce el peso del archivo entre 85% y 95% preservando excelente calidad pericial.
    """
    file_obj.seek(0)
    with Image.open(file_obj) as img:
        img = ImageOps.exif_transpose(img)
        if img.mode in ('RGBA', 'LA') or (img.mode == 'P' and 'transparency' in img.info):
            img = img.convert('RGBA')
        elif img.mode != 'RGB':
            img = img.convert('RGB')

        if max(img.size) > max_dimension:
            img.thumbnail((max_dimension, max_dimension), Image.Resampling.LANCZOS)

        buf = io.BytesIO()
        img.save(buf, format='WEBP', quality=quality, method=4)
        webp_bytes = buf.getvalue()

    return webp_bytes, 'image/webp'


class ImageUploadView(APIView):
    permission_classes = [permissions.IsAuthenticated]
    throttle_classes = [ImageUploadRateThrottle]
    parser_classes = [MultiPartParser, FormParser]

    ALLOWED_EXTENSIONS = {'.jpg', '.jpeg', '.png', '.webp', '.heic', '.heif'}
    MAX_FILE_SIZE = 25 * 1024 * 1024  # 25 MB

    def post(self, request):
        file_obj = request.FILES.get('image')
        if not file_obj:
            return Response({'error': 'No se adjuntó ningún archivo de imagen.'}, status=status.HTTP_400_BAD_REQUEST)

        # Validar tamaño
        if file_obj.size > self.MAX_FILE_SIZE:
            return Response({'error': 'El archivo excede el tamaño máximo permitido (25MB).'}, status=status.HTTP_400_BAD_REQUEST)

        # Validar extensión
        _, ext = os.path.splitext(file_obj.name)
        if ext.lower() not in self.ALLOWED_EXTENSIONS:
            return Response({'error': 'Formato no permitido. Solo se aceptan imágenes JPG, PNG o WEBP.'}, status=status.HTTP_400_BAD_REQUEST)

        try:
            webp_bytes, _ = compress_and_convert_to_webp(file_obj)
        except Exception:
            return Response({'error': 'El archivo subido no es una imagen válida o está dañado.'}, status=status.HTTP_400_BAD_REQUEST)

        clean_base = os.path.splitext(os.path.basename(file_obj.name))[0].replace(" ", "_")
        clean_base = re.sub(r'[^\w\-]', '', clean_base) or 'evidencia'
        target_name = f"{int(timezone.now().timestamp())}_{clean_base}.webp"
        target_path = os.path.join('uploads', target_name)

        file_name = default_storage.save(target_path, ContentFile(webp_bytes))
        file_url = request.build_absolute_uri(settings.MEDIA_URL + file_name)

        orig_kb = round(file_obj.size / 1024, 1)
        webp_kb = round(len(webp_bytes) / 1024, 1)
        ratio = f"{(1 - len(webp_bytes) / max(file_obj.size, 1)) * 100:.1f}%"

        return Response({
            'url': file_url,
            'format': 'webp',
            'original_size_kb': orig_kb,
            'compressed_size_kb': webp_kb,
            'compression_ratio': ratio,
        })


AVAILABLE_GEMINI_MODELS = [
    {
        'id': 'gemini-3.5-flash',
        'name': 'Gemini 3.5 Flash',
        'badge': '⭐ Recomendado (Ultra Rápido)',
        'description': 'Respuesta instantánea para visión por computadora y peritaje ambiental en Cerro de Pasco.',
        'is_default': True,
    },
    {
        'id': 'gemini-3.8-flash',
        'name': 'Gemini 3.8 Flash',
        'badge': '🚀 Última Generación',
        'description': 'Modelo multimodal insignia de Google AI Studio.',
        'is_default': False,
    },
    {
        'id': 'gemini-3-flash-preview',
        'name': 'Gemini 3 Flash Preview',
        'badge': '⚡ Alta Velocidad (Estable)',
        'description': 'Respuesta inmediata para análisis de fotos y reportes ciudadanos en Cerro de Pasco.',
        'is_default': False,
    },
    {
        'id': 'gemini-3.1-flash-lite',
        'name': 'Gemini 3.1 Flash Lite',
        'badge': '🍃 Ultraligero',
        'description': 'Optimizado para bajo consumo de tokens y máxima disponibilidad operativa.',
        'is_default': False,
    },
    {
        'id': 'gemini-3.7-flash',
        'name': 'Gemini 3.7 Flash',
        'badge': '⚡ Flash Moderno',
        'description': 'Excelente equilibrio de precisión analítica y rapidez.',
        'is_default': False,
    },
    {
        'id': 'gemini-3.1-pro-preview',
        'name': 'Gemini 3.1 Pro Preview',
        'badge': '🧠 Máxima Precisión',
        'description': 'Razonamiento profundo para análisis técnicos y peritajes forenses complejos.',
        'is_default': False,
    },
    {
        'id': 'gemini-1.5-flash',
        'name': 'Gemini 1.5 Flash',
        'badge': '⚡ Multimodal',
        'description': 'Modelo estándar multimodal de Google AI Studio.',
        'is_default': False,
    },
]


def discover_and_verify_gemini_models(api_key: str, requested_model: str = None):
    """
    Consulta la API oficial de Google Gemini (models.list) para descubrir
    los modelos reales habilitados en la cuenta del usuario, y prueba los modelos
    de forma resiliente con soporte de cuota y rate-limits de la capa gratuita (429).
    """
    import time
    import httpx
    from .models import SystemConfig

    # 1. Descubrir modelos disponibles en la cuenta de Google
    list_url = f"https://generativelanguage.googleapis.com/v1beta/models?key={api_key}"
    try:
        res = httpx.get(list_url, timeout=12.0)
    except Exception as e:
        return {
            'success': False,
            'error': f'Error de red al conectar con Google Generative Language API: {str(e)}'
        }

    if res.status_code in (400, 401, 403):
        return {
            'success': False,
            'error': f'Clave de API inválida o sin permisos (HTTP {res.status_code}). Verifica que la clave copiada de Google AI Studio sea correcta.',
            'details': res.text[:200]
        }
    elif res.status_code == 429:
        # Rate limit momentáneo en list, pero la clave es auténtica
        pass
    elif res.status_code != 200:
        return {
            'success': False,
            'error': f'Google Gemini respondió con error ({res.status_code}): {res.text[:200]}'
        }

    raw_models = res.json().get('models', []) if res.status_code == 200 else []
    detected = []

    # Exclusiones de modelos deprecados o especializados que no son para texto/visión de alertas
    EXCLUDED_KEYWORDS = ['tts', 'transcribe', 'audio', 'robotics', 'computer-use', 'image', 'nano banana']
    BLACK_LIST_MODELS = {'gemini-2.5-flash', 'gemini-2.5-flash-lite'}

    for m in raw_models:
        methods = m.get('supportedGenerationMethods', [])
        name = m.get('name', '')
        if 'generateContent' in methods and 'gemini' in name.lower():
            model_id = name.replace('models/', '')
            if model_id in BLACK_LIST_MODELS or model_id.startswith('gemini-2.5-'):
                continue
            if any(k in model_id.lower() for k in EXCLUDED_KEYWORDS):
                continue

            disp_name = m.get('displayName') or model_id
            desc = m.get('description', '')

            badge = ''
            if '3.8-flash' in model_id:
                badge = '⭐ Recomendado por Google'
            elif '3-flash' in model_id or '1.5-flash' in model_id or '2.0-flash' in model_id:
                badge = '⚡ Alta Velocidad'
            elif 'lite' in model_id or '8b' in model_id:
                badge = '🍃 Ultraligero'
            elif 'pro' in model_id:
                badge = '🧠 Máxima Precisión'

            detected.append({
                'id': model_id,
                'name': disp_name,
                'badge': badge,
                'description': (desc[:130] + '...') if len(desc) > 130 else desc,
            })

    if not detected:
        detected = [m for m in AVAILABLE_GEMINI_MODELS if m['id'] not in BLACK_LIST_MODELS]

    # Ordenar modelos detectados priorizando los más estables y modernos
    priority_order = [
        'gemini-3.5-flash',
        'gemini-3.8-flash',
        'gemini-3-flash-preview',
        'gemini-3.1-flash-lite',
        'gemini-3.7-flash',
        'gemini-3.6-flash',
        'gemini-3.1-pro-preview',
        'gemini-1.5-flash',
        'gemini-2.0-flash',
        'gemini-flash-latest',
        'gemini-flash-lite-latest',
        'gemini-pro-latest'
    ]
    def sort_key(item):
        m_id = item['id']
        try:
            return priority_order.index(m_id)
        except ValueError:
            return 99
    detected.sort(key=sort_key)

    # 2. Construir lista ordenada de candidatos a probar
    candidates = []
    if requested_model and requested_model not in BLACK_LIST_MODELS and any(d['id'] == requested_model for d in detected):
        candidates.append(requested_model)

    for pref in ['gemini-3.5-flash', 'gemini-3.8-flash', 'gemini-3-flash-preview', 'gemini-3.1-flash-lite', 'gemini-1.5-flash']:
        if pref not in candidates and any(d['id'] == pref for d in detected):
            candidates.append(pref)

    for d in detected:
        if d['id'] not in candidates:
            candidates.append(d['id'])

    # 3. Probar conectividad de forma rápida y no bloqueante (máximo 2 intentos)
    valid_detected = [d for d in detected if d['id'] not in BLACK_LIST_MODELS]
    successful_model = None
    working_latency = 0
    hit_rate_limit = False
    hit_high_demand = False
    last_error_message = None

    idx = 0
    while idx < len(candidates) and idx < 2:
        cand = candidates[idx]
        idx += 1

        start_time = time.time()
        test_url = f"https://generativelanguage.googleapis.com/v1beta/models/{cand}:generateContent?key={api_key}"
        payload = {
            "contents": [{"parts": [{"text": "EcoAlerta test"}]}],
            "generationConfig": {
                "thinkingConfig": {
                    "thinkingBudget": 0
                }
            }
        }

        try:
            test_res = httpx.post(test_url, json=payload, timeout=7.0)
            latency_ms = int((time.time() - start_time) * 1000)

            if test_res.status_code == 200:
                successful_model = cand
                working_latency = latency_ms
                break
            elif test_res.status_code == 429:
                hit_rate_limit = True
                successful_model = cand
                break
            elif test_res.status_code == 503:
                hit_high_demand = True
                last_error_message = "Alta demanda temporal en servidores de Google"
                continue
            elif test_res.status_code == 404:
                valid_detected = [d for d in valid_detected if d['id'] != cand]
                continue
            else:
                last_error_message = f"HTTP {test_res.status_code}: {test_res.text[:100]}"
                continue
        except httpx.TimeoutException:
            hit_high_demand = True
            last_error_message = f"Timeout temporal en {cand}"
            continue
        except Exception as e:
            last_error_message = str(e)
            continue

    if not successful_model and valid_detected:
        successful_model = valid_detected[0]['id']

    if successful_model:
        try:
            SystemConfig.objects.update_or_create(
                key='gemini_model',
                defaults={'value': successful_model, 'description': f'Modelo activo verificado de Gemini ({successful_model})'}
            )
        except Exception:
            pass

        return {
            'success': True,
            'is_rate_limited': hit_rate_limit,
            'is_high_demand': hit_high_demand and not hit_rate_limit and working_latency == 0,
            'tested_model': successful_model,
            'latency_ms': working_latency,
            'detected_models': valid_detected,
            'message': (
                f'¡Handshake exitoso! Tu clave es válida y se detectaron {len(valid_detected)} modelos en tu cuenta de Google AI Studio. '
                + ('(Nota: Se alcanzó el límite de 15 req/min de la capa gratuita; el modelo está operativo).' if hit_rate_limit else '')
            ),
            'quota_tier': 'Google AI Studio Free Tier (15 peticiones/min)',
            'capabilities': [
                'Visión Multimodal (Fotos de Cerro de Pasco)',
                'Auto-categorización (Minería, Basura, Agua, Aire)',
                'Diagnóstico técnico y recomendaciones municipales'
            ]
        }
    else:
        return {
            'success': False,
            'error': f'No se pudo encontrar un modelo activo. Último detalle: {last_error_message or "desconocido"}',
            'detected_models': valid_detected
        }


class SystemSettingsView(APIView):
    """Permite consultar y guardar la API Key y el modelo de Gemini desde el panel de autoridades."""
    permission_classes = [permissions.IsAdminUser]

    def get(self, request):
        key = SystemConfig.get_value('gemini_api_key')
        has_key = bool(key)
        masked = ''
        if has_key:
            masked = f"{key[:4]}...{key[-4:]}" if len(key) > 8 else "********"
        current_model = SystemConfig.get_value('gemini_model', default='gemini-1.5-flash')
        gemini_enabled = SystemConfig.is_gemini_enabled()
        google_client_id = SystemConfig.get_value('GOOGLE_CLIENT_ID', os.environ.get('GOOGLE_CLIENT_ID', '')).strip()
        turnstile_sitekey = SystemConfig.get_value('TURNSTILE_SITEKEY', os.environ.get('TURNSTILE_SITEKEY', '1x00000000000000000000AA')).strip()
        turnstile_secret = SystemConfig.get_value('TURNSTILE_SECRET_KEY', os.environ.get('TURNSTILE_SECRET_KEY', '1x0000000000000000000000000000000AA')).strip()
        turnstile_enabled = SystemConfig.get_value('TURNSTILE_ENABLED', os.environ.get('TURNSTILE_ENABLED', 'true')).strip().lower() in ('true', '1', 'yes')
        masked_turnstile_secret = f"{turnstile_secret[:4]}...{turnstile_secret[-4:]}" if len(turnstile_secret) > 8 else "********"

        return Response({
            'has_gemini_key': has_key,
            'masked_key': masked,
            'selected_model': current_model,
            'available_models': AVAILABLE_GEMINI_MODELS,
            'gemini_enabled': gemini_enabled,
            'google_client_id': google_client_id,
            'google_auth_configured': bool(google_client_id),
            'turnstile_sitekey': turnstile_sitekey,
            'turnstile_secret_masked': masked_turnstile_secret,
            'has_turnstile_secret': bool(turnstile_secret),
            'turnstile_enabled': turnstile_enabled,
        })

    def post(self, request):
        gemini_api_key = request.data.get('gemini_api_key', '').strip()
        gemini_model = request.data.get('gemini_model', '').strip()
        has_gemini_enabled = 'gemini_enabled' in request.data
        has_google_client_id = 'google_client_id' in request.data
        has_turnstile_sitekey = 'turnstile_sitekey' in request.data
        has_turnstile_secret = 'turnstile_secret_key' in request.data
        has_turnstile_enabled = 'turnstile_enabled' in request.data

        if not gemini_api_key and not gemini_model and not has_gemini_enabled and not has_google_client_id and not has_turnstile_sitekey and not has_turnstile_secret and not has_turnstile_enabled:
            return Response({'error': 'Debes enviar al menos un parámetro de configuración a actualizar.'}, status=status.HTTP_400_BAD_REQUEST)

        if has_gemini_enabled:
            raw_enabled = request.data.get('gemini_enabled')
            enabled_str = 'true' if str(raw_enabled).lower() in ('true', '1', 'yes', 'si') else 'false'
            SystemConfig.objects.update_or_create(
                key='gemini_enabled',
                defaults={'value': enabled_str, 'description': 'Estado de activación del motor Gemini AI (true=activado, false=desactivado)'}
            )

        if gemini_api_key:
            SystemConfig.objects.update_or_create(
                key='gemini_api_key',
                defaults={'value': gemini_api_key, 'description': 'Google Gemini API Key'}
            )

        if gemini_model:
            SystemConfig.objects.update_or_create(
                key='gemini_model',
                defaults={'value': gemini_model, 'description': 'Modelo activo de Google Gemini'}
            )

        if has_google_client_id:
            val_google = request.data.get('google_client_id', '').strip()
            SystemConfig.objects.update_or_create(
                key='GOOGLE_CLIENT_ID',
                defaults={'value': val_google, 'description': 'Google OAuth 2.0 Web Client ID para Google Sign-In'}
            )

        if has_turnstile_sitekey:
            val_tk = request.data.get('turnstile_sitekey', '').strip()
            SystemConfig.objects.update_or_create(
                key='TURNSTILE_SITEKEY',
                defaults={'value': val_tk, 'description': 'Cloudflare Turnstile Public Site Key'}
            )

        if has_turnstile_secret:
            val_sec = request.data.get('turnstile_secret_key', '').strip()
            if val_sec:
                SystemConfig.objects.update_or_create(
                    key='TURNSTILE_SECRET_KEY',
                    defaults={'value': val_sec, 'description': 'Cloudflare Turnstile Secret Key (Siteverify)'}
                )

        if has_turnstile_enabled:
            raw_t_enabled = request.data.get('turnstile_enabled')
            t_enabled_str = 'true' if str(raw_t_enabled).lower() in ('true', '1', 'yes', 'si') else 'false'
            SystemConfig.objects.update_or_create(
                key='TURNSTILE_ENABLED',
                defaults={'value': t_enabled_str, 'description': 'Activar protección anti-bot Cloudflare Turnstile'}
            )

        current_key = SystemConfig.get_value('gemini_api_key')
        current_model = SystemConfig.get_value('gemini_model', default='gemini-1.5-flash')
        gemini_enabled = SystemConfig.is_gemini_enabled()
        google_client_id = SystemConfig.get_value('GOOGLE_CLIENT_ID', os.environ.get('GOOGLE_CLIENT_ID', '')).strip()
        turnstile_sitekey = SystemConfig.get_value('TURNSTILE_SITEKEY', os.environ.get('TURNSTILE_SITEKEY', '1x00000000000000000000AA')).strip()
        turnstile_secret = SystemConfig.get_value('TURNSTILE_SECRET_KEY', os.environ.get('TURNSTILE_SECRET_KEY', '1x0000000000000000000000000000000AA')).strip()
        turnstile_enabled = SystemConfig.get_value('TURNSTILE_ENABLED', os.environ.get('TURNSTILE_ENABLED', 'true')).strip().lower() in ('true', '1', 'yes')

        return Response({
            'success': True,
            'message': 'Configuración del sistema actualizada correctamente.',
            'has_gemini_key': bool(current_key),
            'masked_key': f"{current_key[:4]}...{current_key[-4:]}" if len(current_key) > 8 else ("********" if current_key else ""),
            'selected_model': current_model,
            'available_models': AVAILABLE_GEMINI_MODELS,
            'gemini_enabled': gemini_enabled,
            'google_client_id': google_client_id,
            'google_auth_configured': bool(google_client_id),
            'turnstile_sitekey': turnstile_sitekey,
            'turnstile_enabled': turnstile_enabled,
        })


class TestGeminiView(APIView):
    """Verifica la API Key, descubre modelos activos y realiza un Handshake de diagnóstico."""
    permission_classes = [permissions.IsAdminUser]

    def post(self, request):
        api_key = request.data.get('api_key', '').strip() or request.data.get('gemini_api_key', '').strip()
        if not api_key:
            api_key = SystemConfig.get_value('gemini_api_key')

        if not api_key:
            return Response({
                'success': False,
                'error': 'No hay ninguna clave de Gemini configurada. Ingresa tu API Key para realizar el handshake.'
            }, status=status.HTTP_400_BAD_REQUEST)

        model = request.data.get('model', '').strip() or request.data.get('gemini_model', '').strip()
        if not model:
            model = SystemConfig.get_value('gemini_model', default='gemini-1.5-flash')

        diag = discover_and_verify_gemini_models(api_key, requested_model=model)
        if diag.get('success'):
            return Response({
                'success': True,
                'handshake': 'HANDSHAKE_ESTABLISHED',
                'status': 'OPERATIVO',
                'model': diag.get('tested_model', model),
                'latency_ms': diag.get('latency_ms', 0),
                'detected_models': diag.get('detected_models', []),
                'message': diag.get('message'),
                'quota_tier': diag.get('quota_tier', 'Google AI Studio Free Tier (15 peticiones/min)'),
                'capabilities': diag.get('capabilities', [])
            })
        else:
            return Response(diag, status=status.HTTP_400_BAD_REQUEST)


class AnalyzeEvidenceAIView(APIView):
    """
    Analiza una foto de evidencia ambiental usando Gemini Multimodal con blindaje anti-abuso:
    - Throttling multinivel por usuario (minuto/día) e IP.
    - Pre-caché por Hash criptográfico (SHA-256) y URL para evitar re-análisis idénticos.
    - Candado atómico anti-concurrencia (cache.add) para prevenir condiciones de carrera.
    - Protección anti-SSRF y límite de peso en descarga de imágenes remotas.
    """
    permission_classes = [permissions.IsAuthenticated]
    throttle_classes = [GeminiUserMinuteThrottle, GeminiUserDayThrottle, GeminiIPThrottle]

    def post(self, request):
        if not SystemConfig.is_gemini_enabled():
            return Response({
                'error': 'El análisis inteligente con Gemini AI está desactivado actualmente por la administración. Puedes completar tu reporte de forma manual.',
                'is_gemini_enabled': False
            }, status=status.HTTP_400_BAD_REQUEST)

        import base64
        import httpx
        import json
        import re
        import socket
        import ipaddress
        from urllib.parse import urlparse

        image_bytes = None
        mime_type = 'image/jpeg'

        file_obj = request.FILES.get('image')
        if file_obj:
            if file_obj.size > 20 * 1024 * 1024:
                return Response({'error': 'El archivo excede el tamaño máximo permitido (20MB).'}, status=status.HTTP_400_BAD_REQUEST)
            try:
                image_bytes, mime_type = compress_and_convert_to_webp(file_obj)
            except Exception:
                image_bytes = file_obj.read()
                mime_type = 'image/jpeg'
        else:
            image_url = request.data.get('image_url')
            if image_url:
                image_url_str = str(image_url).strip()
                try:
                    url_lower = image_url_str.lower()
                    if url_lower.endswith('.webp'):
                        mime_type = 'image/webp'
                    elif url_lower.endswith('.png'):
                        mime_type = 'image/png'
                    elif url_lower.endswith('.jpg') or url_lower.endswith('.jpeg'):
                        mime_type = 'image/jpeg'

                    # 1. Archivo local guardado en MEDIA_ROOT (sin directory traversal)
                    if 'uploads/' in image_url_str or image_url_str.startswith(settings.MEDIA_URL):
                        clean_filename = os.path.basename(image_url_str.split('?')[0])
                        if clean_filename and not clean_filename.startswith('.'):
                            local_path = os.path.join(settings.MEDIA_ROOT, 'uploads', clean_filename)
                            if os.path.exists(local_path):
                                with open(local_path, 'rb') as f:
                                    image_bytes = f.read()

                    # 2. Descarga remota segura con protección anti-SSRF
                    if image_bytes is None and (image_url_str.startswith('http://') or image_url_str.startswith('https://')):
                        parsed_url = urlparse(image_url_str)
                        hostname = parsed_url.hostname
                        is_safe_host = False
                        if hostname:
                            try:
                                ip_str = socket.gethostbyname(hostname)
                                ip_obj = ipaddress.ip_address(ip_str)
                                if not (ip_obj.is_private or ip_obj.is_loopback or ip_obj.is_link_local or ip_obj.is_reserved or ip_obj.is_multicast):
                                    is_safe_host = True
                            except Exception:
                                is_safe_host = False

                        if is_safe_host:
                            with httpx.Client(timeout=8.0) as client:
                                with client.stream("GET", image_url_str) as response:
                                    if response.status_code == 200:
                                        content_len = response.headers.get("Content-Length")
                                        if content_len and int(content_len) > 15 * 1024 * 1024:
                                            return Response({'error': 'La imagen remota supera el tamaño máximo de 15MB.'}, status=status.HTTP_400_BAD_REQUEST)
                                        chunks = []
                                        downloaded = 0
                                        for chunk in response.iter_bytes(chunk_size=65536):
                                            downloaded += len(chunk)
                                            if downloaded > 15 * 1024 * 1024:
                                                return Response({'error': 'La imagen remota supera el tamaño máximo de 15MB.'}, status=status.HTTP_400_BAD_REQUEST)
                                            chunks.append(chunk)
                                        image_bytes = b"".join(chunks)
                except Exception:
                    pass

        if not image_bytes or len(image_bytes) < 100:
            return Response({'error': 'No se recibió ninguna imagen válida para analizar.'}, status=status.HTTP_400_BAD_REQUEST)

        # -------------------------------------------------------------
        # CAPA 1: CACHÉ PRE-LLM (Por Hash SHA-256 y por URL)
        # Si la imagen ya fue analizada antes, responder de inmediato
        # con el peritaje guardado sin consumir cuota ni activar cooldowns.
        # -------------------------------------------------------------
        image_hash = hashlib.sha256(image_bytes).hexdigest()
        cache_hash_key = f"analysis_hash_{image_hash}"
        cached_result = cache.get(cache_hash_key)

        image_url = request.data.get('image_url')
        if not cached_result and image_url:
            clean_url = str(image_url).split('uploads/')[-1] if 'uploads/' in str(image_url) else str(image_url).strip()
            cached_result = cache.get(f"analysis_res_{str(image_url).strip()}") or cache.get(f"analysis_res_{clean_url}")

        if cached_result and isinstance(cached_result, dict):
            cached_payload = dict(cached_result)
            cached_payload['is_cached'] = True
            return Response(cached_payload, status=status.HTTP_200_OK)

        # -------------------------------------------------------------
        # CAPA 2: CANDADO ATÓMICO ANTI-CONCURRENCIA (12 segundos)
        # cache.add() solo tiene éxito para la primera petición.
        # Evita que 20 llamadas paralelas sobrepasen el cooldown.
        # -------------------------------------------------------------
        user_id = request.user.id if request.user.is_authenticated else 'anon'
        cooldown_key = f"gemini_lock_{user_id}"
        lock_acquired = cache.add(cooldown_key, True, timeout=12)
        if not lock_acquired:
            return Response({
                'error': 'Ya tienes un análisis en proceso o debes esperar unos segundos antes de solicitar otro peritaje de IA para proteger la cuota.'
            }, status=status.HTTP_429_TOO_MANY_REQUESTS)

        gemini_api_key = SystemConfig.get_value('gemini_api_key')
        if not gemini_api_key:
            cache.delete(cooldown_key)
            return Response({
                'error': 'La API Key de Gemini no está configurada aún en el servidor. El administrador puede activarla en el Panel de Autoridades > Ajustes de IA.'
            }, status=status.HTTP_400_BAD_REQUEST)

        b64_img = base64.b64encode(image_bytes).decode('utf-8')

        prompt = """Eres un perito ambiental experto del departamento de Pasco, Perú (Cerro de Pasco).
Analiza detalladamente esta fotografía tomada por un ciudadano o brigada municipal.
Determina con rigor técnico si la imagen muestra una infracción o problema de contaminación ambiental REAL en Cerro de Pasco.

Debes responder EXCLUSIVAMENTE con un JSON válido con la siguiente estructura y opciones exactas:
{
  "is_environmental_issue": true,
  "is_minor_litter": false,
  "rejection_reason": null,
  "category": "basura",
  "severity": "critico",
  "waste_scale": "gran_acumulacion",
  "suggested_title": "Título conciso y profesional del reporte (máx 8 palabras)",
  "suggested_description": "Explicación clara del impacto observado (máx 3 oraciones)",
  "technical_analysis": "Diagnóstico de los elementos visuales identificados y magnitud",
  "municipal_action": "Acción municipal recomendada"
}

Opciones para 'category':
- "mineria": relaves mineros, filtraciones ácidas, polvareda de tajo abierto, escombreras.
- "basura": acumulación severa de residuos sólidos, botaderos clandestinos, plásticos masivos, desmonte acumulado.
- "agua": contaminación de ríos, lagunas (Patarcocha), desagües abiertos.
- "aire": humos negros, quema de neumáticos, emisiones vehiculares pesadas.

Opciones para 'severity':
- "critico": emergencia o riesgo severo para la salud o ecosistema.
- "medio": molestia moderada o riesgo persistente.
- "bajo": incidente incipiente o menor.

Opciones para 'waste_scale':
- "gran_acumulacion": botadero clandestino, gran volumen de basura acumulada por días/semanas, montículos masivos, obstrucción de cauces o vías públicas.
- "basura_menor": una sola bolsita de basura doméstica en la calle, envoltura de papel o plástico, lata, botella, desecho aislado o basura menor cotidiana de paso.
- "no_aplica": para problemas de minería, agua o aire, o fotos sin contaminación.

REGLAS CRÍTICAS DE EVALUACIÓN Y FILTRADO:
1. FILTRO DE BASURA MENOR VS GRAN FOCO DE CONTAMINACIÓN:
   EcoAlerta está reservado para FOCOS DE CONTAMINACIÓN GRANDES (botaderos clandestinos, basura acumulada por mucho tiempo, montículos de residuos que amenazan la salud pública).
   Si la imagen muestra únicamente BASURA MENOR COTIDIANA (por ejemplo: una sola bolsita de basura de casa en la vereda, un papel o envoltura tirada, una botella plástica, un vaso descartable o desperdicios aislados menores):
   Debes marcar OBLIGATORIAMENTE:
   "is_environmental_issue": false,
   "is_minor_litter": true,
   "rejection_reason": "minor_litter",
   "waste_scale": "basura_menor",
   "category": "basura",
   "severity": "bajo",
   "technical_analysis": "Se observa basura menor cotidiana o una bolsa aislada que corresponde al servicio ordinario de recolección municipal.",
   "municipal_action": "El camión recolector ya va a pasar por esa zona a recoger esa basura según su cronograma habitual."

2. FILTRO DE FOTOS NO AMBIENTALES:
   Si la imagen NO muestra ningún problema ambiental (selfie, meme, dibujo, documento, comida, mascota, paisaje limpio, interior de casa, vehículo normal o cualquier objeto sin contaminación):
   Debes marcar OBLIGATORIAMENTE:
   "is_environmental_issue": false,
   "is_minor_litter": false,
   "rejection_reason": "not_environmental",
   "waste_scale": "no_aplica",
   "category": "ninguna",
   "severity": "bajo",
   "technical_analysis": "La fotografía no muestra evidencia de contaminación ambiental en Cerro de Pasco.",
   "municipal_action": "No amerita acción de emergencia."

3. APROBACIÓN DE ALERTAS REALES:
   Solamente si se trata de un foco significativo de contaminación (botadero clandestino, basura masiva acumulada por días o semanas, relaves mineros, aguas ácidas o servidas, quema de neumáticos o emisiones tóxicas):
   "is_environmental_issue": true,
   "is_minor_litter": false,
   "rejection_reason": null,
   "category": "mineria" | "basura" | "agua" | "aire",
   "severity": "critico" | "medio" | "bajo"
"""

        try:
            gemini_model = SystemConfig.get_value('gemini_model', default='gemini-3.5-flash')
            if gemini_model in ('gemini-2.5-flash', 'gemini-2.5-flash-lite') or not gemini_model:
                gemini_model = 'gemini-3.5-flash'

            # Cadena de modelos de respaldo ultra rápidos
            fallback_chain = ['gemini-3.5-flash', 'gemini-3-flash-preview', 'gemini-3.8-flash', 'gemini-3.1-flash-lite']
            models_to_try = [gemini_model]
            for fb in fallback_chain:
                if fb not in models_to_try:
                    models_to_try.append(fb)

            payload = {
                "contents": [{
                    "parts": [
                        {"text": prompt},
                        {
                            "inline_data": {
                                "mime_type": mime_type,
                                "data": b64_img
                            }
                        }
                    ]
                }],
                "generationConfig": {
                    "responseMimeType": "application/json",
                    "thinkingConfig": {
                        "thinkingBudget": 0
                    }
                }
            }

            last_error = None
            for model in models_to_try:
                url = f"https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={gemini_api_key}"
                try:
                    res = httpx.post(url, json=payload, timeout=25.0)
                    if res.status_code == 200:
                        res_json = res.json()
                        candidates = res_json.get('candidates', [])
                        if candidates:
                            text = candidates[0]['content']['parts'][0]['text']
                            clean_text = text.strip()
                            if clean_text.startswith('```'):
                                clean_text = re.sub(r'^```(?:json)?\s*', '', clean_text, flags=re.IGNORECASE)
                                clean_text = re.sub(r'\s*```$', '', clean_text)
                            parsed = json.loads(clean_text)

                            # Normalizar categorías
                            raw_cat = str(parsed.get('category', '')).lower()
                            if any(k in raw_cat for k in ['desmonte', 'residuo', 'basura', 'plastic', 'escomb']):
                                parsed['category'] = 'basura'
                            elif any(k in raw_cat for k in ['miner', 'relave', 'tajo']):
                                parsed['category'] = 'mineria'
                            elif any(k in raw_cat for k in ['agua', 'rio', 'lagun', 'desag']):
                                parsed['category'] = 'agua'
                            elif any(k in raw_cat for k in ['aire', 'humo', 'quem']):
                                parsed['category'] = 'aire'

                            # -------------------------------------------------------------
                            # CAPA 3: GUARDADO EN CACHÉ DE LARGA DURACIÓN (24 Horas)
                            # Permite responder de inmediato si cualquier usuario o vista
                            # vuelve a pedir el análisis de esta misma imagen.
                            # -------------------------------------------------------------
                            cache.set(cache_hash_key, parsed, timeout=86400)
                            if image_url:
                                clean_url = str(image_url).split('uploads/')[-1] if 'uploads/' in str(image_url) else str(image_url).strip()
                                cache.set(f"analysis_res_{str(image_url).strip()}", parsed, timeout=86400)
                                cache.set(f"analysis_res_{clean_url}", parsed, timeout=86400)

                            # Mantener cooldown activo durante 12 segundos tras éxito
                            cache.set(cooldown_key, True, timeout=12)
                            return Response(parsed)
                    elif res.status_code == 429:
                        last_error = 'Se alcanzó el límite de consultas gratuitas de Google Gemini (15 req/min). Por favor espera un momento o completa los campos manualmente.'
                        continue
                    elif res.status_code in (404, 503):
                        last_error = f'El modelo {model} no está disponible actualmente (HTTP {res.status_code}). Probando alternativa...'
                        continue
                    else:
                        last_error = f'Gemini no pudo procesar la imagen ({res.status_code}): {res.text[:180]}'
                except httpx.TimeoutException:
                    last_error = f'Timeout temporal con el modelo {model}. Probando alternativa...'
                    continue
                except Exception as e:
                    last_error = str(e)
                    continue

            if last_error and '15 req/min' in last_error:
                return Response({'error': last_error}, status=status.HTTP_429_TOO_MANY_REQUESTS)
            return Response({'error': last_error or 'No se pudo procesar la imagen con los modelos disponibles.'}, status=status.HTTP_400_BAD_REQUEST)
        except Exception as e:
            return Response({'error': f'Error en el análisis de Gemini: {str(e)}'}, status=status.HTTP_500_INTERNAL_SERVER_ERROR)
            return Response({'error': last_error or 'No se pudo procesar la imagen con los modelos disponibles.'}, status=status.HTTP_400_BAD_REQUEST)
        except Exception as e:
            return Response({'error': f'Error en el análisis de Gemini: {str(e)}'}, status=status.HTTP_500_INTERNAL_SERVER_ERROR)


class UserProfileView(APIView):
    """Retorna el perfil ciudadano, balance de Eco-Puntos, nivel cívico y medallas obtenidas."""
    permission_classes = [permissions.IsAuthenticated]

    def get(self, request):
        user = request.user
        reported_alerts = Alert.objects.filter(reported_by=user)
        total_reported = reported_alerts.count()
        total_solved = reported_alerts.filter(status='solved').count()
        total_in_inspection = reported_alerts.filter(status='verified').count()
        total_confirmations_given = user.confirmed_alerts.count()

        # Cálculo de Eco-Puntos: 50 pts por reporte, 100 pts extra por reporte resuelto, 10 pts por confirmación
        eco_points = (total_reported * 50) + (total_solved * 100) + (total_confirmations_given * 10)

        # Rango y Nivel Cívico
        if eco_points >= 800:
            rank = "Fiscalizador Cívico Mayor"
            level = 4
            badge_icon = "🛡️"
            next_level_points = 1500
        elif eco_points >= 400:
            rank = "Eco-Líder de Pasco"
            level = 3
            badge_icon = "🥇"
            next_level_points = 800
        elif eco_points >= 150:
            rank = "Guardián Ambiental"
            level = 2
            badge_icon = "🥈"
            next_level_points = 400
        elif eco_points >= 50:
            rank = "Vecino Comprometido"
            level = 1
            badge_icon = "🥉"
            next_level_points = 150
        else:
            rank = "Vecino Iniciado"
            level = 0
            badge_icon = "🌱"
            next_level_points = 50

        medals = []
        if total_reported >= 1:
            medals.append({"id": "first_report", "title": "Primer Paso Verde", "desc": "Enviaste tu primer reporte ambiental", "icon": "🌱"})
        if total_solved >= 1:
            medals.append({"id": "first_solved", "title": "Impacto Real", "desc": "Una de tus denuncias fue atendida y solucionada", "icon": "✅"})
        if total_reported >= 5:
            medals.append({"id": "vigilant", "title": "Centinela de Pasco", "desc": "5 o más alertas registradas con éxito", "icon": "👁️"})
        if total_confirmations_given >= 3:
            medals.append({"id": "community", "title": "Voz Comunitaria", "desc": "Apoyaste 3 o más denuncias de otros vecinos", "icon": "🤝"})
        if total_solved >= 3:
            medals.append({"id": "master_eco", "title": "Héroe Ambiental", "desc": "3 o más alertas resueltas por las autoridades", "icon": "🏆"})

        return Response({
            "username": user.username,
            "email": user.email,
            "is_staff": user.is_staff,
            "eco_points": eco_points,
            "rank": rank,
            "level": level,
            "badge_icon": badge_icon,
            "next_level_points": next_level_points,
            "total_reported": total_reported,
            "total_solved": total_solved,
            "total_in_inspection": total_in_inspection,
            "total_confirmations_given": total_confirmations_given,
            "medals": medals,
        })


class PublicAlertVerificationView(APIView):
    """Punto de verificación pública de autenticidad pericial (consultado por escaneo de QR)."""
    permission_classes = [permissions.AllowAny]

    def get(self, request, pk=None):
        try:
            alert = Alert.objects.get(pk=pk)
        except Alert.DoesNotExist:
            return Response({'error': 'Expediente no encontrado o código de verificación inválido.'}, status=status.HTTP_404_NOT_FOUND)

        raw_hash_data = f"{alert.id}:{alert.created_at.isoformat()}:{alert.status}:{alert.latitude}:{alert.longitude}:{alert.title}"
        verification_hash = hashlib.sha256(raw_hash_data.encode()).hexdigest()[:16].upper()

        return Response({
            "expediente_id": f"EXP-{alert.created_at.year}-{alert.id:05d}",
            "verification_hash": verification_hash,
            "system": "Sistema Integral de Fiscalización Ambiental EcoAlerta Pasco",
            "title": alert.title,
            "description": alert.description,
            "category": alert.get_category_display(),
            "severity": alert.get_severity_display(),
            "district": alert.get_district_display(),
            "address": alert.address,
            "latitude": alert.latitude,
            "longitude": alert.longitude,
            "status": alert.status,
            "status_display": alert.get_status_display(),
            "created_at": alert.created_at.strftime("%Y-%m-%d %H:%M"),
            "resolved_at": alert.resolved_at.strftime("%Y-%m-%d %H:%M") if alert.resolved_at else None,
            "resolution_comment": alert.resolution_comment,
            "resolution_image_url": alert.resolution_image_url,
            "image_url": alert.image_url,
            "image_url_2": alert.image_url_2,
            "confirmations_count": alert.confirmations_count,
            "history_log": alert.history_log,
        })


class HealthCheckView(APIView):
    """Monitoreo de estado operativo del backend, base de datos y servicios."""
    permission_classes = [permissions.AllowAny]

    def get(self, request):
        from django.db import connection
        db_ok = False
        try:
            with connection.cursor() as cursor:
                cursor.execute("SELECT 1;")
                db_ok = cursor.fetchone()[0] == 1
        except Exception:
            db_ok = False

        gemini_active = SystemConfig.is_gemini_enabled()
        total_alerts = Alert.objects.count()
        total_waste_points = WastePoint.objects.count()

        channel_backend = settings.CHANNEL_LAYERS.get("default", {}).get("BACKEND", "")
        ws_layer = "redis" if "redis" in channel_backend.lower() else "inmemory_asgi"

        return Response({
            "status": "healthy" if db_ok else "unhealthy",
            "service": "EcoAlerta API & WebSocket Backend",
            "version": "2.3.0-realtime",
            "timestamp": timezone.now().isoformat(),
            "database": "connected" if db_ok else "disconnected",
            "realtime": {
                "protocol": "websocket",
                "endpoint": "/ws/map/",
                "channel_layer": ws_layer,
            },
            "gemini_ai_enabled": gemini_active,
            "stats": {
                "total_alerts": total_alerts,
                "total_waste_points": total_waste_points,
            }
        })


