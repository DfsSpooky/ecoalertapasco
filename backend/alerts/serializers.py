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

