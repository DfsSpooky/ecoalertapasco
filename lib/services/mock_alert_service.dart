import 'dart:async';
import 'package:latlong2/latlong.dart';
import '../models/eco_alert.dart';
import '../models/waste_point.dart';
import 'eco_alert_service.dart';

class MockAlertService implements EcoAlertService {
  final List<EcoAlert> _alerts = [
    EcoAlert(
      id: '1',
      title: 'Relaves Mineros de Quiulacocha',
      description: 'Filtración y arrastre de sedimentos ácidos con metales pesados desde la desmontera hacia bofedales locales.',
      category: EcoCategory.mineria,
      severity: EcoSeverity.critico,
      district: EcoDistrict.simonBolivar,
      address: 'Presa de Relaves Quiulacocha, Simón Bolívar',
      latitude: -10.7022,
      longitude: -76.2871,
      createdAt: DateTime.now().subtract(const Duration(days: 2)),
      imageUrl: 'relaves_quiulacocha_vista_borde.jpg',
    ),
    EcoAlert(
      id: '2',
      title: 'Polvo de Tajo Abierto Raul Rojas',
      description: 'Presencia de partículas de polvo en suspensión que provienen del movimiento de tierras del tajo abierto central.',
      category: EcoCategory.mineria,
      severity: EcoSeverity.critico,
      district: EcoDistrict.chaupimarca,
      address: 'Jirón Lampa s/n (Borde del Tajo Abierto), Chaupimarca',
      latitude: -10.6720,
      longitude: -76.2570,
      createdAt: DateTime.now().subtract(const Duration(days: 5)),
    ),
    EcoAlert(
      id: '3',
      title: 'Plomo en Suelo de Recreo Escolar',
      description: 'Medición de concentración de plomo excede los límites permisibles en áreas verdes contiguas a la escuela de Chaupimarca.',
      category: EcoCategory.mineria,
      severity: EcoSeverity.critico,
      district: EcoDistrict.chaupimarca,
      address: 'Jirón Daniel Alcides Carrión 112 (Plaza Carrión), Chaupimarca',
      latitude: -10.6781,
      longitude: -76.2545,
      createdAt: DateTime.now().subtract(const Duration(days: 12)),
    ),
    EcoAlert(
      id: '4',
      title: 'Filtración de Depósito Ocroyoc',
      description: 'Humedad con coloración anaranjada detectada en el talud exterior del depósito de relaves, indicando posibles microfiltraciones.',
      category: EcoCategory.mineria,
      severity: EcoSeverity.medio,
      district: EcoDistrict.simonBolivar,
      address: 'Sector Ocroyoc, Presa de Relaves, Simón Bolívar',
      latitude: -10.6935,
      longitude: -76.2990,
      createdAt: DateTime.now().subtract(const Duration(days: 8)),
    ),
    EcoAlert(
      id: '5',
      title: 'Polvareda en Zona de Carga Paragsha',
      description: 'Camiones de transporte de mineral transitan sin lona de protección regulada, levantando polvo con alto contenido metálico.',
      category: EcoCategory.mineria,
      severity: EcoSeverity.critico,
      district: EcoDistrict.yanacancha,
      address: 'Avenida El Minero s/n (Zona de Carga Paragsha), Yanacancha',
      latitude: -10.6550,
      longitude: -76.2450,
      createdAt: DateTime.now().subtract(const Duration(days: 1)),
    ),
    EcoAlert(
      id: '6',
      title: 'Aguas Ácidas en Laguna Patarcocha',
      description: 'Coloración verdosa inusual y emanación de gases sulfhídricos debido al vertido de aguas residuales y drenajes ácidos urbanos.',
      category: EcoCategory.agua,
      severity: EcoSeverity.critico,
      district: EcoDistrict.chaupimarca,
      address: 'Jirón Alfonso Ugarte (Ribera Este de Laguna Patarcocha), Chaupimarca',
      latitude: -10.6655,
      longitude: -76.2525,
      createdAt: DateTime.now().subtract(const Duration(days: 3)),
      imageUrl: 'laguna_patarcocha_espuma_verde.jpg',
    ),
    EcoAlert(
      id: '7',
      title: 'Basural Acumulado en Mercado Chaupimarca',
      description: 'Foco infeccioso por acumulación de residuos sólidos orgánicos sin recoger en la vía pública por más de 4 days.',
      category: EcoCategory.basura,
      severity: EcoSeverity.medio,
      district: EcoDistrict.chaupimarca,
      address: 'Jirón Hilario Cabrera 215 (Cerca al Mercado Central), Chaupimarca',
      latitude: -10.6795,
      longitude: -76.2590,
      createdAt: DateTime.now().subtract(const Duration(days: 4)),
    ),
    EcoAlert(
      id: '8',
      title: 'Humo Blanco de Chimenea en Yanacancha',
      description: 'Emanación de gases con olor irritante proveniente del sector industrial metalmecánico durante la madrugada.',
      category: EcoCategory.aire,
      severity: EcoSeverity.medio,
      district: EcoDistrict.yanacancha,
      address: 'Avenida Los Próceres 201 (Frente al Hospital Regional), Yanacancha',
      latitude: -10.6480,
      longitude: -76.2490,
      createdAt: DateTime.now().subtract(const Duration(days: 6)),
    ),
    EcoAlert(
      id: '9',
      title: 'Residuos de Demolición en Río San Juan',
      description: 'Desmonte y materiales de concreto arrojados en las orillas del río, obstaculizando el cauce natural del agua.',
      category: EcoCategory.basura,
      severity: EcoSeverity.bajo,
      district: EcoDistrict.yanacancha,
      address: 'Carretera Central (Sector Puente Río San Juan), Yanacancha',
      latitude: -10.6420,
      longitude: -76.2360,
      createdAt: DateTime.now().subtract(const Duration(days: 15)),
    ),
    EcoAlert(
      id: '10',
      title: 'Quema de Pastizales en Cerro Uliachin',
      description: 'Incendio provocado de cobertura vegetal que genera densas cortinas de humo que cubren el sector sur de la ciudad.',
      category: EcoCategory.aire,
      severity: EcoSeverity.critico,
      district: EcoDistrict.chaupimarca,
      address: 'Asentamiento Humano Uliachin Sector Alto, Chaupimarca',
      latitude: -10.6860,
      longitude: -76.2620,
      createdAt: DateTime.now().subtract(const Duration(days: 7)),
    ),
    EcoAlert(
      id: '11',
      title: 'Puntos Críticos de Plástico en Bulevar',
      description: 'Contenedores públicos desbordados con botellas y envoltorios plásticos dispersados por el viento en bulevar principal.',
      category: EcoCategory.basura,
      severity: EcoSeverity.bajo,
      district: EcoDistrict.yanacancha,
      address: 'Avenida Daniel Alcides Carrión 405 (Bulevar de Yanacancha), Yanacancha',
      latitude: -10.6520,
      longitude: -76.2420,
      createdAt: DateTime.now().subtract(const Duration(days: 10)),
    ),
    EcoAlert(
      id: '12',
      title: 'Descarga Inadecuada de Lubricantes',
      description: 'Taller mecánico informal vierte aceites usados directamente a la red de alcantarillado público sin tratamiento previo.',
      category: EcoCategory.agua,
      severity: EcoSeverity.medio,
      district: EcoDistrict.yanacancha,
      address: 'Avenida Los Incas 480, Yanacancha',
      latitude: -10.6565,
      longitude: -76.2380,
      createdAt: DateTime.now().subtract(const Duration(days: 9)),
    ),
    EcoAlert(
      id: '13',
      title: 'Emisiones de Monóxido por Parque Automotor',
      description: 'Congestión vehicular pesada en hora punta genera altas concentraciones de smog en las estrechas calles céntricas.',
      category: EcoCategory.aire,
      severity: EcoSeverity.bajo,
      district: EcoDistrict.chaupimarca,
      address: 'Jirón Tarapacá 310, Chaupimarca',
      latitude: -10.6750,
      longitude: -76.2555,
      createdAt: DateTime.now().subtract(const Duration(days: 11)),
    ),
    EcoAlert(
      id: '14',
      title: 'Erosión de Suelos por Relave Antiguo',
      description: 'Terrenos baldíos con costras de sal mineralizada y nula vegetación debido a la contaminación histórica de suelo en Champamarca.',
      category: EcoCategory.mineria,
      severity: EcoSeverity.medio,
      district: EcoDistrict.simonBolivar,
      address: 'Comunidad de Champamarca Sector Agropecuario, Simón Bolívar',
      latitude: -10.7100,
      longitude: -76.2730,
      createdAt: DateTime.now().subtract(const Duration(days: 20)),
    ),
    EcoAlert(
      id: '15',
      title: 'Olores Nocivos en Planta de Tratamiento',
      description: 'Emanación de olores nauseabundos debido a fallas de mantenimiento en las lagunas de estabilización de aguas residuales.',
      category: EcoCategory.aire,
      severity: EcoSeverity.medio,
      district: EcoDistrict.simonBolivar,
      address: 'Sector Bellavista s/n, Simón Bolívar',
      latitude: -10.6970,
      longitude: -76.2750,
      createdAt: DateTime.now().subtract(const Duration(days: 14)),
    ),
    EcoAlert(
      id: '16',
      title: 'Derrame de Combustible por Cisterna',
      description: 'Fuga menor de diésel durante el reabastecimiento en grifo local, con escurrimiento hacia la cuneta de evacuación pluvial.',
      category: EcoCategory.agua,
      severity: EcoSeverity.medio,
      district: EcoDistrict.simonBolivar,
      address: 'Carretera Central KM 4.5 (Entrada a Quiulacocha), Simón Bolívar',
      latitude: -10.6880,
      longitude: -76.2720,
      createdAt: DateTime.now().subtract(const Duration(days: 16)),
    ),
  ];

