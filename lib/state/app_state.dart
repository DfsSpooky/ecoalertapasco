import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:geolocator/geolocator.dart';
import 'package:latlong2/latlong.dart';
import 'package:http/http.dart' as http;
import 'package:flutter/foundation.dart' show kIsWeb;
import '../models/eco_alert.dart';
import '../services/eco_alert_service.dart';

class AppState extends ChangeNotifier {
  final EcoAlertService _alertService;
  StreamSubscription<List<EcoAlert>>? _subscription;

  List<EcoAlert> _allAlerts = [];
  bool _isLoading = true;

  // Filtros activos (inicializados con todos seleccionados)
  final Set<EcoCategory> _selectedCategories = Set.from(EcoCategory.values);
  final Set<EcoSeverity> _selectedSeverities = Set.from(EcoSeverity.values);
  
  // Filtro de distrito (null significa "Todos")
  EcoDistrict? _selectedDistrict;

  // Filtro de días (0 significa "Todos")
  int _daysFilter = 0;

  // Búsqueda por texto
  String _searchQuery = '';

  // Filtros de Estado
  bool _showUnresolved = true;
  bool _showResolved = true;

  // Estado del Reporte
  bool _isReportMode = false;
  double? _tempLatitude;
  double? _tempLongitude;

  // Estado de Alerta de Emergencia Simulada
  EcoAlert? _activeEmergencyAlert;

  LatLng? _userLocation;

  // Campos de autenticación
  String? _authToken;
  bool _isLoggedInAuthority = false;
  String? _loggedUsername;
  bool _showAuthorityDashboard = false;
  bool _showDismissed = false;
  EcoAlert? _selectedAlert;
  bool _showTransparencyPortal = false;

  String _currentGeocodedAddress = 'Alineando mira...';
  Timer? _geocodeDebounce;

  String get currentGeocodedAddress => _currentGeocodedAddress;

