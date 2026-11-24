from django.db import migrations

SERENAZGO_GROUP_NAME = 'serenazgo'


def create_serenazgo_group(apps, schema_editor):
    Group = apps.get_model('auth', 'Group')
    Group.objects.get_or_create(name=SERENAZGO_GROUP_NAME)


def remove_serenazgo_group(apps, schema_editor):
    Group = apps.get_model('auth', 'Group')
    Group.objects.filter(name=SERENAZGO_GROUP_NAME).delete()


class Migration(migrations.Migration):

    dependencies = [
        ('alerts', '0011_citizen_security_alerts'),
        ('auth', '0012_alter_user_first_name_max_length'),
    ]

    operations = [
        migrations.RunPython(create_serenazgo_group, remove_serenazgo_group),
    ]
