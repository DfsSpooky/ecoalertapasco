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
            'reporter_latitude': -10.6420,
            'reporter_longitude': -76.2360,
            'image_url': 'https://example.com/evidencia_rio.jpg',
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






class CollectionRouteFleetTests(TestCase):
    """Fase 1 del seguimiento de camiones: rutas, flota y permisos."""

    def setUp(self):
        from django.contrib.auth.models import Group
        from alerts.models import COLLECTOR_GROUP_NAME, Truck

        self.client = APIClient()
        self.citizen = User.objects.create_user(username='vecino', password='password123')
        self.staff_user = User.objects.create_user(username='muni', password='password123', is_staff=True)
        self.driver = User.objects.create_user(username='chofer', password='password123')
        self.driver.groups.add(Group.objects.get(name=COLLECTOR_GROUP_NAME))
        self.truck = Truck.objects.create(code='R-01', plate='ABC-123', driver=self.driver)
        self.route_payload = {
            'name': 'Ruta Prueba',
            'district': 'yanacancha',
            'waste_type': 'general',
            'color': '#FF0000',
            'schedule': 'Lunes 06:00',
            'path': [[-10.6625, -76.2555], [-10.6640, -76.2530]],
            'stops': [
                {'name': 'Inicio', 'latitude': -10.6625, 'longitude': -76.2555, 'eta_offset_minutes': 0},
                {'name': 'Fin', 'latitude': -10.6640, 'longitude': -76.2530, 'eta_offset_minutes': 10},
            ],
        }

    def test_collector_group_created_by_migration(self):
        from django.contrib.auth.models import Group
        self.assertTrue(Group.objects.filter(name='recolectores').exists())

    def test_citizen_cannot_create_route(self):
        self.client.force_authenticate(user=self.citizen)
        res = self.client.post('/api/routes/', self.route_payload, format='json')
        self.assertEqual(res.status_code, status.HTTP_403_FORBIDDEN)

    def test_staff_creates_route_with_stops(self):
        from alerts.models import CollectionRoute
        self.client.force_authenticate(user=self.staff_user)
        res = self.client.post('/api/routes/', self.route_payload, format='json')
        self.assertEqual(res.status_code, status.HTTP_201_CREATED, res.data)
        route = CollectionRoute.objects.get()
        self.assertEqual(route.color, '#ff0000')
        self.assertEqual(list(route.stops.values_list('order', 'name')), [(0, 'Inicio'), (1, 'Fin')])

    def test_route_update_replaces_stops_and_patch_keeps_them(self):
        self.client.force_authenticate(user=self.staff_user)
        rid = self.client.post('/api/routes/', self.route_payload, format='json').data['id']
        res = self.client.patch(f'/api/routes/{rid}/', {'schedule': 'Martes'}, format='json')
        self.assertEqual(len(res.data['stops']), 2)
        payload = dict(self.route_payload, stops=[{'name': 'Única', 'latitude': -10.66, 'longitude': -76.25}])
        res = self.client.put(f'/api/routes/{rid}/', payload, format='json')
        self.assertEqual(res.status_code, status.HTTP_200_OK, res.data)
        self.assertEqual([s['name'] for s in res.data['stops']], ['Única'])

    def test_route_path_validation(self):
        self.client.force_authenticate(user=self.staff_user)
        for bad_path in ([[-10.66, -76.25]], [[-10.66, 'x'], [1, 2]], [[100, 0], [0, 0]], 'nope'):
            res = self.client.post('/api/routes/', dict(self.route_payload, path=bad_path), format='json')
            self.assertEqual(res.status_code, status.HTTP_400_BAD_REQUEST, bad_path)
        res = self.client.post('/api/routes/', dict(self.route_payload, color='green'), format='json')
        self.assertEqual(res.status_code, status.HTTP_400_BAD_REQUEST)

    def test_public_sees_only_published_routes(self):
        from alerts.models import CollectionRoute
        CollectionRoute.objects.create(name='Publicada', path=[[0, 0], [1, 1]])
        CollectionRoute.objects.create(name='Borrador', path=[[0, 0], [1, 1]], is_active=False)
        res = self.client.get('/api/routes/')
        self.assertEqual([r['name'] for r in res.data], ['Publicada'])
        self.client.force_authenticate(user=self.staff_user)
        self.assertEqual(len(self.client.get('/api/routes/').data), 2)

    def test_sync_includes_routes_and_etag_changes(self):
        from alerts.models import CollectionRoute
        first = self.client.get('/api/sync/')
        self.assertEqual(first.data['routes'], [])
        CollectionRoute.objects.create(name='Nueva', path=[[0, 0], [1, 1]])
        second = self.client.get('/api/sync/', {'etag': first.data['etag']})
        self.assertEqual(second.status_code, status.HTTP_200_OK)
        self.assertEqual(len(second.data['routes']), 1)

    def test_truck_privacy_for_citizens(self):
        res = self.client.get('/api/trucks/')
        self.assertEqual(res.status_code, status.HTTP_200_OK)
        self.assertEqual(set(res.data[0].keys()), {'id', 'code', 'waste_type', 'is_active'})
        self.client.force_authenticate(user=self.staff_user)
        res = self.client.get('/api/trucks/')
        self.assertEqual(res.data[0]['plate'], 'ABC-123')
        self.assertEqual(res.data[0]['driver_username'], 'chofer')

    def test_driver_sees_only_assigned_trucks_and_cannot_edit(self):
        from alerts.models import Truck
        Truck.objects.create(code='R-02', plate='XYZ-999')
        self.client.force_authenticate(user=self.driver)
        res = self.client.get('/api/trucks/')
        self.assertEqual([t['code'] for t in res.data], ['R-01'])
        res = self.client.patch(f'/api/trucks/{self.truck.id}/', {'plate': 'HACK'}, format='json')
        self.assertEqual(res.status_code, status.HTTP_403_FORBIDDEN)

    def test_runs_visibility(self):
        from alerts.models import CollectionRoute, RouteRun
        route = CollectionRoute.objects.create(name='R', path=[[0, 0], [1, 1]])
        RouteRun.objects.create(route=route, truck=self.truck, driver=self.driver)
        self.assertEqual(self.client.get('/api/runs/').status_code, status.HTTP_401_UNAUTHORIZED)
        self.client.force_authenticate(user=self.citizen)
        self.assertEqual(self.client.get('/api/runs/').data, [])
        self.client.force_authenticate(user=self.driver)
        self.assertEqual(len(self.client.get('/api/runs/').data), 1)

    def test_one_open_run_per_truck(self):
        from django.db import IntegrityError, transaction
        from alerts.models import CollectionRoute, RouteRun
        route = CollectionRoute.objects.create(name='R', path=[[0, 0], [1, 1]])
        RouteRun.objects.create(route=route, truck=self.truck)
        with self.assertRaises(IntegrityError), transaction.atomic():
            RouteRun.objects.create(route=route, truck=self.truck)

    def test_seed_command_creates_initial_route_once(self):
        from django.core.management import call_command
        from alerts.models import CollectionRoute
        call_command('seed_collection_routes', '--if-empty', verbosity=0)
        call_command('seed_collection_routes', '--if-empty', verbosity=0)
        self.assertEqual(CollectionRoute.objects.count(), 1)
        self.assertEqual(CollectionRoute.objects.get().stops.count(), 5)