  static final List<Map<String, dynamic>> _geocodePoints = [
    // Yanacancha
    {'name': 'Avenida Los Próceres (Frente al Hospital Regional), Yanacancha', 'lat': -10.6480, 'lng': -76.2490, 'district': EcoDistrict.yanacancha},
    {'name': 'Avenida Los Próceres (Cerca al Terminal Terrestre), Yanacancha', 'lat': -10.6550, 'lng': -76.2410, 'district': EcoDistrict.yanacancha},
    {'name': 'Avenida Daniel Alcides Carrión (Bulevar de Yanacancha), Yanacancha', 'lat': -10.6520, 'lng': -76.2420, 'district': EcoDistrict.yanacancha},
    {'name': 'Avenida Los Incas 480, Yanacancha', 'lat': -10.6565, 'lng': -76.2380, 'district': EcoDistrict.yanacancha},
    {'name': 'Carretera Central (Sector Puente Río San Juan), Yanacancha', 'lat': -10.6420, 'lng': -76.2360, 'district': EcoDistrict.yanacancha},
    {'name': 'Avenida El Minero (Sector Industrial La Esperanza), Yanacancha', 'lat': -10.6590, 'lng': -76.2480, 'district': EcoDistrict.yanacancha},
    {'name': 'Jirón Progreso (Cerca a San Juan), Yanacancha', 'lat': -10.6440, 'lng': -76.2390, 'district': EcoDistrict.yanacancha},
    {'name': 'Avenida Los Incas (Esquina Los Próceres), Yanacancha', 'lat': -10.6500, 'lng': -76.2450, 'district': EcoDistrict.yanacancha},
    {'name': 'Avenida Las Américas (Esquina Av. 6 de Diciembre), Yanacancha', 'lat': -10.6640, 'lng': -76.2550, 'district': EcoDistrict.yanacancha},
    {'name': 'Calle Mariátegui (Cerca a Av. Las Américas), Yanacancha', 'lat': -10.6630, 'lng': -76.2540, 'district': EcoDistrict.yanacancha},
    {'name': 'Jirón Moreno (Cerca a Av. Las Américas), Yanacancha', 'lat': -10.6650, 'lng': -76.2560, 'district': EcoDistrict.yanacancha},
    {'name': 'Jirón Diputación, Yanacancha', 'lat': -10.6700, 'lng': -76.2540, 'district': EcoDistrict.yanacancha},
    {'name': 'Jirón Ramón Castilla, Yanacancha', 'lat': -10.6710, 'lng': -76.2530, 'district': EcoDistrict.yanacancha},
    {'name': 'Avenida Daniel Alcides Carrión (Cerca a la UNDAC), Yanacancha', 'lat': -10.6690, 'lng': -76.2520, 'district': EcoDistrict.yanacancha},
    {'name': 'Avenida El Minero (Entrada a la UNDAC), Yanacancha', 'lat': -10.6715, 'lng': -76.2510, 'district': EcoDistrict.yanacancha},
    {'name': 'Avenida Los Incas (Corte Superior de Justicia), Yanacancha', 'lat': -10.6670, 'lng': -76.2460, 'district': EcoDistrict.yanacancha},

    // Chaupimarca
    {'name': 'Jirón Daniel Alcides Carrión 112 (Plaza Daniel Alcides Carrión), Chaupimarca', 'lat': -10.6781, 'lng': -76.2545, 'district': EcoDistrict.chaupimarca},
    {'name': 'Avenida Bolognesi 302 (Frente a Municipalidad), Chaupimarca', 'lat': -10.6790, 'lng': -76.2550, 'district': EcoDistrict.chaupimarca},
    {'name': 'Jirón Alfonso Ugarte (Ribera Este de Laguna Patarcocha), Chaupimarca', 'lat': -10.6655, 'lng': -76.2525, 'district': EcoDistrict.chaupimarca},
    {'name': 'Jirón San Martín (Ribera Oeste de Laguna Patarcocha), Chaupimarca', 'lat': -10.6680, 'lng': -76.2530, 'district': EcoDistrict.chaupimarca},
    {'name': 'Jirón Hilario Cabrera 215 (Cerca al Mercado Central), Chaupimarca', 'lat': -10.6795, 'lng': -76.2590, 'district': EcoDistrict.chaupimarca},
    {'name': 'Jirón Lampa s/n (Borde del Tajo Abierto), Chaupimarca', 'lat': -10.6720, 'lng': -76.2570, 'district': EcoDistrict.chaupimarca},
    {'name': 'Jirón Circunvalación Túpac Amaru (Borde del Tajo Abierto), Chaupimarca', 'lat': -10.6730, 'lng': -76.2620, 'district': EcoDistrict.chaupimarca},
    {'name': 'Jirón Tarapacá 310, Chaupimarca', 'lat': -10.6750, 'lng': -76.2555, 'district': EcoDistrict.chaupimarca},
    {'name': 'Asentamiento Humano Uliachin Sector Alto, Chaupimarca', 'lat': -10.6860, 'lng': -76.2620, 'district': EcoDistrict.chaupimarca},
    {'name': 'Jirón Junín (Cerca a San Cristóbal), Chaupimarca', 'lat': -10.6770, 'lng': -76.2570, 'district': EcoDistrict.chaupimarca},

    // Simón Bolívar
    {'name': 'Presa de Relaves Quiulacocha, Simón Bolívar', 'lat': -10.7022, 'lng': -76.2871, 'district': EcoDistrict.simonBolivar},
    {'name': 'Presa de Relaves Quiulacocha Sector Sur, Simón Bolívar', 'lat': -10.7040, 'lng': -76.2900, 'district': EcoDistrict.simonBolivar},
    {'name': 'Sector Ocroyoc, Presa de Relaves, Simón Bolívar', 'lat': -10.6935, 'lng': -76.2990, 'district': EcoDistrict.simonBolivar},
    {'name': 'Comunidad de Champamarca Sector Agropecuario, Simón Bolívar', 'lat': -10.7100, 'lng': -76.2730, 'district': EcoDistrict.simonBolivar},
    {'name': 'Planta de Lixiviados Sector San Juan, Simón Bolívar', 'lat': -10.6920, 'lng': -76.2750, 'district': EcoDistrict.simonBolivar},
    {'name': 'Sector Bellavista s/n, Simón Bolívar', 'lat': -10.6970, 'lng': -76.2750, 'district': EcoDistrict.simonBolivar},
    {'name': 'Carretera Central KM 4.5 (Entrada a Quiulacocha), Simón Bolívar', 'lat': -10.6880, 'lng': -76.2720, 'district': EcoDistrict.simonBolivar},
  ];

