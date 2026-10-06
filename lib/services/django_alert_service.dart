import 'dart:async';
import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:flutter/foundation.dart' show kIsWeb;
import '../models/eco_alert.dart';
import '../models/waste_point.dart';
import 'eco_alert_service.dart';

class DjangoAlertService implements EcoAlertService {
  String get baseUrl {
    const String envUrl = String.fromEnvironment('BACKEND_URL');
    if (envUrl.isNotEmpty) {
      final formattedUrl = envUrl.endsWith('/') ? envUrl : '$envUrl/';
      return '${formattedUrl}api/alerts/';
    }
    if (kIsWeb) {
      final host = Uri.base.host;
      if (host.isNotEmpty && host != 'localhost' && host != '127.0.0.1') {
        if (host.contains('ngrok')) {
          return 'http://192.168.1.95:8000/api/alerts/';
        }
        return 'http://$host:8000/api/alerts/';
      }
    }
    return 'http://localhost:8000/api/alerts/';
  }

  String get wastePointsUrl {
    const String envUrl = String.fromEnvironment('BACKEND_URL');
    if (envUrl.isNotEmpty) {
      final formattedUrl = envUrl.endsWith('/') ? envUrl : '$envUrl/';
      return '${formattedUrl}api/waste-points/';
    }
    if (kIsWeb) {
      final host = Uri.base.host;
      if (host.isNotEmpty && host != 'localhost' && host != '127.0.0.1') {
        if (host.contains('ngrok')) {
          return 'http://192.168.1.95:8000/api/waste-points/';
        }
        return 'http://$host:8000/api/waste-points/';
      }
    }
    return 'http://localhost:8000/api/waste-points/';
  }

  final _controller = StreamController<List<EcoAlert>>.broadcast();
  Timer? _pollTimer;
  List<EcoAlert> _cachedAlerts = [];
  String? _token;

  DjangoAlertService() {
    _startPolling();
  }

  void _startPolling() {
    _fetchAndEmit();
    _pollTimer = Timer.periodic(const Duration(seconds: 4), (timer) {
      _fetchAndEmit();
    });
  }

  Future<void> _fetchAndEmit() async {
    try {
      final response = await http.get(Uri.parse(baseUrl));
      if (response.statusCode == 200) {
        final List<dynamic> data = json.decode(utf8.decode(response.bodyBytes));
        final alerts = data.map((item) => EcoAlert.fromJson(item)).toList();
        _cachedAlerts = alerts;
        _controller.add(List.unmodifiable(_cachedAlerts));
      }
    } catch (e) {
      // Emitir el caché anterior en caso de desconexión
      _controller.add(List.unmodifiable(_cachedAlerts));
    }
  }

  @override
  Stream<List<EcoAlert>> watchAlerts() {
    return _controller.stream;
  }

  @override
  void setToken(String? token) {
    _token = token;
  }

  @override
  Future<List<WastePoint>> getWastePoints() async {
    try {
      final response = await http.get(Uri.parse(wastePointsUrl));
      if (response.statusCode == 200) {
        final List<dynamic> data = json.decode(utf8.decode(response.bodyBytes));
        return data.map((item) => WastePoint.fromJson(item)).toList();
      }
      return [];
    } catch (e) {
      return [];
    }
  }

  @override
  Future<void> addAlert(EcoAlert alert) async {
    if (_token == null) {
      throw Exception('Debes iniciar sesión para reportar una alerta ambiental.');
    }
    final headers = {
      'Content-Type': 'application/json; charset=UTF-8',
      'Authorization': 'Token $_token',
    };
    final response = await http.post(
      Uri.parse(baseUrl),
      headers: headers,
      body: json.encode(alert.toJson()),
    );
    if (response.statusCode == 201 || response.statusCode == 200) {
      _fetchAndEmit();
    } else {
      String errorMsg = 'Error al registrar la alerta (${response.statusCode})';
      try {
        final errData = json.decode(utf8.decode(response.bodyBytes));
        if (errData is Map) {
          if (errData.containsKey('detail')) {
            errorMsg = errData['detail'].toString();
          } else if (errData.containsKey('error')) {
            errorMsg = errData['error'].toString();
          } else if (errData.values.isNotEmpty) {
            final val = errData.values.first;
            errorMsg = val is List ? val.first.toString() : val.toString();
          }
        }
      } catch (_) {}
      if (response.statusCode == 429) {
        errorMsg = 'Has alcanzado el límite de reportes por hora (máximo 10). Por favor espera un momento.';
      }
      throw Exception(errorMsg);
    }
  }

