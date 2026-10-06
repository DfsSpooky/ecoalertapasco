from rest_framework import viewsets, status, permissions
from rest_framework.decorators import action
from rest_framework.views import APIView
from rest_framework.response import Response
from rest_framework.parsers import MultiPartParser, FormParser
from rest_framework.authtoken.models import Token
from rest_framework.throttling import UserRateThrottle
from rest_framework.exceptions import ValidationError, PermissionDenied
from django.contrib.auth import authenticate, get_user_model
from django.utils import timezone
from django.conf import settings
from django.core.files.storage import default_storage
from django.core.files.base import ContentFile
import os
import math
import io
import re
from PIL import Image, ImageOps

from .models import Alert, WastePoint, SystemConfig
from .serializers import AlertSerializer, WastePointSerializer

User = get_user_model()


class AlertCreateRateThrottle(UserRateThrottle):
    scope = 'alert_create'


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
    queryset = Alert.objects.all().order_by('-created_at')
    serializer_class = AlertSerializer

    def get_permissions(self):
        # Es OBLIGATORIO haber iniciado sesión para crear o modificar alertas
        if self.action in ['create', 'partial_update', 'update', 'destroy']:
            return [permissions.IsAuthenticated()]
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

        user_str = user.username if user.is_authenticated else "Ciudadano"
        now_str = timezone.now().strftime("%Y-%m-%d %H:%M")
        history_log = f"{now_str}|Reportado por {user_str}"
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

        # Guardar cambios
        new_instance = serializer.save()
        
        # Sincronizar fecha de resolución
        if new_instance.status == 'solved' and not new_instance.resolved_at:
            new_instance.resolved_at = timezone.now()
            new_instance.save()
        elif new_instance.status != 'solved' and new_instance.resolved_at:
            new_instance.resolved_at = None
            new_instance.save()

        # Bitácora
        log_entries = []
        user_str = self.request.user.username if self.request.user.is_authenticated else "Autoridad"
        now_str = timezone.now().strftime("%Y-%m-%d %H:%M")

        if old_status != new_instance.status:
            if new_instance.status == 'solved':
                log_entries.append(f"{now_str}|Solucionado por {user_str}. Evidencia: {new_instance.resolution_comment or 'Conforme'}")
            elif new_instance.status == 'dismissed':
                log_entries.append(f"{now_str}|Descartado por {user_str}. Motivo: {new_instance.resolution_comment or 'No aplicable'}")
            else:
                log_entries.append(f"{now_str}|Estado actualizado a {new_instance.get_status_display()} por {user_str}")

        if old_district != new_instance.district:
            log_entries.append(f"{now_str}|Transferido de {old_district} a {new_instance.district} por {user_str}")

        if log_entries:
            current_log = new_instance.history_log or ""
            new_log = current_log + "\n" + "\n".join(log_entries) if current_log else "\n".join(log_entries)
            new_instance.history_log = new_log
            new_instance.save()

    @action(detail=True, methods=['post'], permission_classes=[permissions.IsAuthenticated])
    def confirm(self, request, pk=None):
        """Permite a ciudadanos confirmar o apoyar un reporte existente (+1)."""
        alert = self.get_object()
        user = request.user
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
        if self.action in ['create', 'update', 'partial_update', 'destroy']:
            return [permissions.IsAdminUser()]
        return [permissions.AllowAny()]


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
            token, _ = Token.objects.get_or_create(user=user)
            return Response({
                'token': token.key,
                'username': user.username,
                'email': user.email,
                'is_staff': user.is_staff
            })
        return Response({'error': 'Credenciales inválidas. Verifica tu usuario y contraseña.'}, status=status.HTTP_401_UNAUTHORIZED)