class RouteBroadcastTests(TestCase):
    """Las rutas se difunden por WebSocket al publicarse, editarse o retirarse."""

    def _capture(self):
        from unittest import mock
        return mock.patch('alerts.signals._send_group_event')

    def test_broadcast_on_save_and_unpublish(self):
        from alerts.models import CollectionRoute
        with self._capture() as sent, self.captureOnCommitCallbacks(execute=True):
            route = CollectionRoute.objects.create(name='Viva', path=[[0, 0], [1, 1]])
        self.assertEqual(sent.call_args[0][0]['type'], 'map.route.upserted')

        with self._capture() as sent, self.captureOnCommitCallbacks(execute=True):
            route.is_active = False
            route.save()
        self.assertEqual(sent.call_args[0][0], {'type': 'map.route.deleted', 'id': str(route.pk)})


class DriverTrackingTests(TestCase):
    """Fase 2: Modo Recolector, ciclo de vida del turno e ingesta de GPS."""

    def setUp(self):
        import time
        from django.contrib.auth.models import Group
        from alerts.models import COLLECTOR_GROUP_NAME, CollectionRoute, Truck

        self.client = APIClient()
        group = Group.objects.get(name=COLLECTOR_GROUP_NAME)
        self.driver = User.objects.create_user(username='chofer1', password='x')
        self.driver.groups.add(group)
        self.other_driver = User.objects.create_user(username='chofer2', password='x')
        self.other_driver.groups.add(group)
        self.citizen = User.objects.create_user(username='vecina', password='x')
        self.staff_user = User.objects.create_user(username='jefe', password='x', is_staff=True)
        self.truck = Truck.objects.create(code='R-01', plate='AAA-111', driver=self.driver)
        self.route = CollectionRoute.objects.create(name='Centro', path=[[-10.66, -76.25], [-10.67, -76.26]])
        self.now_ms = int(time.time() * 1000)

    def _start(self, user=None):
        self.client.force_authenticate(user=user or self.driver)
        return self.client.post('/api/runs/start/', {'truck': self.truck.id, 'route': self.route.id}, format='json')

    def _point(self, offset_s, lat=-10.6600, lng=-76.2500, accuracy=10):
        return {'lat': lat, 'lng': lng, 'accuracy': accuracy, 'speed': 5, 'heading': 90, 'ts': self.now_ms + offset_s * 1000}

    def test_login_reports_collector_role(self):
        res = self.client.post('/api/auth/login/', {'username': 'chofer1', 'password': 'x'}, format='json')
        self.assertTrue(res.data['is_collector'])
        res = self.client.post('/api/auth/login/', {'username': 'vecina', 'password': 'x'}, format='json')
        self.assertFalse(res.data['is_collector'])

    def test_driver_context(self):
        self.client.force_authenticate(user=self.driver)
        res = self.client.get('/api/runs/driver-context/')
        self.assertEqual(res.status_code, status.HTTP_200_OK)
        self.assertEqual([t['code'] for t in res.data['trucks']], ['R-01'])
        self.assertEqual(len(res.data['routes']), 1)
        self.assertIsNone(res.data['open_run'])
        self.client.force_authenticate(user=self.citizen)
        self.assertEqual(self.client.get('/api/runs/driver-context/').status_code, status.HTTP_403_FORBIDDEN)

    def test_start_requires_assigned_truck(self):
        self.assertEqual(self._start(self.citizen).status_code, status.HTTP_403_FORBIDDEN)
        self.assertEqual(self._start(self.other_driver).status_code, status.HTTP_403_FORBIDDEN)
        self.assertEqual(self._start().status_code, status.HTTP_201_CREATED)

    def test_start_is_idempotent_and_blocks_double_runs(self):
        first = self._start()
        again = self._start()
        self.assertEqual(again.status_code, status.HTTP_200_OK)
        self.assertEqual(again.data['id'], first.data['id'])
        # Otro conductor (vía autoridad) no puede abrir un segundo turno con el mismo camión
        self.assertEqual(self._start(self.staff_user).status_code, status.HTTP_409_CONFLICT)

    def test_lifecycle_transitions(self):
        run_id = self._start().data['id']
        self.assertEqual(self.client.post(f'/api/runs/{run_id}/resume/').status_code, status.HTTP_409_CONFLICT)
        self.assertEqual(self.client.post(f'/api/runs/{run_id}/pause/').data['status'], 'paused')
        self.assertEqual(self.client.post(f'/api/runs/{run_id}/resume/').data['status'], 'active')
        res = self.client.post(f'/api/runs/{run_id}/finish/')
        self.assertEqual(res.data['status'], 'finished')
        self.assertIsNotNone(res.data['ended_at'])
        # Tras finalizar, el camión queda libre para un nuevo turno
        self.assertEqual(self._start().status_code, status.HTTP_201_CREATED)

    def test_other_driver_cannot_touch_run(self):
        run_id = self._start().data['id']
        self.client.force_authenticate(user=self.other_driver)
        self.assertEqual(self.client.post(f'/api/runs/{run_id}/finish/').status_code, status.HTTP_403_FORBIDDEN)
        res = self.client.post(f'/api/runs/{run_id}/positions/', {'points': [self._point(0)]}, format='json')
        self.assertEqual(res.status_code, status.HTTP_403_FORBIDDEN)

    def test_staff_can_finish_any_run(self):
        run_id = self._start().data['id']
        self.client.force_authenticate(user=self.staff_user)
        self.assertEqual(self.client.post(f'/api/runs/{run_id}/finish/').status_code, status.HTTP_200_OK)

    def test_positions_filtering_and_storage(self):
        from alerts.models import RouteRun
        run_id = self._start().data['id']
        points = [
            self._point(0),
            self._point(1, lat=-10.66001),               # < 2 s desde la anterior → descartada
            self._point(5, lat=-10.6601),                # ok (~11 m en 5 s)
            self._point(10, lat=-10.6602, accuracy=500),  # precisión mala → descartada
            self._point(15, lat=-10.70),                 # salto de ~4 km en 10 s → descartado
            self._point(40, lat=-10.6610),               # ok, > 15 s → se guarda en historial
            {'lat': 'x', 'lng': 1, 'ts': self.now_ms},    # inválido
        ]
        res = self.client.post(f'/api/runs/{run_id}/positions/', {'points': points}, format='json')
        self.assertEqual(res.status_code, status.HTTP_200_OK, res.data)
        self.assertEqual(res.data['accepted'], 3)
        self.assertEqual(res.data['rejected'], 4)
        run = RouteRun.objects.get(pk=run_id)
        self.assertAlmostEqual(run.last_latitude, -10.6610)
        self.assertEqual(run.last_speed_kmh, 18.0)
        # Se guarda el primero y el de +40 s; el de +5 s está a < 15 s y < 40 m del anterior
        self.assertEqual(run.positions.count(), 2)

    def test_positions_rejected_when_paused(self):
        run_id = self._start().data['id']
        self.client.post(f'/api/runs/{run_id}/pause/')
        res = self.client.post(f'/api/runs/{run_id}/positions/', {'points': [self._point(0)]}, format='json')
        self.assertEqual(res.status_code, status.HTTP_409_CONFLICT)
        self.assertEqual(res.data['status'], 'paused')

    def test_position_is_broadcast_and_public_payload_hides_driver(self):
        from unittest import mock
        run_id = self._start().data['id']
        with mock.patch('alerts.tracking._send') as sent, self.captureOnCommitCallbacks(execute=True):
            self.client.post(f'/api/runs/{run_id}/positions/', {'points': [self._point(0), self._point(6, lat=-10.6601)]}, format='json')
        self.assertEqual(sent.call_count, 1)  # solo la última posición del lote
        payload = sent.call_args[0][0]
        self.assertEqual(payload['type'], 'map.truck.position')
        self.assertAlmostEqual(payload['position']['latitude'], -10.6601)

        self.client.force_authenticate(user=None)
        runs = self.client.get('/api/sync/').data['truck_runs']
        self.assertEqual(len(runs), 1)
        self.assertEqual(runs[0]['truck_code'], 'R-01')
        self.assertNotIn('driver', runs[0])
        self.assertNotIn('plate', runs[0])

    def test_finished_run_broadcasts_end_and_leaves_snapshot(self):
        from unittest import mock
        run_id = self._start().data['id']
        with mock.patch('alerts.tracking._send') as sent, self.captureOnCommitCallbacks(execute=True):
            self.client.post(f'/api/runs/{run_id}/finish/')
        self.assertEqual(sent.call_args[0][0], {'type': 'map.truck.run.ended', 'id': str(run_id)})
        self.assertEqual(self.client.get('/api/sync/').data['truck_runs'], [])