  @override
  Future<bool> resolveAlert(String id, String comment, String imageUrl) async {
    try {
      final headers = {'Content-Type': 'application/json; charset=UTF-8'};
      if (_token != null) {
        headers['Authorization'] = 'Token $_token';
      }
      final response = await http.patch(
        Uri.parse('$baseUrl$id/'),
        headers: headers,
        body: json.encode({
          'status': 'solved',
          'resolution_comment': comment,
          'resolution_image_url': imageUrl,
        }),
      );
      if (response.statusCode == 200) {
        _fetchAndEmit();
        return true;
      }
      return false;
    } catch (e) {
      return false;
    }
  }

  @override
  Future<bool> dismissAlert(String id, String comment) async {
    try {
      final headers = {'Content-Type': 'application/json; charset=UTF-8'};
      if (_token != null) {
        headers['Authorization'] = 'Token $_token';
      }
      final response = await http.patch(
        Uri.parse('$baseUrl$id/'),
        headers: headers,
        body: json.encode({
          'status': 'dismissed',
          'resolution_comment': comment,
        }),
      );
      if (response.statusCode == 200) {
        _fetchAndEmit();
        return true;
      }
      return false;
    } catch (e) {
      return false;
    }
  }

  @override
  Future<bool> transferAlert(String id, EcoDistrict newDistrict) async {
    try {
      final headers = {'Content-Type': 'application/json; charset=UTF-8'};
      if (_token != null) {
        headers['Authorization'] = 'Token $_token';
      }
      final response = await http.patch(
        Uri.parse('$baseUrl$id/'),
        headers: headers,
        body: json.encode({
          'district': newDistrict.toString().split('.').last,
        }),
      );
      if (response.statusCode == 200) {
        _fetchAndEmit();
        return true;
      }
      return false;
    } catch (e) {
      return false;
    }
  }

  String get rootApiUrl {
    final alertsUrl = baseUrl;
    if (alertsUrl.contains('/api/alerts/')) {
      return alertsUrl.replaceAll('/api/alerts/', '/api/');
    }
    return alertsUrl.endsWith('/') ? alertsUrl : '$alertsUrl/';
  }

  @override
  String get exportCsvUrl => '${baseUrl}export_csv/';

  @override
  Future<bool> confirmAlert(String id) async {
    if (_token == null) {
      throw Exception('Debes iniciar sesión para confirmar este reporte.');
    }
    try {
      final headers = {
        'Content-Type': 'application/json; charset=UTF-8',
        'Authorization': 'Token $_token',
      };
      final response = await http.post(
        Uri.parse('$baseUrl$id/confirm/'),
        headers: headers,
      );
      if (response.statusCode == 200) {
        _fetchAndEmit();
        return true;
      }
      return false;
    } catch (e) {
      return false;
    }
  }

