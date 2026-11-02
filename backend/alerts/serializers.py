from rest_framework import serializers
from .models import Alert, WastePoint, SystemConfig

class AlertSerializer(serializers.ModelSerializer):
    reporter_name = serializers.CharField(source='reported_by.username', read_only=True, default='Ciudadano')
    has_user_confirmed = serializers.SerializerMethodField()

    class Meta:
        model = Alert
        exclude = ('confirmed_by',)
        read_only_fields = ('created_at', 'history_log', 'reported_by', 'resolved_at', 'confirmations_count')

    def get_has_user_confirmed(self, obj):
        confirmed_ids = self.context.get('confirmed_alert_ids')
        if confirmed_ids is not None:
            return obj.id in confirmed_ids
        request = self.context.get('request')
        if request and hasattr(request, 'user') and request.user.is_authenticated:
            return obj.confirmed_by.filter(id=request.user.id).exists()
        return False


class WastePointSerializer(serializers.ModelSerializer):
    type_display = serializers.CharField(source='get_type_display', read_only=True)
    fill_level_display = serializers.CharField(source='get_fill_level_display', read_only=True)

    class Meta:
        model = WastePoint
        fields = '__all__'