class RouteEditorBackendTests(TestCase):
    """Fase 3: puntos de control, ajuste a calles (OSRM) y borrado seguro."""

    def setUp(self):
        from django.core.cache import cache
        cache.clear()
        self.client = APIClient()
        self.staff_user = User.objects.create_user(username='editor', password='x', is_staff=True)
        self.citizen = User.objects.create_user(username='vecino2', password='x')
        self.waypoints = [[-10.6625, -76.2555], [-10.6640, -76.2530]]

    def _osrm_response(self, status_code=200, payload=None):
        from unittest import mock
        res = mock.Mock()
        res.status_code = status_code
        res.json.return_value = payload if payload is not None else {
            'code': 'Ok',
            'routes': [{
                'distance': 412.3,
                'duration': 61.0,
                'geometry': {'coordinates': [[-76.2555, -10.6625], [-76.2550, -10.6630], [-76.2530, -10.6640]]},
            }],
        }
        return res

    def test_snap_requires_staff(self):
        self.client.force_authenticate(user=self.citizen)
        res = self.client.post('/api/routes/snap/', {'waypoints': self.waypoints}, format='json')
        self.assertEqual(res.status_code, status.HTTP_403_FORBIDDEN)

    def test_snap_returns_lat_lng_path_and_caches(self):
        from unittest import mock
        self.client.force_authenticate(user=self.staff_user)
        with mock.patch('alerts.road_snapping.httpx.get', return_value=self._osrm_response()) as get:
            res = self.client.post('/api/routes/snap/', {'waypoints': self.waypoints}, format='json')
            self.client.post('/api/routes/snap/', {'waypoints': self.waypoints}, format='json')
        self.assertEqual(res.status_code, status.HTTP_200_OK, res.data)
        self.assertEqual(res.data['path'][0], [-10.6625, -76.2555])  # convertido de [lng, lat]
        self.assertEqual(res.data['distance_m'], 412.3)
        self.assertEqual(get.call_count, 1)
        self.assertIn('-76.255500,-10.662500;-76.253000,-10.664000', get.call_args[0][0])

    def test_snap_errors_are_readable(self):
        from unittest import mock
        import httpx
        self.client.force_authenticate(user=self.staff_user)
        with mock.patch('alerts.road_snapping.httpx.get', return_value=self._osrm_response(400, {'code': 'NoRoute'})):
            res = self.client.post('/api/routes/snap/', {'waypoints': self.waypoints}, format='json')
        self.assertEqual(res.status_code, status.HTTP_502_BAD_GATEWAY)
        self.assertIn('calle', res.data['error'])
        with mock.patch('alerts.road_snapping.httpx.get', side_effect=httpx.ConnectError('down')):
            res = self.client.post('/api/routes/snap/', {'waypoints': [[-10.66, -76.25], [-10.67, -76.26]]}, format='json')
        self.assertEqual(res.status_code, status.HTTP_502_BAD_GATEWAY)
        res = self.client.post('/api/routes/snap/', {'waypoints': [[-10.66, -76.25]]}, format='json')
        self.assertEqual(res.status_code, status.HTTP_400_BAD_REQUEST)

    def test_simplify_keeps_shape_within_budget(self):
        from alerts.road_snapping import _fit_vertex_budget, MAX_OUTPUT_VERTICES
        import math
        pts = [[-10.66 + 0.01 * math.sin(i / 50), -76.25 + i * 0.00001] for i in range(6000)]
        out = _fit_vertex_budget(pts)
        self.assertLessEqual(len(out), MAX_OUTPUT_VERTICES)
        self.assertEqual(out[0], pts[0])
        self.assertEqual(out[-1], pts[-1])

    def test_route_stores_waypoints(self):
        self.client.force_authenticate(user=self.staff_user)
        res = self.client.post('/api/routes/', {
            'name': 'Con editor', 'color': '#123456', 'snap_to_roads': True,
            'path': [[-10.6625, -76.2555], [-10.6630, -76.2550], [-10.6640, -76.2530]],
            'waypoints': self.waypoints,
        }, format='json')
        self.assertEqual(res.status_code, status.HTTP_201_CREATED, res.data)
        self.assertEqual(res.data['waypoints'], self.waypoints)
        self.assertTrue(res.data['snap_to_roads'])
        bad = self.client.post('/api/routes/', {'name': 'x', 'path': self.waypoints, 'waypoints': [[1, 'a']]}, format='json')
        self.assertEqual(bad.status_code, status.HTTP_400_BAD_REQUEST)

    def test_delete_route_with_history_is_blocked(self):
        from alerts.models import CollectionRoute, RouteRun, Truck
        route = CollectionRoute.objects.create(name='Con historial', path=self.waypoints)
        truck = Truck.objects.create(code='R-09', plate='HIS-009')
        RouteRun.objects.create(route=route, truck=truck, status='finished')
        self.client.force_authenticate(user=self.staff_user)
        res = self.client.delete(f'/api/routes/{route.id}/')
        self.assertEqual(res.status_code, status.HTTP_409_CONFLICT)
        empty = CollectionRoute.objects.create(name='Sin historial', path=self.waypoints)
        self.assertEqual(self.client.delete(f'/api/routes/{empty.id}/').status_code, status.HTTP_204_NO_CONTENT)