  AppState(this._alertService) {
    _init();
  }

  void _init() {
    _isLoading = true;
    notifyListeners();

    _subscription = _alertService.watchAlerts().listen((alerts) {
      _allAlerts = alerts.toList(); // Hacer una copia editable
      _isLoading = false;
      notifyListeners();
    });
  }

  // Getters
  List<EcoAlert> get allAlerts => _allAlerts;
  bool get isLoading => _isLoading;
  Set<EcoCategory> get selectedCategories => _selectedCategories;
  Set<EcoSeverity> get selectedSeverities => _selectedSeverities;
  EcoDistrict? get selectedDistrict => _selectedDistrict;
  int get daysFilter => _daysFilter;
  String get searchQuery => _searchQuery;

  bool get isReportMode => _isReportMode;
  double? get tempLatitude => _tempLatitude;
  double? get tempLongitude => _tempLongitude;
  LatLng? get userLocation => _userLocation;
  
  EcoAlert? get activeEmergencyAlert => _activeEmergencyAlert;

  String? get authToken => _authToken;
  bool get isLoggedIn => _authToken != null;
  bool get isLoggedInAuthority => _isLoggedInAuthority;
  String? get loggedUsername => _loggedUsername;

  String get apiBaseUrl {
    const String envUrl = String.fromEnvironment('BACKEND_URL');
    if (envUrl.isNotEmpty) {
      return envUrl.endsWith('/') ? envUrl.substring(0, envUrl.length - 1) : envUrl;
    }
    if (kIsWeb) {
      final host = Uri.base.host;
      if (host.isNotEmpty && host != 'localhost' && host != '127.0.0.1') {
        if (host.contains('ngrok')) {
          return 'http://192.168.1.95:8000';
        }
        return 'http://$host:8000';
      }
    }
    return 'http://localhost:8000';
  }

  bool get showUnresolved => _showUnresolved;
  bool get showResolved => _showResolved;

  bool get showAuthorityDashboard => _showAuthorityDashboard;
  set showAuthorityDashboard(bool val) {
    _showAuthorityDashboard = val;
    if (val) {
      _showTransparencyPortal = false;
    }
    notifyListeners();
  }

  bool get showDismissed => _showDismissed;
  set showDismissed(bool val) {
    _showDismissed = val;
    notifyListeners();
  }

  bool get showTransparencyPortal => _showTransparencyPortal;
  set showTransparencyPortal(bool val) {
    _showTransparencyPortal = val;
    if (val) {
      _showAuthorityDashboard = false;
    }
    notifyListeners();
  }

  EcoAlert? get selectedAlert => _selectedAlert;
  set selectedAlert(EcoAlert? alert) {
    _selectedAlert = alert;
    notifyListeners();
  }

  EcoDistrict get authorityDistrict {
    if (_loggedUsername == null) return EcoDistrict.chaupimarca;
    final username = _loggedUsername!.toLowerCase();
    if (username.contains('yanacancha')) {
      return EcoDistrict.yanacancha;
    } else if (username.contains('simon') || username.contains('bolivar')) {
      return EcoDistrict.simonBolivar;
    }
    return EcoDistrict.chaupimarca; // Por defecto Provincial de Pasco
  }

