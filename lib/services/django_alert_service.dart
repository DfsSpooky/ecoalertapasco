import 'dart:async';
import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:latlong2/latlong.dart';
import '../models/eco_alert.dart';
import '../models/waste_point.dart';
import 'eco_alert_service.dart';

class DjangoAlertService implements EcoAlertService {
  String get baseApiUrl {
    const String envUrl = String.fromEnvironment('BACKEND_URL');
    if (envUrl.isNotEmpty) {
      final formattedUrl = envUrl.endsWith('/') ? envUrl : '$envUrl/';
      return '${formattedUrl}api/';
    }
    if (kIsWeb) {
      final host = Uri.base.host;
      if (host.isNotEmpty && host != 'localhost' && host != '127.0.0.1') {
        if (host.contains('ngrok')) {
          return 'http://192.168.1.95:8000/api/';
        }
        return 'http://$host:8000/api/';
      }
    }
    return 'http://localhost:8000/api/';
  }

  String get baseUrl => '${baseApiUrl}alerts/';
  final _controller = StreamController<List<EcoAlert>>.broadcast();
  Timer? _pollTimer;
  List<EcoAlert> _cachedAlerts = [];
  String? _token;

  DjangoAlertService() {
    _initConnection();
  }

  void _initConnection() {
    _fetchAndEmit();
    _startSse();
  }

  // Cliente SSE multiplataforma basado en streams HTTP
  Future<void> _startSse() async {
    final client = http.Client();
    final request = http.Request('GET', Uri.parse('${baseUrl}sse/'));
    request.headers['Accept'] = 'text/event-stream';
    request.headers['Cache-Control'] = 'no-cache';

    try {
      final response = await client.send(request);
      if (response.statusCode == 200) {
        // SSE Conectado con éxito: escuchar flujo de líneas decodificadas por renglón
        utf8.decoder
            .bind(response.stream)
            .transform(const LineSplitter())
            .listen((line) {
          if (line.contains('data: refresh')) {
            _fetchAndEmit();
          }
        }, onError: (e) {
          client.close();
          _startPolling();
        }, onDone: () {
          client.close();
          _startPolling();
        });
      } else {
        client.close();
        _startPolling();
      }
    } catch (e) {
      client.close();
      _startPolling();
    }
  }

  void _startPolling() {
    if (_pollTimer != null && _pollTimer!.isActive) return;
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
  Future<void> addAlert(EcoAlert alert) async {
    try {
      final headers = {'Content-Type': 'application/json; charset=UTF-8'};
      if (_token != null) {
        headers['Authorization'] = 'Token $_token';
      }
      final response = await http.post(
        Uri.parse(baseUrl),
        headers: headers,
        body: json.encode(alert.toJson()),
      );
      if (response.statusCode == 201 || response.statusCode == 200) {
        _fetchAndEmit();
      }
    } catch (e) {
      // Ignorar fallas silenciosamente o dejar que resuelva el flujo principal
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

  void dispose() {
    _pollTimer?.cancel();
    _controller.close();
  }

  @override
  Future<List<WastePoint>> fetchWastePoints() async {
    try {
      final response = await http.get(Uri.parse('${baseApiUrl}municipal-dumps/'));
      if (response.statusCode == 200) {
        final List<dynamic> data = json.decode(utf8.decode(response.bodyBytes));
        return data.map((item) => WastePoint.fromJson(item)).toList();
      }
    } catch (e) {
      // fallback silencioso
    }
    return [];
  }

  @override
  Future<List<LatLng>> fetchCollectorRoute() async {
    try {
      final response = await http.get(Uri.parse('${baseApiUrl}collector-route/'));
      if (response.statusCode == 200) {
        final List<dynamic> data = json.decode(utf8.decode(response.bodyBytes));
        return data.map((item) {
          return LatLng(
            (item['latitude'] as num).toDouble(),
            (item['longitude'] as num).toDouble(),
          );
        }).toList();
      }
    } catch (e) {
      // fallback silencioso
    }
    return [];
  }

  @override
  Future<Map<String, dynamic>> fetchSiteSettings() async {
    try {
      final response = await http.get(Uri.parse('${baseApiUrl}site-settings/'));
      if (response.statusCode == 200) {
        return json.decode(utf8.decode(response.bodyBytes)) as Map<String, dynamic>;
      }
    } catch (e) {
      // fallback silencioso
    }
    return {};
  }
}