class CitizenSecurityAlertsTests(TestCase):
    def setUp(self):
        from django.core.cache import cache
        from django.contrib.auth.models import Group
        from alerts.models import SERENAZGO_GROUP_NAME

        cache.clear()
        self.client = APIClient()
        self.citizen = User.objects.create_user(username='vecino_real', password='password123')
        self.other_citizen = User.objects.create_user(username='otro_vecino', password='password123')
        self.env_staff = User.objects.create_user(username='fiscal_ambiental', password='password123', is_staff=True)
        self.superuser = User.objects.create_superuser(username='super_pasco', email='sup@pasco.pe', password='password123')

        serenazgo_group, _ = Group.objects.get_or_create(name=SERENAZGO_GROUP_NAME)
        self.serenazgo_user = User.objects.create_user(username='sereno_1', password='password123')
        self.serenazgo_user.groups.add(serenazgo_group)

        self.citizen_payload = {
            'title': 'Personas ebrias alterando el orden',
            'description': 'Grupo consumiendo alcohol frente al parque.',
            'category': 'convivencia',
            'severity': 'medio',
            'district': 'yanacancha',
            'address': 'Jr. Los Próceres 120',
            'latitude': -10.6650,
            'longitude': -76.2540,
        }

    def tearDown(self):
        from django.core.cache import cache
        cache.clear()

    def _make_webp_upload(self, name='evidencia.png'):
        from PIL import Image
        import io
        buf = io.BytesIO()
        Image.new('RGB', (40, 40), color=(120, 40, 40)).save(buf, format='PNG')
        return SimpleUploadedFile(name, buf.getvalue(), content_type='image/png')

    def test_citizen_alert_requires_account_but_is_anonymous_publicly_and_expires_in_6h(self):
        from django.utils import timezone
        self.client.force_authenticate(user=self.citizen)
        res = self.client.post('/api/alerts/', self.citizen_payload, format='json')
        self.assertEqual(res.status_code, status.HTTP_201_CREATED, res.data)

        alert = Alert.objects.get(pk=res.data['id'])
        self.assertEqual(alert.reported_by, self.citizen)
        self.assertEqual(alert.domain, 'ciudadana')
        self.assertTrue(alert.is_anonymous_public)
        self.assertTrue(alert.hide_media_public)
        self.assertIsNotNone(alert.expires_at)
        ttl_hours = (alert.expires_at - alert.created_at).total_seconds() / 3600.0
        self.assertGreater(ttl_hours, 5.9)
        self.assertLessEqual(ttl_hours, 6.05)

        # Público (/api/sync/) nunca ve username ni reported_by
        self.client.force_authenticate(user=None)
        sync_res = self.client.get('/api/sync/')
        self.assertEqual(sync_res.status_code, status.HTTP_200_OK)
        self.assertEqual(len(sync_res.data['alerts']), 1)
        pub = sync_res.data['alerts'][0]
        self.assertEqual(pub['domain'], 'ciudadana')
        self.assertEqual(pub['reporter_name'], 'Vecino Anónimo')
        self.assertIsNone(pub['reported_by'])
        self.assertNotIn('vecino_real', pub['history_log'] or '')

        # Al expirar las 6h desaparece del mapa público, pero Serenazgo con include_expired=1 sigue viéndola
        Alert.objects.filter(pk=alert.id).update(expires_at=timezone.now() - timezone.timedelta(minutes=5))
        sync_after = self.client.get('/api/sync/')
        self.assertEqual(len(sync_after.data['alerts']), 0)

        self.client.force_authenticate(user=self.serenazgo_user)
        sereno_hist = self.client.get('/api/alerts/?domain=ciudadana&include_expired=1')
        self.assertEqual(sereno_hist.status_code, status.HTTP_200_OK)
        self.assertEqual(len(sereno_hist.data), 1)

    def test_private_evidence_upload_and_serenazgo_only_access(self):
        # Anónimo no puede subir evidencia privada
        anon_up = self.client.post('/api/upload/?scope=ciudadana', {'image': self._make_webp_upload()}, format='multipart')
        self.assertEqual(anon_up.status_code, status.HTTP_401_UNAUTHORIZED)

        self.client.force_authenticate(user=self.citizen)
        up_res = self.client.post('/api/upload/?scope=ciudadana', {'image': self._make_webp_upload()}, format='multipart')
        self.assertEqual(up_res.status_code, status.HTTP_200_OK, up_res.data)
        self.assertTrue(up_res.data['is_private'])
        self.assertIn('/api/evidence/private/', up_res.data['url'])

        payload = dict(self.citizen_payload, image_url=up_res.data['url'])
        create_res = self.client.post('/api/alerts/', payload, format='json')
        self.assertEqual(create_res.status_code, status.HTTP_201_CREATED, create_res.data)
        alert_id = create_res.data['id']

        # Público no recibe image_url en /api/sync/, solo has_private_evidence=True
        self.client.force_authenticate(user=None)
        sync_res = self.client.get('/api/sync/')
        pub = sync_res.data['alerts'][0]
        self.assertIsNone(pub['image_url'])
        self.assertTrue(pub['has_private_evidence'])

        # Anónimo, otro ciudadano y staff ambiental NO pueden abrir el endpoint protegido
        ev_url = f'/api/alerts/{alert_id}/evidence/1/'
        self.assertEqual(self.client.get(ev_url).status_code, status.HTTP_401_UNAUTHORIZED)

        self.client.force_authenticate(user=self.other_citizen)
        self.assertEqual(self.client.get(ev_url).status_code, status.HTTP_403_FORBIDDEN)

        self.client.force_authenticate(user=self.env_staff)
        self.assertEqual(self.client.get(ev_url).status_code, status.HTTP_403_FORBIDDEN)

        # Serenazgo, superusuario y el propio denunciante sí pueden ver la evidencia
        for allowed_user in (self.serenazgo_user, self.superuser, self.citizen):
            self.client.force_authenticate(user=allowed_user)
            resp = self.client.get(ev_url)
            self.assertEqual(resp.status_code, status.HTTP_200_OK)
            resp.close()

    def test_domain_management_isolation_and_public_verification_blocking(self):
        self.client.force_authenticate(user=self.citizen)
        c_res = self.client.post('/api/alerts/', self.citizen_payload, format='json')
        citizen_alert_id = c_res.data['id']

        env_alert = Alert.objects.create(
            title='Botadero en carretera',
            description='Acumulación de residuos.',
            category='basura',
            severity='medio',
            district='chaupimarca',
            address='Av. Circunvalación',
            latitude=-10.67,
            longitude=-76.26,
            reported_by=self.citizen,
        )

        # Staff ambiental no-superuser NO puede gestionar alertas ciudadanas
        self.client.force_authenticate(user=self.env_staff)
        self.assertEqual(
            self.client.patch(f'/api/alerts/{citizen_alert_id}/', {'status': 'solved'}, format='json').status_code,
            status.HTTP_403_FORBIDDEN,
        )

        # Serenazgo sí gestiona ciudadanas, pero NO ambientales
        self.client.force_authenticate(user=self.serenazgo_user)
        self.assertEqual(
            self.client.patch(f'/api/alerts/{citizen_alert_id}/', {'status': 'solved', 'resolution_comment': 'Patrulla en zona'}, format='json').status_code,
            status.HTTP_200_OK,
        )
        self.assertEqual(
            self.client.patch(f'/api/alerts/{env_alert.id}/', {'status': 'solved'}, format='json').status_code,
            status.HTTP_403_FORBIDDEN,
        )

        # Verificación QR pública ambiental y export_csv ciudadano bloqueados al público
        self.client.force_authenticate(user=None)
        self.assertEqual(self.client.get(f'/api/alerts/{citizen_alert_id}/verify/').status_code, status.HTTP_404_NOT_FOUND)
        self.assertEqual(self.client.get('/api/alerts/export_csv/?domain=ciudadana').status_code, status.HTTP_403_FORBIDDEN)

        self.client.force_authenticate(user=self.serenazgo_user)
        csv_res = self.client.get('/api/alerts/export_csv/?domain=ciudadana')
        self.assertEqual(csv_res.status_code, status.HTTP_200_OK)
        csv_text = csv_res.content.decode('utf-8')
        self.assertIn('Vecino Anónimo', csv_text)
        self.assertNotIn('vecino_real', csv_text)


