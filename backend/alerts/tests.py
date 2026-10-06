from django.test import TestCase
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
        """Validar el registro e inicio de sesión de un nuevo ciudadano."""
        # Registro
        reg_resp = self.client.post('/api/auth/register/', {
            'username': 'nuevo_vecino',
            'email': 'vecino@pasco.gob.pe',
            'password': 'claveSegura123!'
        }, format='json')
        self.assertEqual(reg_resp.status_code, status.HTTP_201_CREATED)
        self.assertIn('token', reg_resp.data)

        # Login
        login_resp = self.client.post('/api/auth/login/', {
            'username': 'nuevo_vecino',
            'password': 'claveSegura123!'
        }, format='json')
        self.assertEqual(login_resp.status_code, status.HTTP_200_OK)
        self.assertIn('token', login_resp.data)

    def test_image_upload_extension_validation(self):
        """Validar rechazo de archivos que no son imágenes válidas."""
        invalid_file = SimpleUploadedFile("script.sh", b"echo hello", content_type="text/x-sh")
        response = self.client.post('/api/upload/', {'image': invalid_file}, format='multipart')
        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('Formato no permitido', response.data['error'])

    def test_image_upload_compresses_and_converts_to_webp(self):
        """Validar que una imagen JPG es convertida y comprimida automáticamente a WebP."""
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