  // Filtrado reactivo de alertas
  List<EcoAlert> get filteredAlerts {
    return _allAlerts.where((alert) {
      final isResolved = alert.status == 'solved';
      final isDismissed = alert.status == 'dismissed';
      
      // Ocultar descartadas para ciudadanos.
      // Para autoridades, mostrar solo si showDismissed está activo.
      if (isDismissed) {
        if (!_isLoggedInAuthority || !_showDismissed) {
          return false;
        }
      }
      
      // Filtros de estado manuales
      if (isResolved && !_showResolved) return false;
      if (!isResolved && !isDismissed && !_showUnresolved) return false;
      
      // Expiración automática de resueltas (3 días)
      if (isResolved && alert.resolvedAt != null) {
        final limit = DateTime.now().subtract(const Duration(days: 3));
        if (alert.resolvedAt!.isBefore(limit)) {
          return false;
        }
      }

      final categoryMatches = _selectedCategories.contains(alert.category);
      final severityMatches = _selectedSeverities.contains(alert.severity);
      final districtMatches = _selectedDistrict == null || alert.district == _selectedDistrict;
      
      final query = _searchQuery.trim().toLowerCase();
      final queryMatches = query.isEmpty ||
          alert.title.toLowerCase().contains(query) ||
          alert.description.toLowerCase().contains(query);

      final dateMatches = _daysFilter == 0 ||
          alert.createdAt.isAfter(DateTime.now().subtract(Duration(days: _daysFilter)));
          
      return categoryMatches && severityMatches && queryMatches && districtMatches && dateMatches;
    }).toList();
  }

  // Cambiar rango de tiempo
  void setDaysFilter(int days) {
    _daysFilter = days;
    notifyListeners();
  }

  // Cambiar distrito de búsqueda
  void setDistrict(EcoDistrict? district) {
    _selectedDistrict = district;
    notifyListeners();
  }

  // Cambiar texto de búsqueda
  void setSearchQuery(String query) {
    _searchQuery = query;
    notifyListeners();
  }

  void toggleShowUnresolved() {
    _showUnresolved = !_showUnresolved;
    notifyListeners();
  }

  void toggleShowResolved() {
    _showResolved = !_showResolved;
    notifyListeners();
  }

  // Alternar filtros de categorías
  void toggleCategory(EcoCategory category) {
    if (_selectedCategories.contains(category)) {
      _selectedCategories.remove(category);
    } else {
      _selectedCategories.add(category);
    }
    notifyListeners();
  }

  // Alternar filtros de severidades
  void toggleSeverity(EcoSeverity severity) {
    if (_selectedSeverities.contains(severity)) {
      _selectedSeverities.remove(severity);
    } else {
      _selectedSeverities.add(severity);
    }
    notifyListeners();
  }

  // Descartar alerta de emergencia
  void dismissEmergencyAlert() {
    _activeEmergencyAlert = null;
    notifyListeners();
  }

  // Simular alerta crítica de emergencia en Cerro de Pasco
  void simulateEmergency() {
    final emergencies = [
      EcoAlert(
        id: 'emergency_${DateTime.now().millisecondsSinceEpoch}_1',
        title: '🚨 EMERGENCIA CRÍTICA: Derrame de Ácidos',
        description: 'Colapso de válvula de contención provoca escurrimiento masivo de aguas ácidas industriales hacia el Río San Juan.',
        category: EcoCategory.agua,
        severity: EcoSeverity.critico,
        district: EcoDistrict.simonBolivar,
        address: 'Planta de Tratamiento Río San Juan, Simón Bolívar',
        latitude: -10.6920,
        longitude: -76.2750,
        createdAt: DateTime.now(),
        imageUrl: 'derrame_emergencia_san_juan.jpg',
      ),
      EcoAlert(
        id: 'emergency_${DateTime.now().millisecondsSinceEpoch}_2',
        title: '💨 CONTAMINACIÓN CRÍTICA: Humo Negro en La Esperanza',
        description: 'Incendio espontáneo de neumáticos y llantas desechadas en depósito minero levanta densas nubes de carbono inhalable.',
        category: EcoCategory.aire,
        severity: EcoSeverity.critico,
        district: EcoDistrict.yanacancha,
        address: 'Avenida El Minero (Sector Industrial La Esperanza), Yanacancha',
        latitude: -10.6590,
        longitude: -76.2480,
        createdAt: DateTime.now(),
        imageUrl: 'incendio_neumaticos_paragsha.jpg',
      ),
      EcoAlert(
        id: 'emergency_${DateTime.now().millisecondsSinceEpoch}_3',
        title: '💀 COLAPSO MINERO: Desprendimiento de Relaves',
        description: 'Fisura menor en corona de la presa de relaves Quiulacocha activa desprendimiento de sedimentos plomizos en bofedal.',
        category: EcoCategory.mineria,
        severity: EcoSeverity.critico,
        district: EcoDistrict.simonBolivar,
        address: 'Presa de Relaves Quiulacocha Sector Sur, Simón Bolívar',
        latitude: -10.7040,
        longitude: -76.2900,
        createdAt: DateTime.now(),
        imageUrl: 'desprendimiento_sedimentos_quiulacocha.jpg',
      ),
    ];

    // Elegir aleatoriamente
    final index = DateTime.now().millisecondsSinceEpoch % emergencies.length;
    final emergency = emergencies[index];

    // Insertar localmente en el dashboard
    _allAlerts.insert(0, emergency);
    _activeEmergencyAlert = emergency;
    notifyListeners();
  }