class AuthImprovementsTests(TestCase):
    def setUp(self):
        from django.core.cache import cache
        from django.contrib.auth.models import Group
        from alerts.models import SystemConfig

        cache.clear()
        self.client = APIClient()
        SystemConfig.objects.update_or_create(
            key='GOOGLE_CLIENT_ID',
            defaults={'value': 'test-client-id.apps.googleusercontent.com'},
        )
        self.citizen = User.objects.create_user(
            username='vecino_login',
            email='vecino.login@example.com',
            password='PasswordSegura123!',
        )
        self.staff_user = User.objects.create_user(
            username='admin_oficial',
            email='admin.oficial@pasco.gob.pe',
            password='PasswordOficial123!',
            is_staff=True,
        )
        serenazgo_group, _ = Group.objects.get_or_create(name='serenazgo')
        self.serenazgo_user = User.objects.create_user(
            username='sereno_oficial',
            email='sereno.oficial@pasco.gob.pe',
            password='PasswordOficial123!',
        )
        self.serenazgo_user.groups.add(serenazgo_group)

        collector_group, _ = Group.objects.get_or_create(name='recolectores')
        self.collector_user = User.objects.create_user(
            username='chofer_oficial',
            email='chofer.oficial@pasco.gob.pe',
            password='PasswordOficial123!',
        )
        self.collector_user.groups.add(collector_group)

    def tearDown(self):
        from django.core.cache import cache
        cache.clear()

    def test_login_accepts_email_or_username(self):
        resp_email = self.client.post('/api/auth/login/', {
            'username': 'VECINO.LOGIN@example.com',
            'password': 'PasswordSegura123!',
        }, format='json')
        self.assertEqual(resp_email.status_code, status.HTTP_200_OK)
        self.assertEqual(resp_email.data['username'], 'vecino_login')
        self.assertIn('token', resp_email.data)

        resp_user = self.client.post('/api/auth/login/', {
            'username': 'vecino_login',
            'password': 'PasswordSegura123!',
        }, format='json')
        self.assertEqual(resp_user.status_code, status.HTTP_200_OK)
        self.assertEqual(resp_user.data['email'], 'vecino.login@example.com')

    def test_register_enforces_min_password_and_returns_role_flags(self):
        short_resp = self.client.post('/api/auth/register/', {
            'username': 'clave_corta',
            'email': 'Corta@Example.com',
            'password': '1234567',
            'turnstile_token': 'dummy_test_token',
        }, format='json')
        self.assertEqual(short_resp.status_code, status.HTTP_400_BAD_REQUEST)

        ok_resp = self.client.post('/api/auth/register/', {
            'username': 'nuevo_pasco',
            'email': 'Nuevo.Pasco@Example.com',
            'password': 'ClaveLarga123!',
            'turnstile_token': 'dummy_test_token',
        }, format='json')
        self.assertEqual(ok_resp.status_code, status.HTTP_201_CREATED)
        self.assertEqual(ok_resp.data['email'], 'nuevo.pasco@example.com')
        self.assertFalse(ok_resp.data['is_staff'])
        self.assertFalse(ok_resp.data['is_superuser'])
        self.assertFalse(ok_resp.data['is_collector'])
        self.assertFalse(ok_resp.data['is_serenazgo'])

    def test_google_auth_logs_in_citizen_and_blocks_official_accounts(self):
        import json
        from unittest.mock import patch, MagicMock

        def make_mock_response(payload):
            mock_resp = MagicMock()
            mock_resp.status = 200
            mock_resp.read.return_value = json.dumps(payload).encode('utf-8')
            mock_cm = MagicMock()
            mock_cm.__enter__.return_value = mock_resp
            mock_cm.__exit__.return_value = False
            return mock_cm

        citizen_payload = {
            'iss': 'https://accounts.google.com',
            'aud': 'test-client-id.apps.googleusercontent.com',
            'email_verified': 'true',
            'email': 'ciudadano.google@gmail.com',
            'name': 'Vecino Google',
            'picture': 'https://example.com/avatar.png',
            'sub': 'google-sub-123',
        }
        with patch('urllib.request.urlopen', return_value=make_mock_response(citizen_payload)):
            resp = self.client.post('/api/auth/google/', {'credential': 'fake-jwt-citizen'}, format='json')
        self.assertEqual(resp.status_code, status.HTTP_200_OK)
        self.assertIn('token', resp.data)
        self.assertEqual(resp.data['email'], 'ciudadano.google@gmail.com')
        self.assertFalse(resp.data['is_staff'])
        self.assertFalse(resp.data['is_serenazgo'])

        for official_email in (
            'admin.oficial@pasco.gob.pe',
            'sereno.oficial@pasco.gob.pe',
            'chofer.oficial@pasco.gob.pe',
        ):
            payload = {
                **citizen_payload,
                'email': official_email,
                'name': 'Oficial',
            }
            with patch('urllib.request.urlopen', return_value=make_mock_response(payload)):
                blocked = self.client.post('/api/auth/google/', {'credential': 'fake-jwt-official'}, format='json')
            self.assertEqual(blocked.status_code, status.HTTP_403_FORBIDDEN, f"Expected 403 for {official_email}")


