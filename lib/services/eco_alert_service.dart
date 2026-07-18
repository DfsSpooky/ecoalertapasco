import 'package:latlong2/latlong.dart';
import '../models/eco_alert.dart';
import '../models/waste_point.dart';

abstract class EcoAlertService {
  Stream<List<EcoAlert>> watchAlerts();
  Future<void> addAlert(EcoAlert alert);
  Future<bool> resolveAlert(String id, String comment, String imageUrl);
  Future<bool> dismissAlert(String id, String comment);
  Future<bool> transferAlert(String id, EcoDistrict newDistrict);
  void setToken(String? token);
  Future<List<WastePoint>> fetchWastePoints();
  Future<List<LatLng>> fetchCollectorRoute();
}