  // Obtener ubicación GPS actual del usuario
  Future<LatLng?> getUserLatLng() async {
    try {
      bool serviceEnabled = await Geolocator.isLocationServiceEnabled();
      if (!serviceEnabled) return null;

      LocationPermission permission = await Geolocator.checkPermission();
      if (permission == LocationPermission.denied) {
        permission = await Geolocator.requestPermission();
        if (permission == LocationPermission.denied) return null;
      }
      
      if (permission == LocationPermission.deniedForever) return null;

      final pos = await Geolocator.getCurrentPosition(
        desiredAccuracy: LocationAccuracy.high,
      );
      _userLocation = LatLng(pos.latitude, pos.longitude);
      notifyListeners();
      return _userLocation;
    } catch (e) {
      return null;
    }
  }

  // Deducir distrito basado en coordenadas
  EcoDistrict _deduceDistrict(double lat, double lng) {
    double minDistance = double.infinity;
    EcoDistrict closestDistrict = EcoDistrict.chaupimarca;

    for (final p in _geocodePoints) {
      final pLat = p['lat'] as double;
      final pLng = p['lng'] as double;
      final d = (pLat - lat) * (pLat - lat) + (pLng - lng) * (pLng - lng);
      if (d < minDistance) {
        minDistance = d;
        closestDistrict = p['district'] as EcoDistrict;
      }
    }

    return closestDistrict;
  }

  // Geocodificación inversa simulada para obtener direcciones legibles en Cerro de Pasco
  String reverseGeocode(double lat, double lng) {
    double minDistance = double.infinity;
    String closestName = 'Calle de Cerro de Pasco, Perú';

    for (final p in _geocodePoints) {
      final pLat = p['lat'] as double;
      final pLng = p['lng'] as double;
      final d = (pLat - lat) * (pLat - lat) + (pLng - lng) * (pLng - lng);
      if (d < minDistance) {
        minDistance = d;
        closestName = p['name'] as String;
      }
    }

    return closestName;
  }

  // Acciones de Reporte
  void startReportMode() {
    _isReportMode = true;
    _tempLatitude = -10.6675;
    _tempLongitude = -76.2567;
    _currentGeocodedAddress = 'Alineando mira...';
    _fetchRealAddress(-10.6675, -76.2567);
    notifyListeners();
  }

  void cancelReportMode() {
    _isReportMode = false;
    _tempLatitude = null;
    _tempLongitude = null;
    notifyListeners();
  }

