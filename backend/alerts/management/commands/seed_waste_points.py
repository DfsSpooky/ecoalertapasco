from django.core.management.base import BaseCommand, CommandError
from django.conf import settings
from alerts.models import WastePoint


class Command(BaseCommand):
    help = 'Registra el catálogo base de estaciones municipales de acopio y reciclaje en Cerro de Pasco sin sobrescribir datos de producción.'

    def add_arguments(self, parser):
        parser.add_argument(
            '--if-empty',
            action='store_true',
            help='Solo inserta los puntos iniciales si la tabla WastePoint está completamente vacía.',
        )
        parser.add_argument(
            '--force-reset',
            action='store_true',
            help='Reemplaza todos los puntos existentes (permitido únicamente con DEBUG=True).',
        )

    def handle(self, *args, **options):
        if options['if_empty'] and WastePoint.objects.exists():
            self.stdout.write(self.style.NOTICE(
                f'Puntos de acopio ya inicializados ({WastePoint.objects.count()} activos). Sin cambios.'
            ))
            return

        if options['force_reset']:
            if not settings.DEBUG:
                raise CommandError('SEGURIDAD: --force-reset no está permitido cuando DEBUG=False.')
            WastePoint.objects.all().delete()
        elif WastePoint.objects.exists():
            self.stdout.write(self.style.NOTICE(
                f'Ya existen {WastePoint.objects.count()} puntos de acopio. Usa --force-reset en desarrollo si deseas recrearlos.'
            ))
            return

        initial_points = [
            {
                'name': 'Punto Limpio de Reciclaje - Plaza Yanacancha',
                'description': 'Estación ecológica municipal con módulos diferenciados para reciclaje de botellas plásticas (PET), latas de aluminio y cartón.',
                'address': 'Plaza Principal de Yanacancha s/n, Yanacancha',
                'latitude': -10.6625,
                'longitude': -76.2555,
                'type': 'recyclable',
                'truck_schedule': 'Lunes, Miércoles y Viernes de 18:00 a 20:00',
                'image_url': '',
                'recommendations': 'Depositar botellas plásticas enjuagadas y aplastadas. No arrojar residuos orgánicos ni desechos sanitarios en estos módulos.',
            },
            {
                'name': 'Punto de Acopio Orgánico - Av. Los Próceres',
                'description': 'Contenedores marrones especiales para residuos biodegradables y restos orgánicos domiciliarios para el programa de compostaje local.',
                'address': 'Av. Los Próceres cuadra 4 (Frente al Mercado El Minero), Yanacancha',
                'latitude': -10.6640,
                'longitude': -76.2530,
                'type': 'organic',
                'truck_schedule': 'Martes, Jueves y Sábado de 06:00 a 08:30',
                'image_url': '',
                'recommendations': 'Apto solo para cáscaras de frutas, verduras, hojas secas y restos de café. Usar bolsas de papel o biodegradables.',
            },
            {
                'name': 'Contenedor Comunal - Hospital Huariaca',
                'description': 'Gran contenedor cerrado para residuos sólidos comunes de vecinos de la zona residencial y comercial.',
                'address': 'Jirón Progreso 230 (Costado del Centro de Salud), Yanacancha',
                'latitude': -10.6685,
                'longitude': -76.2580,
                'type': 'general',
                'truck_schedule': 'Diario (Lunes a Domingo) de 07:00 a 09:00',
                'image_url': '',
                'recommendations': 'Amarrar bien las bolsas de basura. Terminantemente prohibido depositar escombros de construcción o residuos bioinfecciosos.',
            },
            {
                'name': 'Botadero Municipal Controlado - Plaza Quiulacocha',
                'description': 'Punto de disposición de residuos domésticos para vecinos del distrito Simón Bolívar antes del paso de la unidad compactadora.',
                'address': 'Plaza Principal de Quiulacocha s/n, Simón Bolívar',
                'latitude': -10.7020,
                'longitude': -76.2870,
                'type': 'general',
                'truck_schedule': 'Lunes y Jueves de 14:00 a 16:00',
                'image_url': '',
                'recommendations': 'Sacar la basura máximo 30 minutos antes del horario del camión para evitar la dispersión por canes o vientos.',
            },
            {
                'name': 'Estación Ecológica y Pilas - Av. Bolívar Central',
                'description': 'Módulo de acopio integral de residuos reciclables secos y contenedor tubular de seguridad para recolección de pilas y baterías usadas.',
                'address': 'Avenida Bolívar Central 105, Chaupimarca',
                'latitude': -10.6705,
                'longitude': -76.2600,
                'type': 'recyclable',
                'truck_schedule': 'Atención permanente (Lunes a Sábado de 08:00 a 18:00)',
                'image_url': '',
                'recommendations': 'Las pilas y baterías alcalinas deben colocarse en el tubo transparente de acopio para prevenir lixiviados tóxicos.',
            },
        ]

        for data in initial_points:
            WastePoint.objects.create(**data)

        self.stdout.write(self.style.SUCCESS(
            f'Catálogo inicial de acopio municipal registrado ({len(initial_points)} puntos).'
        ))
