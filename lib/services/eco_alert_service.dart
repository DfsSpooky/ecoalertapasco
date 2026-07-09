import '../models/eco_alert.dart';

abstract class EcoAlertService {
  Stream<List<EcoAlert>> watchAlerts();
  Future<void> addAlert(EcoAlert alert);
  Future<bool> resolveAlert(String id, String comment, String imageUrl);
  Future<bool> dismissAlert(String id, String comment);
  Future<bool> transferAlert(String id, EcoDistrict newDistrict);
  void setToken(String? token);
}