  Future<void> _fetchRealAddress(double lat, double lng) async {
    try {
      final url = Uri.parse('https://nominatim.openstreetmap.org/reverse?format=json&lat=$lat&lon=$lng&zoom=18&addressdetails=1');
      final response = await http.get(url, headers: {
        'User-Agent': 'EcoAlertaApp/1.0.0 (contact: ecoalerta@example.com)'
      });
      
      if (response.statusCode == 200) {
        final data = json.decode(response.body);
        final address = data['address'] as Map<String, dynamic>?;
        if (address != null) {
          final road = address['road'] ?? address['suburb'] ?? address['pedestrian'] ?? address['neighbourhood'] ?? 'Calle Innominada';
          
          // Deducir el distrito escaneando todos los valores de dirección devueltos por OSM
          String districtName = '';
          final addressValuesString = address.values.join(' ').toLowerCase();
          
          if (addressValuesString.contains('yanacancha')) {
            districtName = 'Yanacancha';
          } else if (addressValuesString.contains('simon bolivar') || addressValuesString.contains('simón bolívar')) {
            districtName = 'Simón Bolívar';
          } else if (addressValuesString.contains('chaupimarca')) {
            districtName = 'Chaupimarca';
          } else {
            // Fallback a base de datos de proximidad
            final district = _deduceDistrict(lat, lng);
            if (district == EcoDistrict.yanacancha) districtName = 'Yanacancha';
            if (district == EcoDistrict.simonBolivar) districtName = 'Simón Bolívar';
            if (district == EcoDistrict.chaupimarca) districtName = 'Chaupimarca';
          }
          
          _currentGeocodedAddress = '$road, $districtName';
          notifyListeners();
          return;
        }
      }
      _currentGeocodedAddress = reverseGeocode(lat, lng);
      notifyListeners();
    } catch (e) {
      _currentGeocodedAddress = reverseGeocode(lat, lng);
      notifyListeners();
    }
  }

  void updateTempLocation(double lat, double lng) {
    _tempLatitude = lat;
    _tempLongitude = lng;
    
    _geocodeDebounce?.cancel();
    _geocodeDebounce = Timer(const Duration(milliseconds: 400), () {
      _fetchRealAddress(lat, lng);
    });
  }

  void confirmReportLocation(double lat, double lng) {
    _tempLatitude = lat;
    _tempLongitude = lng;
    _geocodeDebounce?.cancel();
    _fetchRealAddress(lat, lng);
    notifyListeners();
  }

  Future<String> fetchAddressString(double lat, double lng) async {
    try {
      final url = Uri.parse('https://nominatim.openstreetmap.org/reverse?format=json&lat=$lat&lon=$lng&zoom=18&addressdetails=1');
      final response = await http.get(url, headers: {
        'User-Agent': 'EcoAlertaApp/1.0.0 (contact: ecoalerta@example.com)'
      });
      
      if (response.statusCode == 200) {
        final data = json.decode(response.body);
        final address = data['address'] as Map<String, dynamic>?;
        if (address != null) {
          final road = address['road'] ?? address['suburb'] ?? address['pedestrian'] ?? address['neighbourhood'] ?? 'Calle Innominada';
          
          String districtName = '';
          final addressValuesString = address.values.join(' ').toLowerCase();
          
          if (addressValuesString.contains('yanacancha')) {
            districtName = 'Yanacancha';
          } else if (addressValuesString.contains('simon bolivar') || addressValuesString.contains('simón bolívar')) {
            districtName = 'Simón Bolívar';
          } else if (addressValuesString.contains('chaupimarca')) {
            districtName = 'Chaupimarca';
          } else {
            final district = _deduceDistrict(lat, lng);
            if (district == EcoDistrict.yanacancha) districtName = 'Yanacancha';
            if (district == EcoDistrict.simonBolivar) districtName = 'Simón Bolívar';
            if (district == EcoDistrict.chaupimarca) districtName = 'Chaupimarca';
          }
          return '$road, $districtName';
        }
      }
      return reverseGeocode(lat, lng);
    } catch (e) {
      return reverseGeocode(lat, lng);
    }
  }

