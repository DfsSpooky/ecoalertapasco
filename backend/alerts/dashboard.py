from alerts.models import Alert, MunicipalDump

def dashboard_callback(request, context):
    # Total de alertas y conteo de estados
    context.update({
        'total_alerts': Alert.objects.count(),
        'critical_alerts': Alert.objects.filter(severity='critico', status='pending').count(),
        'total_dumps': MunicipalDump.objects.filter(type='municipalDump').count(),
        'solved_alerts': Alert.objects.filter(status='solved').count(),
        
        # Conteo por categorías
        'mineria_count': Alert.objects.filter(category='mineria').count(),
        'basura_count': Alert.objects.filter(category='basura').count(),
        'agua_count': Alert.objects.filter(category='agua').count(),
        'aire_count': Alert.objects.filter(category='aire').count(),
        
        # Conteo por distritos
        'yanacancha_count': Alert.objects.filter(district='yanacancha').count(),
        'chaupimarca_count': Alert.objects.filter(district='chaupimarca').count(),
        'simon_count': Alert.objects.filter(district='simonBolivar').count(),
        
        # Últimas 5 alertas reportadas
        'recent_alerts': Alert.objects.all().order_by('-created_at')[:5],
    })
    return context
