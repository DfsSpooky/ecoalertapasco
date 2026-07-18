from django.test import TestCase
from django.contrib.auth import get_user_model
from django.urls import reverse
from rest_framework.test import APITestCase
from rest_framework import status
from django.core.files.uploadedfile import SimpleUploadedFile
from .models import Alert
import json

User = get_user_model()

class SecurityTests(APITestCase):
    def setUp(self):
        # Crear usuario ciudadano
        self.citizen = User.objects.create_user(username='citizen', password='citizen_pass123', email='citizen@example.com')
        # Crear usuario staff general (Provincial / Chaupimarca por defecto)
        self.staff = User.objects.create_user(username='staff', password='staff_pass123', email='staff@example.com', is_staff=True)
        # Crear autoridades distritales
        self.auth_yanacancha = User.objects.create_user(username='autoridad_yanacancha', password='yanacancha_pass123', email='yanacancha@example.com', is_staff=True)
        self.auth_bolivar = User.objects.create_user(username='autoridad_bolivar', password='bolivar_pass123', email='bolivar@example.com', is_staff=True)
        
        # Obtener tokens con format='json'
        # Login ciudadano
        response = self.client.post(reverse('token_login'), {'username': 'citizen', 'password': 'citizen_pass123'}, format='json')
        self.assertEqual(response.status_code, 200, msg=f"Citizen login failed: {response.content}")
        self.citizen_token = response.data['token']
        
        # Login staff general (Pasco Provincial)
        response = self.client.post(reverse('token_login'), {'username': 'staff', 'password': 'staff_pass123'}, format='json')
        self.assertEqual(response.status_code, 200, msg=f"Staff login failed: {response.content}")
        self.staff_token = response.data['token']

        # Login autoridad Yanacancha
        response = self.client.post(reverse('token_login'), {'username': 'autoridad_yanacancha', 'password': 'yanacancha_pass123'}, format='json')
        self.assertEqual(response.status_code, 200)
        self.yanacancha_token = response.data['token']

        # Login autoridad Simón Bolívar
        response = self.client.post(reverse('token_login'), {'username': 'autoridad_bolivar', 'password': 'bolivar_pass123'}, format='json')
        self.assertEqual(response.status_code, 200)
        self.bolivar_token = response.data['token']
        
        # Crear alerta inicial en Simón Bolívar
        self.alert_bolivar = Alert.objects.create(
            title="Derrame de relaves",
            description="Contaminación severa en Simón Bolívar",
            category="mineria",
            severity="critico",
            district="simonBolivar",
            address="Quiulacocha",
            latitude=-10.7022,
            longitude=-76.2871,
            status="pending"
        )
        self.alert = self.alert_bolivar  # Alias para compatibilidad de tests previos
        
        # Crear alerta inicial en Yanacancha
        self.alert_yanacancha = Alert.objects.create(
            title="Basura acumulada Yanacancha",
            description="Acumulación de basura en Los Próceres",
            category="basura",
            severity="medio",
            district="yanacancha",
            address="Los Próceres",
            latitude=-10.6480,
            longitude=-76.2490,
            status="pending"
        )

    def test_citizen_can_create_alert(self):
        self.client.credentials(HTTP_AUTHORIZATION='Token ' + self.citizen_token)
        url = reverse('alert-list')
        data = {
            'title': 'Basural acumulado',
            'description': 'Acumulación de residuos en Yanacancha',
            'category': 'basura',
            'severity': 'medio',
            'district': 'yanacancha',
            'address': 'Av. Los Próceres',
            'latitude': -10.6480,
            'longitude': -76.2490
        }
        response = self.client.post(url, data, format='json')
        self.assertEqual(response.status_code, status.HTTP_201_CREATED)

    def test_anonymous_cannot_create_alert(self):
        self.client.credentials()  # Quitar credenciales
        url = reverse('alert-list')
        data = {
            'title': 'Basural acumulado',
            'description': 'Acumulación de residuos en Yanacancha',
            'category': 'basura',
            'severity': 'medio',
            'district': 'yanacancha',
            'address': 'Av. Los Próceres',
            'latitude': -10.6480,
            'longitude': -76.2490
        }
        response = self.client.post(url, data, format='json')
        self.assertEqual(response.status_code, status.HTTP_401_UNAUTHORIZED)

    def test_citizen_cannot_edit_alert(self):
        self.client.credentials(HTTP_AUTHORIZATION='Token ' + self.citizen_token)
        url = reverse('alert-detail', args=[self.alert.id])
        data = {'title': 'Título vandalizado por ciudadano'}
        response = self.client.patch(url, data, format='json')
        self.assertEqual(response.status_code, status.HTTP_403_FORBIDDEN)

    def test_staff_can_edit_alert(self):
        self.client.credentials(HTTP_AUTHORIZATION='Token ' + self.staff_token)
        url = reverse('alert-detail', args=[self.alert.id])
        data = {'title': 'Título corregido por autoridad'}
        response = self.client.patch(url, data, format='json')
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.alert.refresh_from_db()
        self.assertEqual(self.alert.title, 'Título corregido por autoridad')

    def test_anonymous_cannot_upload_image(self):
        self.client.credentials()
        url = reverse('image_upload')
        small_gif = (
            b'\x47\x49\x46\x38\x39\x61\x01\x00\x01\x00\x00\x00\x00\x21\xf9\x04'
            b'\x01\x0a\x00\x01\x00\x2c\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02'
            b'\x02\x4c\x01\x00\x3b'
        )
        photo = SimpleUploadedFile("test.gif", small_gif, content_type="image/gif")
        response = self.client.post(url, {'image': photo}, format='multipart')
        self.assertEqual(response.status_code, status.HTTP_401_UNAUTHORIZED)

    def test_authenticated_can_upload_valid_image(self):
        self.client.credentials(HTTP_AUTHORIZATION='Token ' + self.citizen_token)
        url = reverse('image_upload')
        small_gif = (
            b'\x47\x49\x46\x38\x39\x61\x01\x00\x01\x00\x00\x00\x00\x21\xf9\x04'
            b'\x01\x0a\x00\x01\x00\x2c\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02'
            b'\x02\x4c\x01\x00\x3b'
        )
        photo = SimpleUploadedFile("test.gif", small_gif, content_type="image/gif")
        response = self.client.post(url, {'image': photo}, format='multipart')
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertIn('url', response.data)
        # Verificar que se renombró a un UUID
        filename = response.data['url'].split('/')[-1]
        self.assertNotEqual(filename, 'test.gif')
        self.assertTrue(filename.endswith('.gif'))

    def test_authenticated_cannot_upload_invalid_type(self):
        self.client.credentials(HTTP_AUTHORIZATION='Token ' + self.citizen_token)
        url = reverse('image_upload')
        text_file = SimpleUploadedFile("malicious.py", b"print('hack')", content_type="text/plain")
        response = self.client.post(url, {'image': text_file}, format='multipart')
        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('error', response.data)

    def test_authenticated_cannot_upload_too_large_image(self):
        self.client.credentials(HTTP_AUTHORIZATION='Token ' + self.citizen_token)
        url = reverse('image_upload')
        # Crear un archivo de 6 MB
        large_data = b'0' * (6 * 1024 * 1024)
        photo = SimpleUploadedFile("large.png", large_data, content_type="image/png")
        response = self.client.post(url, {'image': photo}, format='multipart')
        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('error', response.data)

    # =========================================================================
    # NUEVOS TESTS DE JURISDICCIÓN Y ROLES DE AUTORIDAD (FASE 2)
    # =========================================================================

    def test_distrital_authority_can_only_see_alerts_in_their_district(self):
        # Iniciar sesión como Yanacancha
        self.client.credentials(HTTP_AUTHORIZATION='Token ' + self.yanacancha_token)
        url = reverse('alert-list')
        response = self.client.get(url)
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        
        # Debe ver solo la de Yanacancha, no la de Simón Bolívar
        alert_ids = [item['id'] for item in response.data]
        self.assertIn(self.alert_yanacancha.id, alert_ids)
        self.assertNotIn(self.alert_bolivar.id, alert_ids)

    def test_distrital_authority_cannot_modify_alerts_outside_their_district(self):
        # Yanacancha intenta editar alerta en Simón Bolívar
        self.client.credentials(HTTP_AUTHORIZATION='Token ' + self.yanacancha_token)
        url = reverse('alert-detail', args=[self.alert_bolivar.id])
        data = {'status': 'solved', 'resolution_comment': 'Intento de solución ilegal'}
        response = self.client.patch(url, data, format='json')
        # Debido al filtrado de queryset por distrito en get_queryset, la alerta externa no existe para esta vista (404)
        self.assertEqual(response.status_code, status.HTTP_404_NOT_FOUND)

    def test_distrital_authority_cannot_transfer_alert_to_another_district(self):
        # Yanacancha intenta transferir su alerta a Simón Bolívar
        self.client.credentials(HTTP_AUTHORIZATION='Token ' + self.yanacancha_token)
        url = reverse('alert-detail', args=[self.alert_yanacancha.id])
        data = {'district': 'simonBolivar'}
        response = self.client.patch(url, data, format='json')
        self.assertEqual(response.status_code, status.HTTP_403_FORBIDDEN)

    def test_provincial_supervisor_can_see_and_modify_everything_and_transfer(self):
        # Iniciar sesión como staff general (provincial)
        self.client.credentials(HTTP_AUTHORIZATION='Token ' + self.staff_token)
        
        # 1. Puede ver todo
        url = reverse('alert-list')
        response = self.client.get(url)
        alert_ids = [item['id'] for item in response.data]
        self.assertIn(self.alert_yanacancha.id, alert_ids)
        self.assertIn(self.alert_bolivar.id, alert_ids)

        # 2. Puede modificar una alerta en Yanacancha
        detail_url = reverse('alert-detail', args=[self.alert_yanacancha.id])
        data = {'status': 'solved', 'resolution_comment': 'Solucionado por Provincial'}
        response = self.client.patch(detail_url, data, format='json')
        self.assertEqual(response.status_code, status.HTTP_200_OK)

        # 3. Puede transferir una alerta de Simón Bolívar a Yanacancha
        detail_url_bolivar = reverse('alert-detail', args=[self.alert_bolivar.id])
        data_transfer = {'district': 'yanacancha'}
        response = self.client.patch(detail_url_bolivar, data_transfer, format='json')
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.alert_bolivar.refresh_from_db()
        self.assertEqual(self.alert_bolivar.district, 'yanacancha')