  Future<bool> createAlert({
    required String title,
    required String description,
    required EcoCategory category,
    required EcoSeverity severity,
    required String address,
    String? imageUrl,
  }) async {
    if (_tempLatitude == null || _tempLongitude == null) return false;

    final deduced = _deduceDistrict(_tempLatitude!, _tempLongitude!);

    final newAlert = EcoAlert(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      title: title,
      description: description,
      category: category,
      severity: severity,
      district: deduced,
      address: address,
      latitude: _tempLatitude!,
      longitude: _tempLongitude!,
      createdAt: DateTime.now(),
      imageUrl: imageUrl,
    );

    // Salir del modo de reporte
    _isReportMode = false;
    _tempLatitude = null;
    _tempLongitude = null;
    notifyListeners();

    try {
      await _alertService.addAlert(newAlert);
      return true;
    } catch (e) {
      return false;
    }
  }

  Future<bool> resolveAlert(String id, String comment, String imageUrl) async {
    try {
      final success = await _alertService.resolveAlert(id, comment, imageUrl);
      return success;
    } catch (e) {
      return false;
    }
  }

  Future<bool> dismissAlert(String id, String comment) async {
    try {
      final success = await _alertService.dismissAlert(id, comment);
      return success;
    } catch (e) {
      return false;
    }
  }

  Future<bool> transferAlert(String id, EcoDistrict newDistrict) async {
    try {
      final success = await _alertService.transferAlert(id, newDistrict);
      return success;
    } catch (e) {
      return false;
    }
  }

  Future<bool> login(String username, String password) async {
    try {
      final response = await http.post(
        Uri.parse('$apiBaseUrl/api/auth/login/'),
        headers: {'Content-Type': 'application/json; charset=UTF-8'},
        body: json.encode({
          'username': username,
          'password': password,
        }),
      );

      if (response.statusCode == 200) {
        final data = json.decode(response.body);
        final token = data['token'] as String?;
        final isStaff = data['is_staff'] as bool? ?? false;
        final name = data['username'] as String?;

        if (token != null) {
          _authToken = token;
          _isLoggedInAuthority = isStaff;
          _loggedUsername = name;
          _alertService.setToken(token);
          notifyListeners();
          return true;
        }
      }
      return false;
    } catch (e) {
      return false;
    }
  }

  Future<bool> register(String username, String email, String password) async {
    try {
      final response = await http.post(
        Uri.parse('$apiBaseUrl/api/auth/register/'),
        headers: {'Content-Type': 'application/json; charset=UTF-8'},
        body: json.encode({
          'username': username,
          'email': email,
          'password': password,
        }),
      );

      if (response.statusCode == 201 || response.statusCode == 200) {
        final data = json.decode(response.body);
        final token = data['token'] as String?;
        final isStaff = data['is_staff'] as bool? ?? false;
        final name = data['username'] as String?;

        if (token != null) {
          _authToken = token;
          _isLoggedInAuthority = isStaff;
          _loggedUsername = name;
          _alertService.setToken(token);
          notifyListeners();
          return true;
        }
      }
      return false;
    } catch (e) {
      return false;
    }
  }

  void logout() {
    _authToken = null;
    _isLoggedInAuthority = false;
    _loggedUsername = null;
    _showAuthorityDashboard = false;
    _showDismissed = false;
    _selectedAlert = null;
    _showTransparencyPortal = false;
    _alertService.setToken(null);
    notifyListeners();
  }

  Future<String?> uploadImage(List<int> bytes, String fileName) async {
    try {
      final request = http.MultipartRequest(
        'POST',
        Uri.parse('$apiBaseUrl/api/upload/'),
      );
      
      request.files.add(
        http.MultipartFile.fromBytes(
          'image',
          bytes,
          filename: fileName,
        ),
      );

      final streamedResponse = await request.send();
      final response = await http.Response.fromStream(streamedResponse);

      if (response.statusCode == 200) {
        final data = json.decode(response.body);
        return data['url'] as String?;
      }
      return null;
    } catch (e) {
      return null;
    }
  }

  @override
  void dispose() {
    _subscription?.cancel();
    super.dispose();
  }
}
