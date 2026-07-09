import 'package:supabase_flutter/supabase_flutter.dart';
import '../models/eco_alert.dart';
import 'eco_alert_service.dart';

class SupabaseAlertService implements EcoAlertService {
  final _client = Supabase.instance.client;

  @override
  Stream<List<EcoAlert>> watchAlerts() {
    // Escuchar el flujo de datos en tiempo real de la tabla 'eco_alerts'
    return _client
        .from('eco_alerts')
        .stream(primaryKey: ['id'])
        .order('created_at', ascending: false)
        .map((data) {
          return data.map((json) => EcoAlert.fromJson(json)).toList();
        });
  }

  @override
  Future<void> addAlert(EcoAlert alert) async {
    // Insertar un nuevo registro. location se envía en formato WKT
    await _client.from('eco_alerts').insert({
      'title': alert.title,
      'description': alert.description,
      'category': alert.category.toString().split('.').last,
      'severity': alert.severity.toString().split('.').last,
      'district': alert.district.toString().split('.').last,
      'location': 'SRID=4326;POINT(${alert.longitude} ${alert.latitude})',
      'image_url': alert.imageUrl,
    });
  }

  @override
  Future<bool> resolveAlert(String id, String comment, String imageUrl) async {
    try {
      await _client.from('eco_alerts').update({
        'status': 'solved',
        'resolution_comment': comment,
        'resolution_image_url': imageUrl,
        'resolved_at': DateTime.now().toIso8601String(),
      }).eq('id', id);
      return true;
    } catch (e) {
      return false;
    }
  }

  @override
  Future<bool> dismissAlert(String id, String comment) async {
    try {
      await _client.from('eco_alerts').update({
        'status': 'dismissed',
        'resolution_comment': comment,
      }).eq('id', id);
      return true;
    } catch (e) {
      return false;
    }
  }

  @override
  Future<bool> transferAlert(String id, EcoDistrict newDistrict) async {
    try {
      await _client.from('eco_alerts').update({
        'district': newDistrict.toString().split('.').last,
      }).eq('id', id);
      return true;
    } catch (e) {
      return false;
    }
  }

  @override
  void setToken(String? token) {}
}