  final StreamController<List<EcoAlert>> _controller = StreamController<List<EcoAlert>>.broadcast();

  MockAlertService() {
    // Enviar el estado inicial después de un pequeño delay
    Timer(const Duration(milliseconds: 300), () {
      _controller.add(List.unmodifiable(_alerts));
    });
  }

  @override
  Stream<List<EcoAlert>> watchAlerts() {
    return _controller.stream;
  }

  @override
  Future<void> addAlert(EcoAlert alert) async {
    // Simular latencia de base de datos
    await Future.delayed(const Duration(milliseconds: 800));
    _alerts.insert(0, alert);
    _controller.add(List.unmodifiable(_alerts));
  }

  @override
  Future<bool> resolveAlert(String id, String comment, String imageUrl) async {
    await Future.delayed(const Duration(milliseconds: 600));
    final index = _alerts.indexWhere((a) => a.id == id);
    if (index != -1) {
      _alerts[index] = _alerts[index].copyWith(
        status: 'solved',
        resolutionComment: comment,
        resolutionImageUrl: imageUrl,
        resolvedAt: DateTime.now(),
      );
      _controller.add(List.unmodifiable(_alerts));
      return true;
    }
    return false;
  }

  @override
  Future<bool> dismissAlert(String id, String comment) async {
    await Future.delayed(const Duration(milliseconds: 400));
    final index = _alerts.indexWhere((a) => a.id == id);
    if (index != -1) {
      _alerts[index] = _alerts[index].copyWith(
        status: 'dismissed',
        resolutionComment: comment,
      );
      _controller.add(List.unmodifiable(_alerts));
      return true;
    }
    return false;
  }

