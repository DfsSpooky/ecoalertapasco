from rest_framework import serializers
from .models import Alert

class AlertSerializer(serializers.ModelSerializer):
    class Meta:
        model = Alert
        fields = '__all__'
        read_only_fields = ('created_at',)


from .models import CollectorRoutePoint, MunicipalDump

class CollectorRoutePointSerializer(serializers.ModelSerializer):
    class Meta:
        model = CollectorRoutePoint
        fields = '__all__'


class MunicipalDumpSerializer(serializers.ModelSerializer):
    class Meta:
        model = MunicipalDump
        fields = '__all__'


from .models import SiteConfiguration

class SiteConfigurationSerializer(serializers.ModelSerializer):
    admin_url = serializers.SerializerMethodField()

    class Meta:
        model = SiteConfiguration
        fields = '__all__'

    def get_admin_url(self, obj):
        import os
        request = self.context.get('request')
        admin_path = os.environ.get('DJANGO_ADMIN_PATH', 'ecoalerta-secret-admin-portal').strip('/')
        if admin_path:
            path = f"/{admin_path}/"
        else:
            path = "/admin/"
        
        if request:
            return request.build_absolute_uri(path)
        return path

