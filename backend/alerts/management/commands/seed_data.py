from django.core.management.base import BaseCommand, CommandError
from django.conf import settings
from alerts.models import Alert
from django.utils import timezone
import datetime

class Command(BaseCommand):
    help = 'Carga datos iniciales de referencia para entornos de desarrollo local (protegido contra borrado en producción).'

    def add_arguments(self, parser):
        parser.add_argument(
            '--force-reset',
            action='store_true',
            help='Elimina las alertas existentes antes de insertar los datos de prueba (SOLO desarrollo).',
        )
        parser.add_argument(
            '--allow-production',
            action='store_true',
            help='Permite ejecutar el comando aunque DEBUG=False (solo si la tabla está vacía).',
        )

    def handle(self, *args, **options):
        if not settings.DEBUG and not options['allow_production']:
            raise CommandError(
                'SEGURIDAD DE PRODUCCIÓN: seed_data está bloqueado cuando DEBUG=False para proteger las denuncias ciudadanas reales.'
            )

        if options['force_reset']:
            if not settings.DEBUG:
                raise CommandError(
                    'SEGURIDAD CRÍTICA: --force-reset está terminantemente prohibido cuando DEBUG=False.'
                )
            self.stdout.write(self.style.WARNING('Eliminando alertas locales por flag explícito --force-reset...'))
            Alert.objects.all().delete()
        elif Alert.objects.exists():
            self.stdout.write(self.style.NOTICE(
                f'Omitiendo seed_data: ya existen {Alert.objects.count()} alertas registradas en la base de datos.'
            ))
            return

        # Datos de prueba para entorno local
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
                
        self.stdout.write(self.style.SUCCESS(f'Base de datos poblada con éxito con {len(alerts_data)} alertas.'))