  @override
  Future<bool> transferAlert(String id, EcoDistrict newDistrict) async {
    await Future.delayed(const Duration(milliseconds: 400));
    final index = _alerts.indexWhere((a) => a.id == id);
    if (index != -1) {
      _alerts[index] = _alerts[index].copyWith(
        district: newDistrict,
      );
      _controller.add(List.unmodifiable(_alerts));
      return true;
    }
    return false;
  }

  @override
  void setToken(String? token) {}

  @override
  Future<List<WastePoint>> fetchWastePoints() async {
    return const [
      WastePoint(
        id: 'waste-1',
        name: 'Contenedor Plaza Yanacancha',
        description: 'Plaza Principal de Yanacancha. Puntos de separación esmeralda.',
        location: LatLng(-10.6625, -76.2555),
        type: WasteType.recyclable,
        fillLevel: FillLevel.low,
        truckSchedule: 'Lunes, Miércoles y Viernes a las 19:00',
      ),
      WastePoint(
        id: 'waste-2',
        name: 'Punto de Acopio Av. Los Próceres',
        description: 'Cerca al mercado local. Depósitos de residuos orgánicos municipales.',
        location: LatLng(-10.6640, -76.2530),
        type: WasteType.organic,
        fillLevel: FillLevel.medium,
        truckSchedule: 'Martes, Jueves y Sábado a las 18:30',
      ),
      WastePoint(
        id: 'waste-3',
        name: 'Contenedor General Hospital Huariaca',
        description: 'Residuos generales municipales no peligrosos.',
        location: LatLng(-10.6685, -76.2580),
        type: WasteType.general,
        fillLevel: FillLevel.full,
        truckSchedule: 'Diario (Lunes a Domingo) a las 08:00',
      ),
      WastePoint(
        id: 'waste-4',
        name: 'Contenedor Plaza Quiulacocha',
        description: 'Residuos generales. Punto de acopio del distrito Simón Bolívar.',
        location: LatLng(-10.6720, -76.2625),
        type: WasteType.general,
        fillLevel: FillLevel.medium,
        truckSchedule: 'Lunes y Jueves a las 14:00',
      ),
      WastePoint(
        id: 'waste-5',
        name: 'Punto Limpio Av. Bolívar Central',
        description: 'Contenedores verdes para reciclaje de papel, plástico y vidrio.',
        location: LatLng(-10.6705, -76.2600),
        type: WasteType.recyclable,
        fillLevel: FillLevel.low,
        truckSchedule: 'Martes y Sábado a las 16:00',
      ),
      WastePoint(
        id: 'waste-6',
        name: 'Botadero Municipal San Juan',
        description: 'Punto de acopio municipal oficial en el Sector San Juan. Autorizado para depositar bolsas de basura domésticas.',
        location: LatLng(-10.6750, -76.2520),
        type: WasteType.municipalDump,
        fillLevel: FillLevel.medium,
        truckSchedule: 'Recolección diaria por camión municipal compactador a las 20:00',
      ),
      WastePoint(
        id: 'waste-7',
        name: 'Botadero Oficial Yanacancha Alta',
        description: 'Botadero autorizado y supervisado por la Municipalidad Distrital de Yanacancha. Depósito seguro de bolsas de basura.',
        location: LatLng(-10.6580, -76.2480),
        type: WasteType.municipalDump,
        fillLevel: FillLevel.low,
        truckSchedule: 'Lunes, Miércoles y Viernes a las 22:00',
      ),
      WastePoint(
        id: 'waste-8',
        name: 'Punto de Desecho Simón Bolívar (La Esperanza)',
        description: 'Punto limpio oficial municipal. Contenedor de gran capacidad para almacenamiento temporal de bolsas de basura.',
        location: LatLng(-10.6690, -76.2710),
        type: WasteType.municipalDump,
        fillLevel: FillLevel.full,
        truckSchedule: 'Diario a las 06:00',
      ),
    ];
  }

  @override
  Future<List<LatLng>> fetchCollectorRoute() async {
    return const [
      LatLng(-10.6625, -76.2555), // Plaza Yanacancha
      LatLng(-10.6640, -76.2530), // Av. Los Próceres
      LatLng(-10.6685, -76.2580), // Hospital Huariaca
      LatLng(-10.6705, -76.2600), // Av. Bolívar Central
      LatLng(-10.6720, -76.2625), // Plaza Quiulacocha
    ];
  }

  @override
  Future<Map<String, dynamic>> fetchSiteSettings() async {
    return {
      'id': 1,
      'logo': null,
      'icon': null,
      'is_maintenance_mode': false,
      'maintenance_message': 'El sistema se encuentra en mantenimiento temporal por incidentes. Disculpe las molestias.',
    };
  }
}
