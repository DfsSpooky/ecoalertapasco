from django.core.management.base import BaseCommand
from django.contrib.auth import get_user_model
import os

class Command(BaseCommand):
    help = "Crea o actualiza el usuario administrador principal de EcoAlerta con privilegios totales."

    def add_arguments(self, parser):
        parser.add_argument(
            '--username',
            type=str,
            default=os.environ.get('ADMIN_USERNAME', 'admin_pasco'),
            help='Nombre de usuario del administrador',
        )
        parser.add_argument(
            '--password',
            type=str,
            default=os.environ.get('ADMIN_PASSWORD', 'EcoPasco2026!#Secure'),
            help='Contraseña segura del administrador',
        )
        parser.add_argument(
            '--email',
            type=str,
            default=os.environ.get('ADMIN_EMAIL', 'admin@ecoalerta.duckdns.org'),
            help='Correo electrónico del administrador',
        )

    def handle(self, *args, **options):
        username = options['username'].strip()
        password = options['password']
        email = options['email'].strip()

        User = get_user_model()
        user, created = User.objects.get_or_create(
            username=username,
            defaults={
                'email': email,
                'is_staff': True,
                'is_superuser': True,
                'is_active': True,
            }
        )

        user.set_password(password)
        user.is_staff = True
        user.is_superuser = True
        user.is_active = True
        user.email = email
        user.save()

        action = "creado exitosamente" if created else "actualizado exitosamente"
        self.stdout.write(self.style.SUCCESS(
            f"Usuario administrador '{username}' {action} con todos los privilegios."
        ))
