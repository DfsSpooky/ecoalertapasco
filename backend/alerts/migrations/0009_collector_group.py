from django.db import migrations

COLLECTOR_GROUP_NAME = 'recolectores'


def create_collector_group(apps, schema_editor):
    Group = apps.get_model('auth', 'Group')
    Group.objects.get_or_create(name=COLLECTOR_GROUP_NAME)


def remove_collector_group(apps, schema_editor):
    Group = apps.get_model('auth', 'Group')
    Group.objects.filter(name=COLLECTOR_GROUP_NAME).delete()


class Migration(migrations.Migration):

    dependencies = [
        ('alerts', '0008_fleet_routes'),
        ('auth', '0012_alter_user_first_name_max_length'),
    ]

    operations = [
        migrations.RunPython(create_collector_group, remove_collector_group),
    ]
