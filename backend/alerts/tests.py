import os
from django.test import TestCase
from django.conf import settings
from django.contrib.auth import get_user_model
from rest_framework.test import APIClient
from rest_framework import status
from django.core.files.uploadedfile import SimpleUploadedFile
from alerts.models import Alert, WastePoint

User = get_user_model()

class EcoAlertaTests(TestCase):
    def setUp(self):
        self.client = APIClient()
        self.citizen = User.objects.create_user(username='ciudadano1', email='ciudadano1@example.com', password='password123')
        self.staff_user = User.objects.create_user(username='autoridad1', email='autoridad1@example.com', password='password123', is_staff=True)

        self.alert_data = {
            'title': 'Contaminación de Agua en Río San Juan',
            'description': 'Descarga anómala de sedimentos cerca al puente.',
            'category': 'agua',
            'severity': 'critico',
            'district': 'yanacancha',
            'address': 'Carretera Central, Yanacancha',
            'latitude': -10.6420,
            'longitude': -76.2360,
        }

    def test_alert_creation_requires_login(self):
        """Validar que un usuario sin autenticar NO puede crear alertas (401)."""
        response = self.client.post('/api/alerts/', self.alert_data, format='json')
        self.assertEqual(response.status_code, status.HTTP_401_UNAUTHORIZED)
        self.assertEqual(Alert.objects.count(), 0)

    def test_authenticated_citizen_can_create_alert(self):
        """Validar que un ciudadano autenticado sí puede registrar una alerta (201)."""
        self.client.force_authenticate(user=self.citizen)
        response = self.client.post('/api/alerts/', self.alert_data, format='json')
        self.assertEqual(response.status_code, status.HTTP_201_CREATED)
        self.assertEqual(Alert.objects.count(), 1)
        alert = Alert.objects.first()
        self.assertEqual(alert.reported_by, self.citizen)
        self.assertIn('Reportado por ciudadano1', alert.history_log)

    def test_citizen_cannot_resolve_alert(self):
        """Validar que un ciudadano común NO puede resolver una alerta (403)."""
        self.client.force_authenticate(user=self.citizen)
        create_resp = self.client.post('/api/alerts/', self.alert_data, format='json')
        alert_id = create_resp.data['id']

        # Intento de marcarla como resuelta
        patch_resp = self.client.patch(f'/api/alerts/{alert_id}/', {'status': 'solved', 'resolution_comment': 'Resuelto por mi'}, format='json')
        self.assertEqual(patch_resp.status_code, status.HTTP_403_FORBIDDEN)

    def test_staff_authority_can_resolve_alert(self):
        """Validar que el personal autorizado (is_staff=True) sí puede resolver alertas."""
        self.client.force_authenticate(user=self.citizen)
        create_resp = self.client.post('/api/alerts/', self.alert_data, format='json')
        alert_id = create_resp.data['id']

        self.client.force_authenticate(user=self.staff_user)
        patch_resp = self.client.patch(
            f'/api/alerts/{alert_id}/',
            {
                'status': 'solved',
                'resolution_comment': 'Se envió brigada municipal a inspeccionar y limpiar.'
            },
            format='json'
        )
        self.assertEqual(patch_resp.status_code, status.HTTP_200_OK)
        alert = Alert.objects.get(id=alert_id)
        self.assertEqual(alert.status, 'solved')
        self.assertIsNotNone(alert.resolved_at)
        self.assertIn('Solucionado por autoridad1', alert.history_log)

    def test_waste_points_public_listing(self):
        """Validar que los puntos limpios y de acopio son públicos y se consultan correctamente."""
        WastePoint.objects.create(
            name='Contenedor Yanacancha Central',
            description='Punto limpio',
            latitude=-10.6625,
            longitude=-76.2555,
            type='recyclable',
            fill_level='low',
            truck_schedule='Lunes 18:00'
        )
        response = self.client.get('/api/waste-points/')
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertEqual(len(response.data), 1)

    def test_citizen_registration_and_login(self):
        """Validar el registro con Turnstile (is_staff=False garantizado) y login de autoridades."""
        # Registro ciudadano seguro con test dummy token
        reg_resp = self.client.post('/api/auth/register/', {
            'username': 'nuevo_vecino',
            'email': 'vecino@pasco.gob.pe',
            'password': 'claveSegura123!',
            'turnstile_token': 'dummy_test_token'
        }, format='json')
        self.assertEqual(reg_resp.status_code, status.HTTP_201_CREATED)
        self.assertIn('token', reg_resp.data)
        self.assertFalse(reg_resp.data['is_staff'])

        # Login para autoridades / personal registrado sí debe funcionar
        login_resp = self.client.post('/api/auth/login/', {
            'username': 'autoridad1',
            'password': 'password123'
        }, format='json')
        self.assertEqual(login_resp.status_code, status.HTTP_200_OK)
        self.assertIn('token', login_resp.data)
        self.assertTrue(login_resp.data['is_staff'])

    def test_image_upload_extension_validation(self):
        """Validar rechazo de archivos que no son imágenes válidas."""
        self.client.force_authenticate(user=self.citizen)
        invalid_file = SimpleUploadedFile("script.sh", b"echo hello", content_type="text/x-sh")
        response = self.client.post('/api/upload/', {'image': invalid_file}, format='multipart')
        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('Formato no permitido', response.data['error'])

    def test_image_upload_compresses_and_converts_to_webp(self):
        """Validar que una imagen JPG es convertida y comprimida automáticamente a WebP."""
        self.client.force_authenticate(user=self.citizen)
        import io
        from PIL import Image
        img = Image.new('RGB', (800, 600), color=(100, 150, 200))
        buf = io.BytesIO()
        img.save(buf, format='JPEG')
        jpeg_bytes = buf.getvalue()

        test_file = SimpleUploadedFile("basura_pasco.jpg", jpeg_bytes, content_type="image/jpeg")
        response = self.client.post('/api/upload/', {'image': test_file}, format='multipart')
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertIn('url', response.data)
        self.assertTrue(response.data['url'].endswith('.webp'))
        self.assertEqual(response.data['format'], 'webp')
        self.assertIn('compressed_size_kb', response.data)

    def test_anti_spam_cooldown_rejects_immediate_second_alert(self):
        """Validar que un usuario no puede enviar una segunda alerta antes de 60s."""
        self.client.force_authenticate(user=self.citizen)
        resp1 = self.client.post('/api/alerts/', self.alert_data, format='json')
        self.assertEqual(resp1.status_code, status.HTTP_201_CREATED)

        # Intento inmediato de crear otra alerta en ubicación distinta
        other_alert = self.alert_data.copy()
        other_alert['latitude'] = -10.6800
        other_alert['longitude'] = -76.2800
        other_alert['title'] = 'Otra alerta diferente'

        resp2 = self.client.post('/api/alerts/', other_alert, format='json')
        self.assertEqual(resp2.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('Debes esperar', str(resp2.data))

    def test_anti_spam_duplicate_proximity_rejects_same_category_within_50m(self):
        """Validar que otro usuario no puede crear una alerta duplicada en radio de 50m."""
        self.client.force_authenticate(user=self.citizen)
        resp1 = self.client.post('/api/alerts/', self.alert_data, format='json')
        self.assertEqual(resp1.status_code, status.HTTP_201_CREATED)

        # Segundo ciudadano intenta reportar lo mismo a 10 metros
        other_citizen = User.objects.create_user(username='vecino2', email='v2@example.com', password='pass')
        self.client.force_authenticate(user=other_citizen)

        near_alert = self.alert_data.copy()
        near_alert['title'] = 'Derrame similar reportado por otro vecino'
        # Aproximadamente 10 metros de diferencia (~0.00009 grados)
        near_alert['latitude'] = self.alert_data['latitude'] + 0.00009

        resp2 = self.client.post('/api/alerts/', near_alert, format='json')
        self.assertEqual(resp2.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('Ya existe una alerta activa', str(resp2.data))

    def test_community_confirmation_upvote(self):
        """Validar que ciudadanos pueden confirmar/apoyar alertas existentes (+1)."""
        self.client.force_authenticate(user=self.citizen)
        create_resp = self.client.post('/api/alerts/', self.alert_data, format='json')
        alert_id = create_resp.data['id']

        other_citizen = User.objects.create_user(username='apoyante1', email='apoyo@example.com', password='pass')
        self.client.force_authenticate(user=other_citizen)

        # Confirmar
        conf_resp = self.client.post(f'/api/alerts/{alert_id}/confirm/')
        self.assertEqual(conf_resp.status_code, status.HTTP_200_OK)
        self.assertTrue(conf_resp.data['confirmed'])
        self.assertEqual(conf_resp.data['confirmations_count'], 1)

        # Desconfirmar
        unconf_resp = self.client.post(f'/api/alerts/{alert_id}/confirm/')
        self.assertEqual(unconf_resp.status_code, status.HTTP_200_OK)
        self.assertFalse(unconf_resp.data['confirmed'])
        self.assertEqual(unconf_resp.data['confirmations_count'], 0)

    def test_export_alerts_csv(self):
        """Validar la descarga de reportes en formato CSV."""
        self.client.force_authenticate(user=self.citizen)
        self.client.post('/api/alerts/', self.alert_data, format='json')

        resp = self.client.get('/api/alerts/export_csv/')
        self.assertEqual(resp.status_code, status.HTTP_200_OK)
        self.assertEqual(resp['Content-Type'], 'text/csv; charset=utf-8')
        content = resp.content.decode('utf-8')
        self.assertIn('Título', content)
        self.assertIn('Contaminación de Agua en Río San Juan', content)

    def test_system_settings_gemini_key_permissions(self):
        """Validar que solo el administrador puede configurar la API Key de Gemini."""
        # Usuario común no puede guardar
        self.client.force_authenticate(user=self.citizen)
        fail_resp = self.client.post('/api/system/settings/', {'gemini_api_key': 'AIzaFakeKey123'}, format='json')
        self.assertEqual(fail_resp.status_code, status.HTTP_403_FORBIDDEN)

        # Staff sí puede guardar
        self.client.force_authenticate(user=self.staff_user)
        ok_resp = self.client.post('/api/system/settings/', {'gemini_api_key': 'AIzaFakeKey123'}, format='json')
        self.assertEqual(ok_resp.status_code, status.HTTP_200_OK)
        self.assertTrue(ok_resp.data['has_gemini_key'])
        self.assertEqual(ok_resp.data['masked_key'], 'AIza...y123')

    def test_ai_analysis_requires_authentication(self):
        """Validar que usuarios anónimos no pueden invocar el analizador de IA."""
        self.client.force_authenticate(user=None)
        resp = self.client.post('/api/ai/analyze-evidence/', {'image_url': 'http://example.com/test.jpg'}, format='json')
        self.assertEqual(resp.status_code, status.HTTP_401_UNAUTHORIZED)

    def test_ai_analysis_pre_cache_prevents_gemini_reinvocation(self):
        """Validar que si una imagen ya está analizada en caché, se responde con is_cached sin llamar a la API."""
        from django.core.cache import cache
        self.client.force_authenticate(user=self.citizen)

        fake_analysis = {
            'is_environmental_issue': True,
            'is_minor_litter': False,
            'category': 'basura',
            'severity': 'critico',
            'suggested_title': 'Botadero Clandestino Detectado',
            'suggested_description': 'Acumulación masiva de residuos sólidos.',
            'technical_analysis': 'Montículo de basura de gran volumen.',
            'municipal_action': 'Envío de maquinaria pesada.'
        }
        test_url = '/media/uploads/test_evidencia_cached.webp'
        clean_name = 'test_evidencia_cached.webp'

        # Simular que el archivo local existe o está registrado en caché
        cache.set(f"analysis_res_{test_url}", fake_analysis, timeout=3600)
        cache.set(f"analysis_res_{clean_name}", fake_analysis, timeout=3600)

        # Crear dummy en disco para que pase la verificación de archivo local
        os.makedirs(os.path.join(settings.MEDIA_ROOT, 'uploads'), exist_ok=True)
        local_path = os.path.join(settings.MEDIA_ROOT, 'uploads', clean_name)
        with open(local_path, 'wb') as f:
            f.write(b"RIFF" + b"X" * 500)

        try:
            resp = self.client.post('/api/ai/analyze-evidence/', {'image_url': test_url}, format='json')
            self.assertEqual(resp.status_code, status.HTTP_200_OK)
            self.assertTrue(resp.data.get('is_cached'))
            self.assertEqual(resp.data['category'], 'basura')
            self.assertEqual(resp.data['suggested_title'], 'Botadero Clandestino Detectado')
        finally:
            if os.path.exists(local_path):
                os.remove(local_path)

    def test_ai_analysis_concurrency_lock_rejects_simultaneous_requests(self):
        """Validar que el candado atómico anti-concurrencia rechaza peticiones simultáneas del mismo usuario (HTTP 429)."""
        from django.core.cache import cache
        self.client.force_authenticate(user=self.citizen)

        # Establecer el candado activo como si una petición estuviese en curso
        lock_key = f"gemini_lock_{self.citizen.id}"
        cache.set(lock_key, True, timeout=12)

        # Crear archivo temporal
        clean_name = 'test_lock_image.webp'
        os.makedirs(os.path.join(settings.MEDIA_ROOT, 'uploads'), exist_ok=True)
        local_path = os.path.join(settings.MEDIA_ROOT, 'uploads', clean_name)
        with open(local_path, 'wb') as f:
            f.write(b"RIFF" + b"Y" * 500)

        try:
            resp = self.client.post('/api/ai/analyze-evidence/', {'image_url': f'/media/uploads/{clean_name}'}, format='json')
            self.assertEqual(resp.status_code, status.HTTP_429_TOO_MANY_REQUESTS)
            self.assertIn('Ya tienes un análisis en proceso', resp.data['error'])
        finally:
            cache.delete(lock_key)
            if os.path.exists(local_path):
                os.remove(local_path)

    def test_citizen_cannot_mass_assign_status_or_resolution(self):
        """Validar que un ciudadano no puede inyectar status 'solved' ni resolución al crear una alerta."""
        self.client.force_authenticate(user=self.citizen)
        tampered_data = self.alert_data.copy()
        tampered_data['status'] = 'solved'
        tampered_data['resolution_comment'] = 'Resuelto fraudulentamente'
        tampered_data['resolution_image_url'] = 'http://example.com/fake.jpg'

        resp = self.client.post('/api/alerts/', tampered_data, format='json')
        self.assertEqual(resp.status_code, status.HTTP_201_CREATED)

        created_alert = Alert.objects.get(id=resp.data['id'])
        self.assertEqual(created_alert.status, 'pending')
        self.assertIsNone(created_alert.resolution_comment)
        self.assertIsNone(created_alert.resolution_image_url)
        self.assertIsNone(created_alert.resolved_at)

    def test_citizen_cannot_self_confirm_own_alert(self):
        """Validar que un usuario no puede inflar con (+1) su propio reporte ambiental."""
        self.client.force_authenticate(user=self.citizen)
        create_resp = self.client.post('/api/alerts/', self.alert_data, format='json')
        self.assertEqual(create_resp.status_code, status.HTTP_201_CREATED)
        alert_id = create_resp.data['id']

        # El mismo ciudadano intenta respaldarse a sí mismo
        conf_resp = self.client.post(f'/api/alerts/{alert_id}/confirm/')
        self.assertEqual(conf_resp.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('No puedes respaldar tu propia denuncia', str(conf_resp.data))

    def test_logout_invalidates_token(self):
        """Validar que el cierre de sesión elimina e invalida el token en el servidor."""
        from rest_framework.authtoken.models import Token
        token, _ = Token.objects.get_or_create(user=self.citizen)
        self.client.credentials(HTTP_AUTHORIZATION=f'Token {token.key}')

        # Cerrar sesión en el backend
        logout_resp = self.client.post('/api/auth/logout/')
        self.assertEqual(logout_resp.status_code, status.HTTP_200_OK)
        self.assertFalse(Token.objects.filter(key=token.key).exists())

        # Petición posterior con el mismo token debe ser rechazada (401)
        followup_resp = self.client.get('/api/auth/profile/')
        self.assertEqual(followup_resp.status_code, status.HTTP_401_UNAUTHORIZED)