  @override
  Future<Map<String, dynamic>> analyzeEvidenceAI({
    List<int>? imageBytes,
    String? imageUrl,
    String? fileName,
  }) async {
    if (_token == null) {
      throw Exception('Debes iniciar sesión para solicitar análisis con IA.');
    }
    final analyzeUrl = '${rootApiUrl}ai/analyze-evidence/';
    // Preferimos imageUrl si ya fue subida y comprimida a WebP en el backend (payload liviano y veloz)
    if (imageUrl != null && imageUrl.isNotEmpty) {
      final response = await http.post(
        Uri.parse(analyzeUrl),
        headers: {
          'Content-Type': 'application/json; charset=UTF-8',
          'Authorization': 'Token $_token',
        },
        body: json.encode({'image_url': imageUrl}),
      );
      final data = json.decode(utf8.decode(response.bodyBytes));
      if (response.statusCode == 200 && data is Map<String, dynamic>) {
        return data;
      }
      throw Exception(data['error'] ?? 'Error al analizar evidencia con Gemini.');
    } else if (imageBytes != null && imageBytes.isNotEmpty) {
      final request = http.MultipartRequest('POST', Uri.parse(analyzeUrl));
      request.headers['Authorization'] = 'Token $_token';
      request.files.add(
        http.MultipartFile.fromBytes(
          'image',
          imageBytes,
          filename: fileName ?? 'evidence.jpg',
        ),
      );
      final streamedResponse = await request.send();
      final response = await http.Response.fromStream(streamedResponse);
      final data = json.decode(utf8.decode(response.bodyBytes));
      if (response.statusCode == 200 && data is Map<String, dynamic>) {
        return data;
      }
      throw Exception(data['error'] ?? 'Error al analizar evidencia con Gemini.');
    }
    throw Exception('No se proporcionó imagen para analizar.');
  }

  @override
  Future<Map<String, dynamic>> getSystemSettings() async {
    try {
      final headers = <String, String>{};
      if (_token != null) {
        headers['Authorization'] = 'Token $_token';
      }
      final response = await http.get(
        Uri.parse('${rootApiUrl}system/settings/'),
        headers: headers,
      );
      if (response.statusCode == 200) {
        return json.decode(utf8.decode(response.bodyBytes));
      }
      return {'has_gemini_key': false, 'masked_key': ''};
    } catch (e) {
      return {'has_gemini_key': false, 'masked_key': ''};
    }
  }

  @override
  Future<bool> saveGeminiApiKey(String apiKey, {String? model, bool? enabled}) async {
    if (_token == null) {
      throw Exception('Solo administradores pueden configurar la clave de Gemini.');
    }
    final Map<String, dynamic> body = {};
    if (apiKey.isNotEmpty) {
      body['gemini_api_key'] = apiKey;
    }
    if (model != null && model.isNotEmpty) {
      body['gemini_model'] = model;
    }
    if (enabled != null) {
      body['gemini_enabled'] = enabled;
    }
    final response = await http.post(
      Uri.parse('${rootApiUrl}system/settings/'),
      headers: {
        'Content-Type': 'application/json; charset=UTF-8',
        'Authorization': 'Token $_token',
      },
      body: json.encode(body),
    );
    if (response.statusCode == 200) {
      return true;
    }
    final data = json.decode(utf8.decode(response.bodyBytes));
    throw Exception(data['error'] ?? 'Error al guardar clave de Gemini.');
  }

  @override
  Future<Map<String, dynamic>> testGeminiApiKey({String? model, String? apiKey}) async {
    if (_token == null) {
      throw Exception('Debes ser administrador para realizar el handshake.');
    }
    final Map<String, dynamic> body = {};
    if (model != null && model.isNotEmpty) {
      body['model'] = model;
    }
    if (apiKey != null && apiKey.isNotEmpty) {
      body['api_key'] = apiKey;
    }
    final response = await http.post(
      Uri.parse('${rootApiUrl}system/test-gemini/'),
      headers: {
        'Content-Type': 'application/json; charset=UTF-8',
        'Authorization': 'Token $_token',
      },
      body: json.encode(body),
    );
    final data = json.decode(utf8.decode(response.bodyBytes));
    if (response.statusCode == 200) {
      return data;
    }
    throw Exception(data['error'] ?? 'Error en el handshake con Gemini.');
  }

  void dispose() {
    _pollTimer?.cancel();
    _controller.close();
  }
}