class RegisterView(APIView):
    authentication_classes = []
    permission_classes = []

    def post(self, request):
        username = request.data.get('username')
        email = request.data.get('email')
        password = request.data.get('password')

        if not username or not email or not password:
            return Response({'error': 'Por favor completa todos los campos obligatorios.'}, status=status.HTTP_400_BAD_REQUEST)

        if User.objects.filter(username=username).exists():
            return Response({'error': 'El nombre de usuario ya está registrado.'}, status=status.HTTP_400_BAD_REQUEST)

        if User.objects.filter(email=email).exists():
            return Response({'error': 'El correo electrónico ya está en uso.'}, status=status.HTTP_400_BAD_REQUEST)

        try:
            user = User.objects.create_user(username=username, email=email, password=password)
            token, _ = Token.objects.get_or_create(user=user)
            return Response({
                'token': token.key,
                'username': user.username,
                'email': user.email,
                'is_staff': user.is_staff
            }, status=status.HTTP_201_CREATED)
        except Exception as e:
            return Response({'error': str(e)}, status=status.HTTP_400_BAD_REQUEST)


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
    authentication_classes = []
    permission_classes = []
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
    """Permite consultar y guardar la API Key y el modelo de Gemini desde el panel."""
    def get_permissions(self):
        if self.request.method == 'GET':
            return [permissions.AllowAny()]
        return [permissions.IsAdminUser()]

    def get(self, request):
        key = SystemConfig.get_value('gemini_api_key')
        has_key = bool(key)
        masked = ''
        if has_key:
            masked = f"{key[:4]}...{key[-4:]}" if len(key) > 8 else "********"
        current_model = SystemConfig.get_value('gemini_model', default='gemini-1.5-flash')
        gemini_enabled = SystemConfig.is_gemini_enabled()
        return Response({
            'has_gemini_key': has_key,
            'masked_key': masked,
            'selected_model': current_model,
            'available_models': AVAILABLE_GEMINI_MODELS,
            'gemini_enabled': gemini_enabled,
        })

    def post(self, request):
        gemini_api_key = request.data.get('gemini_api_key', '').strip()
        gemini_model = request.data.get('gemini_model', '').strip()
        has_gemini_enabled = 'gemini_enabled' in request.data

        if not gemini_api_key and not gemini_model and not has_gemini_enabled:
            return Response({'error': 'Debes enviar al menos la clave de API, el modelo o el estado de activación a configurar.'}, status=status.HTTP_400_BAD_REQUEST)

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

        current_key = SystemConfig.get_value('gemini_api_key')
        current_model = SystemConfig.get_value('gemini_model', default='gemini-1.5-flash')
        gemini_enabled = SystemConfig.is_gemini_enabled()
        return Response({
            'success': True,
            'message': 'Configuración de Gemini actualizada correctamente.',
            'has_gemini_key': bool(current_key),
            'masked_key': f"{current_key[:4]}...{current_key[-4:]}" if len(current_key) > 8 else ("********" if current_key else ""),
            'selected_model': current_model,
            'available_models': AVAILABLE_GEMINI_MODELS,
            'gemini_enabled': gemini_enabled,
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
    """Analiza una foto de evidencia ambiental usando Gemini Multimodal."""
    permission_classes = [permissions.IsAuthenticated]

    def post(self, request):
        if not SystemConfig.is_gemini_enabled():
            return Response({
                'error': 'El análisis inteligente con Gemini AI está desactivado actualmente por la administración. Puedes completar tu reporte de forma manual.',
                'is_gemini_enabled': False
            }, status=status.HTTP_400_BAD_REQUEST)

        gemini_api_key = SystemConfig.get_value('gemini_api_key')
        if not gemini_api_key:
            return Response({
                'error': 'La API Key de Gemini no está configurada aún en el servidor. El administrador puede activarla en el Panel de Autoridades > Ajustes de IA.'
            }, status=status.HTTP_400_BAD_REQUEST)

        # Protección anti-abuso de cuota gratuita de Gemini (cooldown de 3 segundos por usuario)
        from django.core.cache import cache
        user_id = request.user.id if request.user.is_authenticated else 'anon'
        cooldown_key = f"gemini_analysis_cooldown_{user_id}"
        if cache.get(cooldown_key):
            return Response({
                'error': 'Por favor espera unos segundos antes de analizar otra imagen para proteger la cuota de la IA.'
            }, status=status.HTTP_429_TOO_MANY_REQUESTS)

        import base64
        import httpx
        import json
        import re

        image_bytes = None
        mime_type = 'image/jpeg'

        file_obj = request.FILES.get('image')
        if file_obj:
            try:
                image_bytes, mime_type = compress_and_convert_to_webp(file_obj)
            except Exception:
                image_bytes = file_obj.read()
                mime_type = 'image/jpeg'
        else:
            image_url = request.data.get('image_url')
            if image_url:
                try:
                    url_lower = image_url.lower()
                    if url_lower.endswith('.webp'):
                        mime_type = 'image/webp'
                    elif url_lower.endswith('.png'):
                        mime_type = 'image/png'
                    elif url_lower.endswith('.jpg') or url_lower.endswith('.jpeg'):
                        mime_type = 'image/jpeg'

                    # Si es archivo guardado localmente en MEDIA_ROOT
                    if 'uploads/' in image_url:
                        clean_path = image_url.split('uploads/')[-1]
                        local_path = os.path.join(settings.MEDIA_ROOT, 'uploads', clean_path)
                        if os.path.exists(local_path):
                            with open(local_path, 'rb') as f:
                                image_bytes = f.read()
                    if image_bytes is None and image_url.startswith('http'):
                        r = httpx.get(image_url, timeout=10.0)
                        if r.status_code == 200:
                            image_bytes = r.content
                except Exception:
                    pass

        if not image_bytes:
            return Response({'error': 'No se recibió ninguna imagen válida para analizar.'}, status=status.HTTP_400_BAD_REQUEST)

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

                            # Activar cooldown de 3s
                            cache.set(cooldown_key, True, timeout=3)
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

