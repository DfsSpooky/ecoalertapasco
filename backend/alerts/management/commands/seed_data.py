from django.core.management.base import BaseCommand
from alerts.models import Alert
from django.utils import timezone
import datetime

class Command(BaseCommand):
    help = 'Poblar la base de datos con datos de prueba reales para Cerro de Pasco'

    def add_arguments(self, parser):
        parser.add_argument(
            '--force',
            action='store_true',
            help='Forzar el vaciado y la reinserción de datos de prueba',
        )

    def handle(self, *args, **options):
        force = options.get('force', False)
        
        # Crear usuarios de autoridad por defecto (siempre se valida)
        from django.contrib.auth import get_user_model
        User = get_user_model()
        
        authorities = [
            {'username': 'autoridad_pasco', 'email': 'pasco@ecoalerta.gov.pe', 'is_staff': True},
            {'username': 'autoridad_yanacancha', 'email': 'yanacancha@ecoalerta.gov.pe', 'is_staff': True},
            {'username': 'autoridad_bolivar', 'email': 'bolivar@ecoalerta.gov.pe', 'is_staff': True},
        ]
        
        for auth in authorities:
            if not User.objects.filter(username=auth['username']).exists():
                User.objects.create_user(
                    username=auth['username'],
                    email=auth['email'],
                    password='EcoalertaSecure123!',
                    is_staff=auth['is_staff']
                )
                self.stdout.write(self.style.SUCCESS(f"Usuario de autoridad '{auth['username']}' creado (Pass: EcoalertaSecure123!)."))

        # Inicializar configuración del sitio por defecto si no existe
        from alerts.models import SiteConfiguration
        SiteConfiguration.objects.get_or_create(id=1)

        # Verificar si ya existen alertas
        if Alert.objects.exists() and not force:
            self.stdout.write(self.style.WARNING('La base de datos ya contiene alertas. Omitiendo población de semillas para proteger datos reales.'))
            self.stdout.write('Use --force si desea limpiar y repoblar la base de datos de alertas.')
            return

        # Limpiar base de datos
        self.stdout.write('Limpiando base de datos de alertas...')
        Alert.objects.all().delete()
        
        # Datos de prueba
        alerts_data = [
            {
                'title': 'Relaves Mineros de Quiulacocha',
                'description': 'Filtración y arrastre de sedimentos ácidos con metales pesados desde la desmontera hacia bofedales locales.',
                'category': 'mineria',
                'severity': 'critico',
                'district': 'simonBolivar',
                'address': 'Presa de Relaves Quiulacocha, Simón Bolívar',
                'latitude': -10.7022,
                'longitude': -76.2871,
                'status': 'pending',
                'created_offset_days': 2
            },
            {
                'title': 'Polvo de Tajo Abierto Raul Rojas',
                'description': 'Presencia de partículas de polvo en suspensión que provienen del movimiento de tierras del tajo abierto central.',
                'category': 'mineria',
                'severity': 'critico',
                'district': 'chaupimarca',
                'address': 'Jirón Lampa s/n (Borde del Tajo Abierto), Chaupimarca',
                'latitude': -10.6720,
                'longitude': -76.2570,
                'status': 'verified',
                'created_offset_days': 5
            },
            {
                'title': 'Plomo en Suelo de Recreo Escolar',
                'description': 'Medición de concentración de plomo excede los límites permisibles en áreas verdes contiguas a la escuela de Chaupimarca.',
                'category': 'mineria',
                'severity': 'critico',
                'district': 'chaupimarca',
                'address': 'Jirón Daniel Alcides Carrión 112 (Plaza Daniel Alcides Carrión), Chaupimarca',
                'latitude': -10.6781,
                'longitude': -76.2545,
                'status': 'pending',
                'created_offset_days': 12
            },
            {
                'title': 'Aguas Ácidas en Laguna Patarcocha',
                'description': 'Coloración verdosa inusual y emanación de gases sulfhídricos debido al vertido de aguas residuales y drenajes ácidos urbanos.',
                'category': 'agua',
                'severity': 'critico',
                'district': 'chaupimarca',
                'address': 'Jirón Alfonso Ugarte (Ribera Este de Laguna Patarcocha), Chaupimarca',
                'latitude': -10.6655,
                'longitude': -76.2525,
                'status': 'solved',
                'created_offset_days': 3,
                'resolution_comment': 'Se instaló un sistema piloto de aireación por microburbujas y cal para neutralizar la acidez del agua.',
                'resolution_image_url': 'https://images.unsplash.com/photo-1541888946425-d81bb19240f5?q=80&w=600&auto=format&fit=crop'
            },
            {
                'title': 'Basural Acumulado en Av. El Minero',
                'description': 'Punto crítico de acumulación de residuos sólidos domiciliarios y comerciales que bloquean la vereda peatonal.',
                'category': 'basura',
                'severity': 'medio',
                'district': 'yanacancha',
                'address': 'Avenida El Minero (Sector Industrial La Esperanza), Yanacancha',
                'latitude': -10.6590,
                'longitude': -76.2480,
                'status': 'pending',
                'created_offset_days': 4
            },
            {
                'title': 'Monóxido y Ruido en Av. Italia',
                'description': 'Emisiones vehiculares excesivas de buses y camiones de carga pesada en hora punta debido a embotellamiento crónico.',
                'category': 'aire',
                'severity': 'medio',
                'district': 'chaupimarca',
                'address': 'Avenida Bolognesi 302 (Frente a Municipalidad), Chaupimarca',
                'latitude': -10.6790,
                'longitude': -76.2550,
                'status': 'pending',
                'created_offset_days': 7
            },
            {
                'title': 'Residuos Hospitalarios Expuestos',
                'description': 'Desechos de jeringas y gasas encontrados en contenedores abiertos fuera del Hospital Regional de Yanacancha.',
                'category': 'basura',
                'severity': 'critico',
                'district': 'yanacancha',
                'address': 'Avenida Los Próceres (Frente al Hospital Regional), Yanacancha',
                'latitude': -10.6480,
                'longitude': -76.2490,
                'status': 'pending',
                'created_offset_days': 1
            },
            {
                'title': 'Acumulación de Residuos de Construcción',
                'description': 'Desmonte de ladrillos y tierra abandonado en plena Avenida Las Américas obstruyendo el carril de subida.',
                'category': 'basura',
                'severity': 'bajo',
                'district': 'yanacancha',
                'address': 'Avenida Las Américas (Esquina Av. 6 de Diciembre), Yanacancha',
                'latitude': -10.6640,
                'longitude': -76.2550,
                'status': 'solved',
                'created_offset_days': 10,
                'resolution_comment': 'El personal de limpieza municipal retiró el desmonte con maquinaria pesada y multó al propietario.',
                'resolution_image_url': 'https://images.unsplash.com/photo-1541888946425-d81bb19240f5?q=80&w=600&auto=format&fit=crop'
            },
            {
                'title': 'Desborde de Aguas Servidas en Calle Mariátegui',
                'description': 'Colapso de una tubería matriz de alcantarillado, generando olores fétidos cerca a las viviendas.',
                'category': 'agua',
                'severity': 'critico',
                'district': 'yanacancha',
                'address': 'Calle Mariátegui (Cerca a Av. Las Américas), Yanacancha',
                'latitude': -10.6630,
                'longitude': -76.2540,
                'status': 'verified',
                'created_offset_days': 3
            },
            {
                'title': 'Erosión de Suelos en Champamarca',
                'description': 'Pérdida de capa arable y sedimentación debido al escurrimiento superficial de aguas industriales sin tratamiento.',
                'category': 'mineria',
                'severity': 'medio',
                'district': 'simonBolivar',
                'address': 'Comunidad de Champamarca Sector Agropecuario, Simón Bolívar',
                'latitude': -10.7100,
                'longitude': -76.2730,
                'status': 'pending',
                'created_offset_days': 8
            }
        ]
        
        now = timezone.now()
        for data in alerts_data:
            offset = data.pop('created_offset_days')
            created_at = now - datetime.timedelta(days=offset)
            
            # Extraer campos de resolución si los hay
            res_comment = data.pop('resolution_comment', None)
            res_img = data.pop('resolution_image_url', None)
            
            alert = Alert.objects.create(
                **data
            )
            
            # Hack para poner fecha de creación exacta del pasado (auto_now_add lo sobreescribe en create)
            Alert.objects.filter(pk=alert.pk).update(created_at=created_at)
            
            if data['status'] == 'solved':
                Alert.objects.filter(pk=alert.pk).update(
                    resolution_comment=res_comment,
                    resolution_image_url=res_img,
                    resolved_at=created_at + datetime.timedelta(hours=6)
                )
                
        # Poblar MunicipalDump y CollectorRoutePoint
        from alerts.models import CollectorRoutePoint, MunicipalDump

        if force or not MunicipalDump.objects.exists():
            self.stdout.write('Limpiando base de datos de contenedores y botaderos...')
            MunicipalDump.objects.all().delete()
            CollectorRoutePoint.objects.all().delete()

            # Sembrar Contenedores y Botaderos Oficiales
            dumps_data = [
                # 5 Contenedores originales
                {
                    'name': 'Contenedor Plaza Yanacancha',
                    'description': 'Plaza Principal de Yanacancha. Puntos de separación esmeralda.',
                    'latitude': -10.6625,
                    'longitude': -76.2555,
                    'type': 'recyclable',
                    'fill_level': 'low',
                    'schedule': 'Lunes, Miércoles y Viernes a las 19:00',
                },
                {
                    'name': 'Punto de Acopio Av. Los Próceres',
                    'description': 'Cerca al mercado local. Depósitos de residuos orgánicos municipales.',
                    'latitude': -10.6640,
                    'longitude': -76.2530,
                    'type': 'organic',
                    'fill_level': 'medium',
                    'schedule': 'Martes, Jueves y Sábado a las 18:30',
                },
                {
                    'name': 'Contenedor General Hospital Huariaca',
                    'description': 'Residuos generales municipales no peligrosos.',
                    'latitude': -10.6685,
                    'longitude': -76.2580,
                    'type': 'general',
                    'fill_level': 'full',
                    'schedule': 'Diario (Lunes a Domingo) a las 08:00',
                },
                {
                    'name': 'Contenedor Plaza Quiulacocha',
                    'description': 'Residuos generales. Punto de acopio del distrito Simón Bolívar.',
                    'latitude': -10.6720,
                    'longitude': -76.2625,
                    'type': 'general',
                    'fill_level': 'medium',
                    'schedule': 'Lunes y Jueves a las 14:00',
                },
                {
                    'name': 'Punto Limpio Av. Bolívar Central',
                    'description': 'Contenedores verdes para reciclaje de papel, plástico y vidrio.',
                    'latitude': -10.6705,
                    'longitude': -76.2600,
                    'type': 'recyclable',
                    'fill_level': 'low',
                    'schedule': 'Martes y Sábado a las 16:00',
                },
                # 3 Botaderos oficiales
                {
                    'name': 'Botadero Municipal San Juan',
                    'description': 'Punto de acopio municipal oficial en el Sector San Juan. Autorizado para depositar bolsas de basura domésticas.',
                    'latitude': -10.6750,
                    'longitude': -76.2520,
                    'type': 'municipalDump',
                    'fill_level': 'medium',
                    'schedule': 'Recolección diaria por camión municipal compactador a las 20:00',
                },
                {
                    'name': 'Botadero Oficial Yanacancha Alta',
                    'description': 'Botadero autorizado y supervisado por la Municipalidad Distrital de Yanacancha. Depósito seguro de bolsas de basura.',
                    'latitude': -10.6580,
                    'longitude': -76.2480,
                    'type': 'municipalDump',
                    'fill_level': 'low',
                    'schedule': 'Lunes, Miércoles y Viernes a las 22:00',
                },
                {
                    'name': 'Punto de Desecho Simón Bolívar (La Esperanza)',
                    'description': 'Punto limpio oficial municipal. Contenedor de gran capacidad para almacenamiento temporal de bolsas de basura.',
                    'latitude': -10.6690,
                    'longitude': -76.2710,
                    'type': 'municipalDump',
                    'fill_level': 'full',
                    'schedule': 'Diario a las 06:00',
                },
            ]

            for data_dump in dumps_data:
                MunicipalDump.objects.create(**data_dump)
            self.stdout.write(self.style.SUCCESS(f'Puntos de residuo poblados: {len(dumps_data)} creados.'))

            # Sembrar los puntos de la ruta del recolector (únicamente los 5 contenedores normales)
            route_points = [
                {'latitude': -10.6625, 'longitude': -76.2555, 'order': 1}, # Plaza Yanacancha
                {'latitude': -10.6640, 'longitude': -76.2530, 'order': 2}, # Av. Los Próceres
                {'latitude': -10.6685, 'longitude': -76.2580, 'order': 3}, # Hospital Huariaca
                {'latitude': -10.6705, 'longitude': -76.2600, 'order': 4}, # Av. Bolívar Central
                {'latitude': -10.6720, 'longitude': -76.2625, 'order': 5}, # Plaza Quiulacocha
            ]

            for rp in route_points:
                CollectorRoutePoint.objects.create(**rp)
            self.stdout.write(self.style.SUCCESS(f'Puntos de ruta de recolector poblados: {len(route_points)} creados.'))

        self.stdout.write(self.style.SUCCESS(f'Base de datos poblada con éxito con {len(alerts_data)} alertas.'))
