import '../models/eco_alert.dart';
import '../models/waste_point.dart';

abstract class EcoAlertService {
  Stream<List<EcoAlert>> watchAlerts();
  Future<void> addAlert(EcoAlert alert);
  Future<bool> resolveAlert(String id, String comment, String imageUrl);
  Future<bool> dismissAlert(String id, String comment);
  Future<bool> transferAlert(String id, EcoDistrict newDistrict);
  Future<List<WastePoint>> getWastePoints();
  void setToken(String? token);

  Future<bool> confirmAlert(String id);
  Future<Map<String, dynamic>> analyzeEvidenceAI({List<int>? imageBytes, String? imageUrl, String? fileName});
  Future<Map<String, dynamic>> getSystemSettings();
  Future<bool> saveGeminiApiKey(String apiKey, {String? model, bool? enabled});
  Future<Map<String, dynamic>> testGeminiApiKey({String? model, String? apiKey});
  String get exportCsvUrl;
}