class LiveEvidenceAndGpsIntegrityTests(TestCase):
    def setUp(self):
        from django.core.cache import cache
        cache.clear()
        self.client = APIClient()
        self.citizen = User.objects.create_user(
            username='vecino_camara',
            email='vecino.camara@example.com',
            password='PasswordSegura123!',
        )
        self.other_citizen = User.objects.create_user(
            username='vecino_camara_2',
            email='vecino.camara2@example.com',
            password='PasswordSegura123!',
        )
        self.base_env_payload = {
            'title': 'Botadero clandestino en Yanacancha',
            'description': 'Acumulación de residuos sólidos junto a la vía.',
            'category': 'basura',
            'severity': 'medio',
            'district': 'yanacancha',
            'address': 'Av. Los Próceres 400, Yanacancha',
            'latitude': -10.6520,
            'longitude': -76.2420,
            'reporter_latitude': -10.6522,
            'reporter_longitude': -76.2421,
        }

    def tearDown(self):
        from django.core.cache import cache
        cache.clear()

    def _make_textured_image(self, seed=0, size=(320, 240), crop_margin=0, format_name='JPEG'):
        import io
        from PIL import Image, ImageDraw
        img = Image.new('RGB', size, (30 + seed * 15, 60, 90))
        draw = ImageDraw.Draw(img)
        for i in range(0, size[0], 24):
            color = ((i * 7 + seed * 53) % 255, (i * 11 + seed * 29) % 255, (i * 13 + 80) % 255)
            draw.rectangle([i, (i * 3 + seed * 20) % (size[1] - 40), i + 20, size[1] - 10], fill=color)
        draw.ellipse([40 + seed * 10, 30, 180, 170], fill=(220, 180 - seed * 30, 40))
        if crop_margin > 0:
            img = img.crop((crop_margin, crop_margin, size[0] - crop_margin, size[1] - crop_margin))
            img = img.resize((240, 180), Image.Resampling.LANCZOS)
        buf = io.BytesIO()
        img.save(buf, format=format_name, quality=85)
        return buf.getvalue()

    def test_environmental_alert_requires_photo_and_nearby_gps(self):
        self.client.force_authenticate(user=self.citizen)

        # 1. Sin foto -> 400
        no_photo = dict(self.base_env_payload)
        res_no_photo = self.client.post('/api/alerts/', no_photo, format='json')
        self.assertEqual(res_no_photo.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('fotografía en vivo', str(res_no_photo.data))

        # Subir foto válida de cámara en vivo
        img_bytes = self._make_textured_image(seed=1)
        up_file = SimpleUploadedFile('camara_1.jpg', img_bytes, content_type='image/jpeg')
        up_res = self.client.post(
            '/api/upload/?purpose=alert',
            {'image': up_file, 'capture_source': 'live_camera'},
            format='multipart',
        )
        self.assertEqual(up_res.status_code, status.HTTP_200_OK, up_res.data)
        self.assertEqual(up_res.data['capture_source'], 'live_camera')
        self.assertTrue(up_res.data['sha256'])
        self.assertEqual(len(up_res.data['phash']), 16)

        # 2. Sin GPS del dispositivo -> 400
        no_gps = dict(self.base_env_payload, image_url=up_res.data['url'])
        no_gps.pop('reporter_latitude')
        no_gps.pop('reporter_longitude')
        res_no_gps = self.client.post('/api/alerts/', no_gps, format='json')
        self.assertEqual(res_no_gps.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('GPS', str(res_no_gps.data))

        # 3. Con GPS lejano (> 300m) -> 400
        far_gps = dict(self.base_env_payload, image_url=up_res.data['url'], reporter_latitude=-10.6720)
        res_far = self.client.post('/api/alerts/', far_gps, format='json')
        self.assertEqual(res_far.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('metros del punto marcado', str(res_far.data))

        # 4. Con foto en vivo + GPS cercano (< 300m) -> 201 y sin filtrar coords privadas del denunciante
        ok_payload = dict(self.base_env_payload, image_url=up_res.data['url'])
        res_ok = self.client.post('/api/alerts/', ok_payload, format='json')
        self.assertEqual(res_ok.status_code, status.HTTP_201_CREATED, res_ok.data)
        self.assertTrue(res_ok.data['gps_verified'])
        self.assertEqual(res_ok.data['capture_source'], 'live_camera')
        self.assertNotIn('reporter_latitude', res_ok.data)
        self.assertNotIn('reporter_longitude', res_ok.data)
        self.assertNotIn('evidence_phash_1', res_ok.data)

    def test_gallery_source_rejected_on_upload_and_create(self):
        self.client.force_authenticate(user=self.citizen)
        img_bytes = self._make_textured_image(seed=2)
        up_file = SimpleUploadedFile('galeria.jpg', img_bytes, content_type='image/jpeg')
        resp = self.client.post(
            '/api/upload/?purpose=alert',
            {'image': up_file, 'capture_source': 'gallery'},
            format='multipart',
        )
        self.assertEqual(resp.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('cámara', str(resp.data))

    def test_duplicate_photo_sha256_and_perceptual_dhash_blocked(self):
        self.client.force_authenticate(user=self.citizen)
        img1_bytes = self._make_textured_image(seed=3, size=(400, 300))
        up1 = self.client.post(
            '/api/upload/?purpose=alert',
            {'image': SimpleUploadedFile('foto1.jpg', img1_bytes, content_type='image/jpeg'), 'capture_source': 'live_camera'},
            format='multipart',
        )
        self.assertEqual(up1.status_code, status.HTTP_200_OK)

        p1 = dict(self.base_env_payload, image_url=up1.data['url'])
        create1 = self.client.post('/api/alerts/', p1, format='json')
        self.assertEqual(create1.status_code, status.HTTP_201_CREATED)

        # Otro usuario intenta subir la misma foto exacta en otro lugar -> 400 en upload
        self.client.force_authenticate(user=self.other_citizen)
        up_dup_exact = self.client.post(
            '/api/upload/?purpose=alert',
            {'image': SimpleUploadedFile('foto1_copy.jpg', img1_bytes, content_type='image/jpeg'), 'capture_source': 'live_camera'},
            format='multipart',
        )
        self.assertEqual(up_dup_exact.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('ya fue utilizada', str(up_dup_exact.data))

        # Intento con leve recorte/recompresión/reescalado (dHash perceptual) -> 400 en upload
        cropped_bytes = self._make_textured_image(seed=3, size=(400, 300), crop_margin=4, format_name='PNG')
        up_dup_perceptual = self.client.post(
            '/api/upload/?purpose=alert',
            {'image': SimpleUploadedFile('foto1_crop.png', cropped_bytes, content_type='image/png'), 'capture_source': 'live_camera'},
            format='multipart',
        )
        self.assertEqual(up_dup_perceptual.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('ya fue utilizada', str(up_dup_perceptual.data))

        # Incluso si se sube sin purpose=alert, perform_create debe bloquear el reporte duplicado
        up_bypass = self.client.post(
            '/api/upload/',
            {'image': SimpleUploadedFile('bypass.png', cropped_bytes, content_type='image/png')},
            format='multipart',
        )
        self.assertEqual(up_bypass.status_code, status.HTTP_200_OK)
        far_location_payload = dict(
            self.base_env_payload,
            title='Otro punto distinto',
            latitude=-10.6900,
            longitude=-76.2700,
            reporter_latitude=-10.6901,
            reporter_longitude=-76.2701,
            image_url=up_bypass.data['url'],
        )
        blocked_create = self.client.post('/api/alerts/', far_location_payload, format='json')
        self.assertEqual(blocked_create.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('ya fue utilizada', str(blocked_create.data))

        # Foto genuinamente distinta en otra ubicación -> 201 OK
        img2_bytes = self._make_textured_image(seed=9, size=(400, 300))
        up2 = self.client.post(
            '/api/upload/?purpose=alert',
            {'image': SimpleUploadedFile('foto2.jpg', img2_bytes, content_type='image/jpeg'), 'capture_source': 'live_camera'},
            format='multipart',
        )
        self.assertEqual(up2.status_code, status.HTTP_200_OK)
        ok_second = dict(far_location_payload, image_url=up2.data['url'])
        create2 = self.client.post('/api/alerts/', ok_second, format='json')
        self.assertEqual(create2.status_code, status.HTTP_201_CREATED, create2.data)

    def test_citizen_security_keeps_photo_optional_but_checks_gps_distance(self):
        self.client.force_authenticate(user=self.citizen)
        citizen_payload = {
            'title': 'Zona sin alumbrado público',
            'description': 'Poste apagado en esquina oscura.',
            'category': 'riesgo_urbano',
            'severity': 'medio',
            'district': 'chaupimarca',
            'address': 'Jr. Tarapacá 200',
            'latitude': -10.6750,
            'longitude': -76.2555,
            'reporter_latitude': -10.7100,
            'reporter_longitude': -76.2900,
        }
        far_res = self.client.post('/api/alerts/', citizen_payload, format='json')
        self.assertEqual(far_res.status_code, status.HTTP_400_BAD_REQUEST)

        citizen_payload['reporter_latitude'] = -10.6752
        citizen_payload['reporter_longitude'] = -76.2556
        ok_res = self.client.post('/api/alerts/', citizen_payload, format='json')
        self.assertEqual(ok_res.status_code, status.HTTP_201_CREATED, ok_res.data)
        self.assertTrue(ok_res.data['gps_verified'])


