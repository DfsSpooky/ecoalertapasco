import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:latlong2/latlong.dart';
import 'dart:ui' as ui;
import 'package:google_fonts/google_fonts.dart';
import 'package:provider/provider.dart';
import '../models/eco_alert.dart';
import '../models/waste_point.dart';
import '../state/app_state.dart';
import 'alert_form_dialog.dart';
import 'auth_dialog.dart';
import 'pulsing_critical_dot.dart';
import 'package:file_picker/file_picker.dart';

class EcoMapView extends StatefulWidget {
  const EcoMapView({super.key});

  @override
  State<EcoMapView> createState() => _EcoMapViewState();
}

class _EcoMapViewState extends State<EcoMapView> {
  final MapController _mapController = MapController();
  final ScrollController _alertsScrollController = ScrollController();
  String? _lastSelectedAlertId;
  bool _isDarkMap = false; // MODO CLARO por defecto según pedido del usuario
  bool _isListExpanded = true;
  bool _showWasteLayer = false;
  bool _showDistrictBoundaries = false;

  @override
  void dispose() {
    _alertsScrollController.dispose();
    super.dispose();
  }

  // Datos simulados de Contenedores y Botaderos en Cerro de Pasco
  final List<WastePoint> _wastePoints = [
    const WastePoint(
      id: 'waste-1',
      name: 'Contenedor Plaza Yanacancha',
      description: 'Plaza Principal de Yanacancha. Puntos de separación esmeralda.',
      location: LatLng(-10.6625, -76.2555),
      type: WasteType.recyclable,
      fillLevel: FillLevel.low,
      truckSchedule: 'Lunes, Miércoles y Viernes a las 19:00',
    ),
    const WastePoint(
      id: 'waste-2',
      name: 'Punto de Acopio Av. Los Próceres',
      description: 'Cerca al mercado local. Depósitos de residuos orgánicos municipales.',
      location: LatLng(-10.6640, -76.2530),
      type: WasteType.organic,
      fillLevel: FillLevel.medium,
      truckSchedule: 'Martes, Jueves y Sábado a las 18:30',
    ),
    const WastePoint(
      id: 'waste-3',
      name: 'Contenedor General Hospital Huariaca',
      description: 'Residuos generales municipales no peligrosos.',
      location: LatLng(-10.6685, -76.2580),
      type: WasteType.general,
      fillLevel: FillLevel.full,
      truckSchedule: 'Diario (Lunes a Domingo) a las 08:00',
    ),
    const WastePoint(
      id: 'waste-4',
      name: 'Contenedor Plaza Quiulacocha',
      description: 'Residuos generales. Punto de acopio del distrito Simón Bolívar.',
      location: LatLng(-10.6720, -76.2625),
      type: WasteType.general,
      fillLevel: FillLevel.medium,
      truckSchedule: 'Lunes y Jueves a las 14:00',
    ),
    const WastePoint(
      id: 'waste-5',
      name: 'Punto Limpio Av. Bolívar Central',
      description: 'Contenedores verdes para reciclaje de papel, plástico y vidrio.',
      location: LatLng(-10.6705, -76.2600),
      type: WasteType.recyclable,
      fillLevel: FillLevel.low,
      truckSchedule: 'Martes y Sábado a las 16:00',
    ),
  ];

  // Coordenadas que trazan la ruta del camión recolector de basura (para polilínea)
  final List<LatLng> _truckRoutePoints = const [
    LatLng(-10.6625, -76.2555), // Plaza Yanacancha
    LatLng(-10.6640, -76.2530), // Av. Los Próceres
    LatLng(-10.6685, -76.2580), // Hospital Huariaca
    LatLng(-10.6705, -76.2600), // Av. Bolívar Central
    LatLng(-10.6720, -76.2625), // Plaza Quiulacocha
  ];

  void _showAuthDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => const AuthDialog(),
    );
  }

  // Coordenadas centrales de Cerro de Pasco
  final LatLng _centerPasco = const LatLng(-10.6675, -76.2567);

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final isMobile = MediaQuery.of(context).size.width < 850;
      if (isMobile) {
        setState(() {
          _isListExpanded = false;
        });
      }
      // Obtener ubicación GPS inicial del usuario de forma proactiva
      Provider.of<AppState>(context, listen: false).getUserLatLng();
    });
  }

  void _showWasteDetailDialog(BuildContext context, WastePoint point) {
    showModalBottomSheet(
      context: context,
      backgroundColor: Colors.transparent,
      builder: (context) {
        Color fillColor;
        double fillPercentage;
        String fillText;
        switch (point.fillLevel) {
          case FillLevel.low:
            fillColor = const Color(0xFF2E7D32);
            fillPercentage = 0.25;
            fillText = 'Disponible (25%)';
            break;
          case FillLevel.medium:
            fillColor = const Color(0xFFF57C00);
            fillPercentage = 0.60;
            fillText = 'Moderado (60%)';
            break;
          case FillLevel.full:
            fillColor = const Color(0xFFD32F2F);
            fillPercentage = 0.95;
            fillText = 'Lleno - Reporte de vaciado (95%)';
            break;
        }

        String wasteText;
        Color wasteColor;
        switch (point.type) {
          case WasteType.recyclable:
            wasteText = 'Reciclables ♻️ (Plástico, Papel, Vidrio)';
            wasteColor = const Color(0xFF1B5E20);
            break;
          case WasteType.organic:
            wasteText = 'Orgánicos 🍂 (Cáscaras, Restos orgánicos)';
            wasteColor = const Color(0xFFE65100);
            break;
          case WasteType.general:
            wasteText = 'Residuos Generales 🗑️ (No reciclables)';
            wasteColor = const Color(0xFF37474F);
            break;
        }

        return Container(
          decoration: const BoxDecoration(
            color: Colors.white,
            borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
          ),
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Center(
                child: Container(
                  width: 48,
                  height: 5,
                  decoration: BoxDecoration(
                    color: const Color(0xFFE2E8F0),
                    borderRadius: BorderRadius.circular(10),
                  ),
                ),
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  const Icon(Icons.pin_drop_rounded, color: Color(0xFF2E7D32), size: 28),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      point.name,
                      style: GoogleFonts.outfit(
                        fontSize: 20,
                        fontWeight: FontWeight.bold,
                        color: const Color(0xFF0F172A),
                      ),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 8),
              Text(
                point.description,
                style: GoogleFonts.outfit(
                  fontSize: 14,
                  color: const Color(0xFF64748B),
                ),
              ),
              const SizedBox(height: 16),
              const Divider(color: Color(0xFFE2E8F0)),
              const SizedBox(height: 16),
              Text(
                'TIPO DE RESIDUO ACEPTADO',
                style: GoogleFonts.outfit(
                  fontSize: 11,
                  fontWeight: FontWeight.w800,
                  color: const Color(0xFF94A3B8),
                  letterSpacing: 0.5,
                ),
              ),
              const SizedBox(height: 6),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                decoration: BoxDecoration(
                  color: wasteColor.withOpacity(0.1),
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: wasteColor.withOpacity(0.3)),
                ),
                child: Text(
                  wasteText,
                  style: GoogleFonts.outfit(
                    color: wasteColor,
                    fontSize: 13,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
              const SizedBox(height: 16),
              Text(
                'NIVEL DE LLENADO DEL CONTENEDOR',
                style: GoogleFonts.outfit(
                  fontSize: 11,
                  fontWeight: FontWeight.w800,
                  color: const Color(0xFF94A3B8),
                  letterSpacing: 0.5,
                ),
              ),
              const SizedBox(height: 6),
              Row(
                children: [
                  Expanded(
                    child: ClipRRect(
                      borderRadius: BorderRadius.circular(4),
                      child: LinearProgressIndicator(
                        value: fillPercentage,
                        color: fillColor,
                        backgroundColor: const Color(0xFFF1F5F9),
                        minHeight: 8,
                      ),
                    ),
                  ),
                  const SizedBox(width: 12),
                  Text(
                    fillText,
                    style: GoogleFonts.outfit(
                      color: fillColor,
                      fontSize: 13,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 20),
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: const Color(0xFFF8FAFC),
                  borderRadius: BorderRadius.circular(12),
                  border: Border.all(color: const Color(0xFFE2E8F0)),
                ),
                child: Row(
                  children: [
                    const Icon(Icons.local_shipping_rounded, color: Color(0xFF475569), size: 24),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            'Paso del Camión Recolector:',
                            style: GoogleFonts.outfit(
                              fontSize: 12,
                              color: const Color(0xFF64748B),
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          const SizedBox(height: 2),
                          Text(
                            point.truckSchedule,
                            style: GoogleFonts.outfit(
                              fontSize: 13,
                              color: const Color(0xFF0F172A),
                              fontWeight: FontWeight.w600,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 20),
              SizedBox(
                width: double.infinity,
                child: ElevatedButton(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: const Color(0xFF0F172A),
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                  ),
                  onPressed: () => Navigator.pop(context),
                  child: Text(
                    'Entendido',
                    style: GoogleFonts.outfit(
                      color: Colors.white,
                      fontSize: 14,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);
    final alerts = appState.filteredAlerts;
    final isMobile = MediaQuery.of(context).size.width < 850;

    // Centrar mapa si la alerta seleccionada cambia globalmente (ej. desde el dashboard)
    if (appState.selectedAlert != null && appState.selectedAlert!.id != _lastSelectedAlertId) {
      _lastSelectedAlertId = appState.selectedAlert!.id;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        _mapController.move(
          LatLng(appState.selectedAlert!.latitude, appState.selectedAlert!.longitude),
          15.5,
        );
      });
    } else if (appState.selectedAlert == null && _lastSelectedAlertId != null) {
      _lastSelectedAlertId = null;
    }

    // URL de Tiles
    final tileUrl = _isDarkMap
        ? 'https://a.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png'
        : 'https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png';

    // Generar marcadores
    final List<Marker> markers = alerts.map((alert) {
      return Marker(
        point: LatLng(alert.latitude, alert.longitude),
        width: 42,
        height: 42,
        child: GestureDetector(
          onTap: () {
            appState.selectedAlert = alert;
          },
          child: _MapPin(
            category: alert.category,
            severity: alert.severity,
            status: alert.status,
            isSelected: appState.selectedAlert?.id == alert.id,
          ),
        ),
      );
    }).toList();

    // Agregar marcador de la ubicación actual del usuario (Punto Azul GPS)
    if (appState.userLocation != null) {
      markers.add(
        Marker(
          point: appState.userLocation!,
          width: 32,
          height: 32,
          child: Stack(
            alignment: Alignment.center,
            children: [
              Container(
                width: 24,
                height: 24,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: const Color(0xFF0288D1).withOpacity(0.25),
                ),
              ),
              Container(
                width: 14,
                height: 14,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: Colors.white,
                  border: Border.all(
                    color: const Color(0xFF0288D1),
                    width: 3,
                  ),
                  boxShadow: [
                    BoxShadow(
                      color: Colors.black.withOpacity(0.2),
                      blurRadius: 4,
                      offset: const Offset(0, 1),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      );
    }

    // Agregar marcadores de contenedores si la capa está activa
    if (_showWasteLayer) {
      for (final wp in _wastePoints) {
        Color itemColor;
        IconData itemIcon;
        switch (wp.type) {
          case WasteType.recyclable:
            itemColor = const Color(0xFF2E7D32); // Green
            itemIcon = Icons.recycling_rounded;
            break;
          case WasteType.organic:
            itemColor = const Color(0xFFE65100); // Amber/Orange
            itemIcon = Icons.eco_rounded;
            break;
          case WasteType.general:
            itemColor = const Color(0xFF37474F); // Grey/Slate
            itemIcon = Icons.delete_outline_rounded;
            break;
        }

        markers.add(
          Marker(
            point: wp.location,
            width: 38,
            height: 38,
            child: GestureDetector(
              onTap: () {
                _showWasteDetailDialog(context, wp);
              },
              child: Stack(
                alignment: Alignment.center,
                children: [
                  Container(
                    width: 38,
                    height: 38,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: itemColor.withOpacity(0.2),
                    ),
                  ),
                  Container(
                    width: 28,
                    height: 28,
                    decoration: BoxDecoration(
                      color: itemColor,
                      shape: BoxShape.circle,
                      border: Border.all(color: Colors.white, width: 2),
                      boxShadow: [
                        BoxShadow(
                          color: Colors.black.withOpacity(0.15),
                          blurRadius: 4,
                          offset: const Offset(0, 2),
                        )
                      ],
                    ),
                    child: Center(
                      child: Icon(
                        itemIcon,
                        color: Colors.white,
                        size: 14,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        );
      }
    }

    return Column(
      children: [
        // Mapa y overlays
        Expanded(
          flex: 4,
          child: Stack(
            children: [
              // Widget del Mapa
              FlutterMap(
                mapController: _mapController,
                options: MapOptions(
                  initialCenter: _centerPasco,
                  initialZoom: 14.0,
                  minZoom: 10,
                  maxZoom: 18,
                  onTap: (tapPosition, point) {
                    if (appState.isReportMode) {
                      // Auto-zoom a 17.0 para máxima precisión de ubicación
                      _mapController.move(point, 17.0);
                      appState.confirmReportLocation(point.latitude, point.longitude);
                    } else {
                      // Limpiar selección
                      appState.selectedAlert = null;
                    }
                  },
                  onPositionChanged: (position, hasGesture) {
                    if (appState.isReportMode) {
                      appState.updateTempLocation(
                        position.center.latitude,
                        position.center.longitude,
                      );
                    }
                  },
                ),
                children: [
                  TileLayer(
                    urlTemplate: tileUrl,
                    subdomains: const ['a', 'b', 'c'],
                    userAgentPackageName: 'com.ecoalerta.app',
                  ),
                  
                  // Capa de Ruta de Recolector (Polilínea verde discontinua/semitransparente)
                  if (_showWasteLayer)
                    PolylineLayer(
                      polylines: [
                        Polyline(
                          points: _truckRoutePoints,
                          color: const Color(0xFF2E7D32).withOpacity(0.55),
                          strokeWidth: 4.5,
                        ),
                      ],
                    ),

                  // Capa de Límites Poligonales de Distritos
                  if (_showDistrictBoundaries)
                    PolygonLayer(
                      polygons: [
                        // Chaupimarca (Centro/Sur - Color Ámbar)
                        Polygon(
                          points: const [
                            LatLng(-10.6800, -76.2650),
                            LatLng(-10.6655, -76.2630),
                            LatLng(-10.6655, -76.2480),
                            LatLng(-10.6800, -76.2480),
                          ],
                          color: const Color(0xFFFFB74D).withOpacity(0.12),
                          borderColor: const Color(0xFFFB8C00),
                          borderStrokeWidth: 2.0,
                        ),
                        // Yanacancha (Norte - Color Azul)
                        Polygon(
                          points: const [
                            LatLng(-10.6655, -76.2630),
                            LatLng(-10.6500, -76.2630),
                            LatLng(-10.6500, -76.2450),
                            LatLng(-10.6655, -76.2450),
                          ],
                          color: const Color(0xFF64B5F6).withOpacity(0.12),
                          borderColor: const Color(0xFF1E88E5),
                          borderStrokeWidth: 2.0,
                        ),
                        // Simón Bolívar (Oeste - Color Verde)
                        Polygon(
                          points: const [
                            LatLng(-10.6800, -76.2800),
                            LatLng(-10.6500, -76.2800),
                            LatLng(-10.6500, -76.2630),
                            LatLng(-10.6800, -76.2630),
                          ],
                          color: const Color(0xFF81C784).withOpacity(0.12),
                          borderColor: const Color(0xFF43A047),
                          borderStrokeWidth: 2.0,
                        ),
                      ],
                    ),

                  if (appState.isProximityFilterActive && appState.userLocation != null)
                    CircleLayer(
                      circles: [
                        CircleMarker(
                          point: appState.userLocation!,
                          radius: appState.proximityRadius,
                          useRadiusInMeter: true,
                          color: const Color(0xFF0288D1).withOpacity(0.08),
                          borderColor: const Color(0xFF0288D1).withOpacity(0.45),
                          borderStrokeWidth: 1.5,
                        ),
                      ],
                    ),

                  MarkerLayer(markers: markers),
                ],
              ),
      
              // Banner de Emergencia Crítica Simulada
              if (appState.activeEmergencyAlert != null)
                Positioned(
                  top: 80,
                  left: 16,
                  right: 16,
                  child: Center(
                    child: Container(
                      constraints: const BoxConstraints(maxWidth: 480),
                      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                      decoration: BoxDecoration(
                        color: const Color(0xFFFEF2F2), // Red 50
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(color: const Color(0xFFEF4444), width: 2), // Red 500
                        boxShadow: [
                          BoxShadow(
                            color: Colors.black.withOpacity(0.12),
                            blurRadius: 16,
                            offset: const Offset(0, 4),
                          )
                        ],
                      ),
                      child: Row(
                        children: [
                          const Icon(Icons.warning_rounded, color: Color(0xFFDC2626), size: 24),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                Text(
                                  appState.activeEmergencyAlert!.title,
                                  style: GoogleFonts.outfit(
                                    fontSize: 13,
                                    fontWeight: FontWeight.bold,
                                    color: const Color(0xFF991B1B),
                                  ),
                                ),
                                const SizedBox(height: 2),
                                Text(
                                  '¡Detección de crisis activa! Protocolo de reacción inmediata.',
                                  style: GoogleFonts.outfit(
                                    fontSize: 11,
                                    color: const Color(0xFF7F1D1D),
                                  ),
                                ),
                              ],
                            ),
                          ),
                          const SizedBox(width: 8),
                          TextButton(
                            onPressed: () {
                              final alert = appState.activeEmergencyAlert!;
                                appState.selectedAlert = alert;
                            },
                            style: TextButton.styleFrom(
                              foregroundColor: const Color(0xFFDC2626),
                              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                            ),
                            child: Text(
                              'Enfocar',
                              style: GoogleFonts.outfit(fontWeight: FontWeight.bold, fontSize: 12),
                            ),
                          ),
                          IconButton(
                            icon: const Icon(Icons.close_rounded, color: Color(0xFF991B1B), size: 18),
                            onPressed: () {
                              appState.dismissEmergencyAlert();
                            },
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
      
              // Mira central en modo reporte (Visor Premium con dirección en tiempo real)
              if (appState.isReportMode)
                IgnorePointer(
                  child: Center(
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        // Tooltip flotante con dirección en tiempo real
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                          decoration: BoxDecoration(
                            color: const Color(0xFF0F172A), // Slate 900
                            borderRadius: BorderRadius.circular(20),
                            border: Border.all(color: const Color(0xFF334155), width: 1.5),
                            boxShadow: [
                              BoxShadow(
                                color: Colors.black.withOpacity(0.25),
                                blurRadius: 12,
                                offset: const Offset(0, 4),
                              )
                            ],
                          ),
                          child: Row(
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              const Icon(Icons.location_on_rounded, color: Color(0xFF10B981), size: 14),
                              const SizedBox(width: 6),
                              Text(
                                appState.currentGeocodedAddress,
                                style: GoogleFonts.outfit(
                                  color: Colors.white,
                                  fontSize: 12,
                                  fontWeight: FontWeight.bold,
                                ),
                              ),
                            ],
                          ),
                        ),
                        const SizedBox(height: 8),
                        
                        // Conector de mira
                        Container(
                          width: 2,
                          height: 10,
                          color: const Color(0xFF0288D1),
                        ),
                        
                        // Mira digital y de precisión
                        Stack(
                          alignment: Alignment.center,
                          children: [
                            // Anillo exterior difuminado
                            Container(
                              width: 56,
                              height: 56,
                              decoration: BoxDecoration(
                                shape: BoxShape.circle,
                                color: const Color(0xFF0288D1).withOpacity(0.08),
                                border: Border.all(
                                  color: const Color(0xFF0288D1).withOpacity(0.35),
                                  width: 1,
                                ),
                              ),
                            ),
                            
                            // Fondo del visor
                            Container(
                              width: 26,
                              height: 26,
                              decoration: BoxDecoration(
                                shape: BoxShape.circle,
                                color: Colors.white.withOpacity(0.85),
                                border: Border.all(
                                  color: const Color(0xFF0288D1),
                                  width: 2,
                                ),
                              ),
                            ),
                            
                            // Punto de precisión rojo
                            Container(
                              width: 6,
                              height: 6,
                              decoration: const BoxDecoration(
                                color: Color(0xFFEF4444), // Red 500
                                shape: BoxShape.circle,
                              ),
                            ),
                            
                            // Llave de enfoque digital
                            const Icon(
                              Icons.center_focus_strong_rounded,
                              color: Color(0xFF0288D1),
                              size: 40,
                            ),
                          ],
                        ),
                        // Alineado exactamente al centro matemático del mapa para coincidir con position.center
                        const SizedBox(height: 0),
                      ],
                    ),
                  ),
                ),
      
              // Botones de Zoom en el mapa (+ / -) para precisión táctil
              Positioned(
                left: 16,
                top: isMobile ? 80 : MediaQuery.of(context).size.height / 2 - 50,
                child: Column(
                  children: [
                    Container(
                      decoration: BoxDecoration(
                        color: Colors.white,
                        borderRadius: BorderRadius.circular(12),
                        border: Border.all(color: const Color(0xFFE2E8F0), width: 1.5),
                        boxShadow: [
                          BoxShadow(
                            color: Colors.black.withOpacity(0.08),
                            blurRadius: 8,
                            offset: const Offset(0, 2),
                          )
                        ],
                      ),
                      child: IconButton(
                        icon: const Icon(Icons.add_rounded, color: Color(0xFF0F172A), size: 22),
                        tooltip: 'Acercar',
                        onPressed: () {
                          final currentZoom = _mapController.camera.zoom;
                          if (currentZoom < 18) {
                            _mapController.move(_mapController.camera.center, currentZoom + 0.5);
                          }
                        },
                      ),
                    ),
                    const SizedBox(height: 8),
                    Container(
                      decoration: BoxDecoration(
                        color: Colors.white,
                        borderRadius: BorderRadius.circular(12),
                        border: Border.all(color: const Color(0xFFE2E8F0), width: 1.5),
                        boxShadow: [
                          BoxShadow(
                            color: Colors.black.withOpacity(0.08),
                            blurRadius: 8,
                            offset: const Offset(0, 2),
                          )
                        ],
                      ),
                      child: IconButton(
                        icon: const Icon(Icons.remove_rounded, color: Color(0xFF0F172A), size: 22),
                        tooltip: 'Alejar',
                        onPressed: () {
                          final currentZoom = _mapController.camera.zoom;
                          if (currentZoom > 10) {
                            _mapController.move(_mapController.camera.center, currentZoom - 0.5);
                          }
                        },
                      ),
                    ),
                  ],
                ),
              ),

              // Botón de Menú (para abrir el Drawer en versión móvil)
              Positioned(
                top: 16,
                left: 16,
                child: Builder(
                  builder: (context) {
                    if (!isMobile) return const SizedBox.shrink();
                    return Container(
                      decoration: BoxDecoration(
                        color: Colors.white,
                        borderRadius: BorderRadius.circular(12),
                        border: Border.all(color: const Color(0xFFE2E8F0), width: 1.5),
                        boxShadow: [
                          BoxShadow(
                            color: Colors.black.withOpacity(0.08),
                            blurRadius: 8,
                            offset: const Offset(0, 2),
                          )
                        ],
                      ),
                      child: IconButton(
                        icon: const Icon(
                          Icons.menu_rounded,
                          color: Color(0xFF0288D1),
                        ),
                        tooltip: 'Filtros y Estadísticas',
                        onPressed: () {
                          Scaffold.of(context).openDrawer();
                        },
                      ),
                    );
                  }
                ),
              ),

              // Encabezado del Mapa con botón de cambiar estilo
              Positioned(
                top: 16,
                right: 16,
                child: Row(
                  children: [
                    // Indicador de modo reporte
                    if (appState.isReportMode)
                      Container(
                        margin: const EdgeInsets.only(right: 8),
                        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                        decoration: BoxDecoration(
                          color: const Color(0xFFE0F2FE), // Light Blue 100
                          borderRadius: BorderRadius.circular(20),
                          border: Border.all(color: const Color(0xFF0288D1), width: 1.5),
                          boxShadow: [
                            BoxShadow(
                              color: Colors.black.withOpacity(0.08),
                              blurRadius: 8,
                            )
                          ],
                        ),
                        child: Row(
                          children: [
                            const Icon(Icons.location_searching, color: Color(0xFF0288D1), size: 16),
                            const SizedBox(width: 6),
                            Text(
                              'Modo Ubicación Activo',
                              style: GoogleFonts.outfit(
                                color: const Color(0xFF075985), // Blue 800
                                fontSize: 12,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                          ],
                        ),
                      ),
                    
                    // Botón de Contenedores y Rutas
                    if (!appState.isReportMode)
                      Container(
                        margin: const EdgeInsets.only(right: 8),
                        decoration: BoxDecoration(
                          color: _showWasteLayer ? const Color(0xFFE8F5E9) : Colors.white,
                          borderRadius: BorderRadius.circular(12),
                          border: Border.all(
                            color: _showWasteLayer ? const Color(0xFF2E7D32) : const Color(0xFFE2E8F0),
                            width: 1.5,
                          ),
                          boxShadow: [
                            BoxShadow(
                              color: Colors.black.withOpacity(0.08),
                              blurRadius: 8,
                              offset: const Offset(0, 2),
                            )
                          ],
                        ),
                        child: TextButton.icon(
                          icon: Icon(
                            Icons.recycling_rounded,
                            color: _showWasteLayer ? const Color(0xFF2E7D32) : const Color(0xFF64748B),
                            size: 16,
                          ),
                          label: Text(
                            '♻️ Contenedores',
                            style: GoogleFonts.outfit(
                              color: _showWasteLayer ? const Color(0xFF2E7D32) : const Color(0xFF475569),
                              fontSize: 12,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          style: TextButton.styleFrom(
                            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                          ),
                          onPressed: () {
                            setState(() {
                              _showWasteLayer = !_showWasteLayer;
                            });
                          },
                        ),
                      ),

                    // Botón de Límites de Distritos
                    if (!appState.isReportMode)
                      Container(
                        margin: const EdgeInsets.only(right: 8),
                        decoration: BoxDecoration(
                          color: _showDistrictBoundaries ? const Color(0xFFFFF3E0) : Colors.white,
                          borderRadius: BorderRadius.circular(12),
                          border: Border.all(
                            color: _showDistrictBoundaries ? const Color(0xFFFF9800) : const Color(0xFFE2E8F0),
                            width: 1.5,
                          ),
                          boxShadow: [
                            BoxShadow(
                              color: Colors.black.withOpacity(0.08),
                              blurRadius: 8,
                              offset: const Offset(0, 2),
                            )
                          ],
                        ),
                        child: TextButton.icon(
                          icon: Icon(
                            Icons.map_outlined,
                            color: _showDistrictBoundaries ? const Color(0xFFFF9800) : const Color(0xFF64748B),
                            size: 16,
                          ),
                          label: Text(
                            '🗺️ Límites Distritos',
                            style: GoogleFonts.outfit(
                              color: _showDistrictBoundaries ? const Color(0xFFFF9800) : const Color(0xFF475569),
                              fontSize: 12,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          style: TextButton.styleFrom(
                            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                          ),
                          onPressed: () {
                            setState(() {
                              _showDistrictBoundaries = !_showDistrictBoundaries;
                            });
                          },
                        ),
                      ),

                    // Botón de geolocalización GPS
                    Container(
                      margin: const EdgeInsets.only(right: 8),
                      decoration: BoxDecoration(
                        color: Colors.white,
                        borderRadius: BorderRadius.circular(12),
                        border: Border.all(color: const Color(0xFFE2E8F0), width: 1.5),
                        boxShadow: [
                          BoxShadow(
                            color: Colors.black.withOpacity(0.08),
                            blurRadius: 8,
                            offset: const Offset(0, 2),
                          )
                        ],
                      ),
                      child: TextButton.icon(
                        icon: const Icon(
                          Icons.my_location_rounded,
                          color: Color(0xFF0288D1),
                          size: 16,
                        ),
                        label: Text(
                          'Mi Ubicación',
                          style: GoogleFonts.outfit(
                            color: const Color(0xFF0288D1),
                            fontSize: 12,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                        style: TextButton.styleFrom(
                          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                        ),
                        onPressed: () async {
                          final latLng = await appState.getUserLatLng();
                          if (latLng != null) {
                            _mapController.move(latLng, 16.0);
                            appState.updateTempLocation(latLng.latitude, latLng.longitude);
                            if (context.mounted) {
                              ScaffoldMessenger.of(context).showSnackBar(
                                SnackBar(
                                  backgroundColor: const Color(0xFF0288D1),
                                  duration: const Duration(seconds: 2),
                                  content: Text(
                                    'Mapa centrado en tu ubicación GPS.',
                                    style: GoogleFonts.outfit(color: Colors.white),
                                  ),
                                ),
                              );
                            }
                          } else {
                            if (context.mounted) {
                              ScaffoldMessenger.of(context).showSnackBar(
                                SnackBar(
                                  backgroundColor: Colors.amber.shade800,
                                  duration: const Duration(seconds: 3),
                                  content: Text(
                                    'No se pudo obtener la ubicación. Verifique los permisos GPS.',
                                    style: GoogleFonts.outfit(color: Colors.white),
                                  ),
                                ),
                              );
                            }
                          }
                        },
                      ),
                    ),
                    
                    // Botón de alternar Tema del Mapa
                    Container(
                      decoration: BoxDecoration(
                        color: Colors.white,
                        borderRadius: BorderRadius.circular(12),
                        border: Border.all(color: const Color(0xFFE2E8F0), width: 1.5),
                        boxShadow: [
                          BoxShadow(
                            color: Colors.black.withOpacity(0.08),
                            blurRadius: 8,
                            offset: const Offset(0, 2),
                          )
                        ],
                      ),
                      child: IconButton(
                        icon: Icon(
                          _isDarkMap ? Icons.light_mode_rounded : Icons.dark_mode_rounded,
                          color: _isDarkMap ? const Color(0xFFE65100) : const Color(0xFF1E293B),
                        ),
                        tooltip: _isDarkMap ? 'Mapa Claro' : 'Mapa Oscuro',
                        onPressed: () {
                          setState(() {
                            _isDarkMap = !_isDarkMap;
                          });
                        },
                      ),
                    ),
                  ],
                ),
              ),
      
              // Leyenda Filtrable del Mapa
              if (!appState.isReportMode)
                Positioned(
                  bottom: 88,
                  right: 24,
                  child: Container(
                    width: 155,
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: Colors.white,
                      borderRadius: BorderRadius.circular(16),
                      border: Border.all(color: const Color(0xFFE2E8F0), width: 1.5),
                      boxShadow: [
                        BoxShadow(
                          color: Colors.black.withOpacity(0.08),
                          blurRadius: 10,
                          offset: const Offset(0, 2),
                        )
                      ],
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text(
                          'CATEGORÍAS',
                          style: GoogleFonts.outfit(
                            fontSize: 9,
                            fontWeight: FontWeight.w800,
                            color: const Color(0xFF64748B),
                            letterSpacing: 0.5,
                          ),
                        ),
                        const SizedBox(height: 8),
                        _buildLegendToggleItem(
                          context,
                          appState,
                          EcoCategory.mineria,
                          'Minería',
                          Icons.terrain_rounded,
                          const Color(0xFFE65100),
                        ),
                        const SizedBox(height: 6),
                        _buildLegendToggleItem(
                          context,
                          appState,
                          EcoCategory.basura,
                          'Basura',
                          Icons.delete_outline_rounded,
                          const Color(0xFF607D8B),
                        ),
                        const SizedBox(height: 6),
                        _buildLegendToggleItem(
                          context,
                          appState,
                          EcoCategory.agua,
                          'Agua',
                          Icons.water_drop_rounded,
                          const Color(0xFF0288D1),
                        ),
                        const SizedBox(height: 6),
                        _buildLegendToggleItem(
                          context,
                          appState,
                          EcoCategory.aire,
                          'Aire',
                          Icons.air_rounded,
                          const Color(0xFF00897B),
                        ),
                        const Padding(
                          padding: EdgeInsets.symmetric(vertical: 8.0),
                          child: Divider(color: Color(0xFFE2E8F0), height: 1),
                        ),
                        Text(
                          'SEVERIDAD',
                          style: GoogleFonts.outfit(
                            fontSize: 9,
                            fontWeight: FontWeight.w800,
                            color: const Color(0xFF64748B),
                            letterSpacing: 0.5,
                          ),
                        ),
                        const SizedBox(height: 8),
                        _buildLegendSeverityToggleItem(
                          context,
                          appState,
                          EcoSeverity.critico,
                          'Crítico',
                          const Color(0xFFD32F2F),
                        ),
                        const SizedBox(height: 6),
                        _buildLegendSeverityToggleItem(
                          context,
                          appState,
                          EcoSeverity.medio,
                          'Medio',
                          const Color(0xFFFFA000),
                        ),
                        const SizedBox(height: 6),
                        _buildLegendSeverityToggleItem(
                          context,
                          appState,
                          EcoSeverity.bajo,
                          'Bajo',
                          const Color(0xFF388E3C),
                        ),
                      ],
                    ),
                  ),
                ),
      
              // Botón Flotante para Reportar Alerta
              if (!appState.isReportMode)
                Positioned(
                  bottom: 24,
                  right: 24,
                  child: FloatingActionButton.extended(
                    onPressed: () {
                      if (!appState.isLoggedIn) {
                        _showAuthDialog(context);
                      } else {
                        appState.startReportMode();
                      }
                    },
                    backgroundColor: const Color(0xFF0288D1), // Blue
                    hoverColor: const Color(0xFF01579B),
                    elevation: 4,
                    icon: const Icon(Icons.add_location_alt_rounded, color: Colors.white),
                    label: Text(
                      'Reportar Alerta',
                      style: GoogleFonts.outfit(
                        fontWeight: FontWeight.bold,
                        fontSize: 14,
                        color: Colors.white,
                      ),
                    ),
                  ),
                ),
      
              // Panel de instrucciones / confirmación en Modo Reporte
              if (appState.isReportMode)
                Positioned(
                  bottom: 24,
                  left: 24,
                  right: 24,
                  child: Center(
                    child: Container(
                      constraints: const BoxConstraints(maxWidth: 500),
                      padding: const EdgeInsets.all(16),
                      decoration: BoxDecoration(
                        color: Colors.white,
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(color: const Color(0xFFE2E8F0), width: 1.5),
                        boxShadow: [
                          BoxShadow(
                            color: Colors.black.withOpacity(0.12),
                            blurRadius: 16,
                            offset: const Offset(0, 4),
                          )
                        ],
                      ),
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Text(
                            '📍 Ubica el incidente ecológico',
                            style: GoogleFonts.outfit(
                              color: const Color(0xFF0F172A),
                              fontSize: 16,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          const SizedBox(height: 6),
                          Text(
                            'Arrastra el mapa para alinear el marcador central con el punto del reporte.',
                            textAlign: TextAlign.center,
                            style: GoogleFonts.outfit(
                              color: const Color(0xFF334155),
                              fontSize: 13,
                            ),
                          ),
                          const SizedBox(height: 16),
                          Row(
                            children: [
                              Expanded(
                                child: OutlinedButton(
                                  onPressed: () {
                                    appState.cancelReportMode();
                                  },
                                  style: OutlinedButton.styleFrom(
                                    foregroundColor: const Color(0xFF475569),
                                    side: const BorderSide(color: Color(0xFFCBD5E1)),
                                    padding: const EdgeInsets.symmetric(vertical: 14),
                                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                                  ),
                                  child: Text('Cancelar', style: GoogleFonts.outfit(fontWeight: FontWeight.w600)),
                                ),
                              ),
                              const SizedBox(width: 12),
                              Expanded(
                                child: ElevatedButton(
                                  onPressed: () {
                                    // Confirmar ubicación
                                    final lat = appState.tempLatitude ?? _centerPasco.latitude;
                                    final lng = appState.tempLongitude ?? _centerPasco.longitude;
                                    appState.confirmReportLocation(lat, lng);
                                    
                                    // Abrir formulario
                                    showDialog(
                                      context: context,
                                      barrierDismissible: false,
                                      builder: (context) => const AlertFormDialog(),
                                    );
                                  },
                                  style: ElevatedButton.styleFrom(
                                    backgroundColor: const Color(0xFF0288D1),
                                    foregroundColor: Colors.white,
                                    elevation: 0,
                                    padding: const EdgeInsets.symmetric(vertical: 14),
                                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                                  ),
                                  child: Text(
                                    'Confirmar Ubicación',
                                    style: GoogleFonts.outfit(fontWeight: FontWeight.bold),
                                  ),
                                ),
                              ),
                            ],
                          )
                        ],
                      ),
                    ),
                  ),
                ),
      
              // Detalle de alerta seleccionada
              if (appState.selectedAlert != null && !appState.isReportMode)
                Positioned(
                  bottom: isMobile ? 12 : 24,
                  left: isMobile ? 16 : 24,
                  right: isMobile ? 16 : null,
                  child: _AlertDetailCard(
                    alert: appState.selectedAlert!,
                    onClose: () {
                      appState.selectedAlert = null;
                    },
                  ),
                ),
            ],
          ),
        ),
        
        // Listado de Alertas en el área inferior (colapsable)
        Container(
          height: _isListExpanded ? (isMobile ? 170 : 200) : 44,
          decoration: const BoxDecoration(
            color: Color(0xFFF8FAFC),
            border: Border(
              top: BorderSide(color: Color(0xFFE2E8F0), width: 1.5),
            ),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Cabecera del Listado (manija de colapso)
              InkWell(
                onTap: () {
                  setState(() {
                    _isListExpanded = !_isListExpanded;
                  });
                },
                child: Padding(
                  padding: const EdgeInsets.only(left: 16.0, right: 16.0, top: 12.0, bottom: 12.0),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Expanded(
                        child: Row(
                          children: [
                            Icon(
                              _isListExpanded ? Icons.keyboard_arrow_down : Icons.keyboard_arrow_up,
                              color: const Color(0xFF64748B),
                              size: 18,
                            ),
                            const SizedBox(width: 6),
                            Expanded(
                              child: Text(
                                isMobile ? 'ALERTAS EN EL ÁREA' : 'LISTADO DE ALERTAS ACTIVAS',
                                overflow: TextOverflow.ellipsis,
                                maxLines: 1,
                                style: GoogleFonts.outfit(
                                  fontSize: 11,
                                  fontWeight: FontWeight.w800,
                                  color: const Color(0xFF64748B),
                                  letterSpacing: 1.0,
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),
                      const SizedBox(width: 8),
                      Text(
                        isMobile ? '${alerts.length} act.' : 'Mostrando ${alerts.length} alertas',
                        style: GoogleFonts.outfit(
                          fontSize: 11,
                          fontWeight: FontWeight.w600,
                          color: const Color(0xFF0288D1),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              if (_isListExpanded) ...[
              
              // Fila horizontal de tarjetas
              Expanded(
                child: alerts.isEmpty
                    ? Center(
                        child: Text(
                          'No hay alertas registradas que coincidan con los filtros.',
                          style: GoogleFonts.outfit(
                            color: const Color(0xFF94A3B8),
                            fontSize: 13,
                          ),
                        ),
                      )
                    : Listener(
                        onPointerSignal: (pointerSignal) {
                          if (pointerSignal is PointerScrollEvent) {
                            final newOffset = _alertsScrollController.offset +
                                pointerSignal.scrollDelta.dy;
                            _alertsScrollController.jumpTo(
                              newOffset.clamp(
                                0.0,
                                _alertsScrollController.position.maxScrollExtent,
                              ),
                            );
                          }
                        },
                        child: ListView.builder(
                          controller: _alertsScrollController,
                          scrollDirection: Axis.horizontal,
                          padding: const EdgeInsets.only(left: 16.0, right: 16.0, bottom: 16.0),
                          itemCount: alerts.length,
                          itemBuilder: (context, index) {
                          final alert = alerts[index];
                          
                          // Severidad Styling
                          Color sevColor;
                          String sevEmoji;
                          switch (alert.severity) {
                            case EcoSeverity.critico:
                              sevColor = const Color(0xFFD32F2F);
                              sevEmoji = '💀';
                              break;
                            case EcoSeverity.medio:
                              sevColor = const Color(0xFFFFA000);
                              sevEmoji = '⚠️';
                              break;
                            case EcoSeverity.bajo:
                              sevColor = const Color(0xFF388E3C);
                              sevEmoji = '🍃';
                              break;
                          }

                          // Categoría Styling
                          IconData catIcon;
                          switch (alert.category) {
                            case EcoCategory.mineria:
                              catIcon = Icons.terrain_rounded;
                              break;
                            case EcoCategory.basura:
                              catIcon = Icons.delete_outline_rounded;
                              break;
                            case EcoCategory.agua:
                              catIcon = Icons.water_drop_rounded;
                              break;
                            case EcoCategory.aire:
                              catIcon = Icons.air_rounded;
                              break;
                          }

                          final isSelected = appState.selectedAlert?.id == alert.id;

                          return GestureDetector(
                            onTap: () {
                              appState.selectedAlert = alert;
                              _mapController.move(
                                LatLng(alert.latitude, alert.longitude),
                                15.5,
                              );
                            },
                            child: AnimatedContainer(
                              duration: const Duration(milliseconds: 200),
                              width: 250,
                              margin: const EdgeInsets.only(right: 12.0),
                              padding: const EdgeInsets.all(12.0),
                              decoration: BoxDecoration(
                                color: isSelected ? const Color(0xFFE0F2FE) : Colors.white,
                                borderRadius: BorderRadius.circular(12),
                                border: Border.all(
                                  color: isSelected ? const Color(0xFF0288D1) : const Color(0xFFE2E8F0),
                                  width: 1.5,
                                ),
                                boxShadow: [
                                  BoxShadow(
                                    color: Colors.black.withOpacity(0.04),
                                    blurRadius: 4,
                                    offset: const Offset(0, 2),
                                  )
                                ],
                              ),
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  // Fila superior (Severidad & Cierre/Icono)
                                  Row(
                                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                                    children: [
                                      Container(
                                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                                        decoration: BoxDecoration(
                                          color: sevColor.withOpacity(0.12),
                                          borderRadius: BorderRadius.circular(6),
                                        ),
                                        child: Text(
                                          '$sevEmoji ${alert.severity.toString().split('.').last.toUpperCase()}',
                                          style: GoogleFonts.outfit(
                                            fontSize: 9,
                                            fontWeight: FontWeight.bold,
                                            color: sevColor,
                                          ),
                                        ),
                                      ),
                                      Icon(catIcon, color: const Color(0xFF0288D1), size: 16),
                                    ],
                                  ),
                                  const SizedBox(height: 8),
                                  
                                  // Título
                                  Text(
                                    alert.title,
                                    maxLines: 1,
                                    overflow: TextOverflow.ellipsis,
                                    style: GoogleFonts.outfit(
                                      fontSize: 13,
                                      fontWeight: FontWeight.bold,
                                      color: const Color(0xFF0F172A),
                                    ),
                                  ),
                                  const SizedBox(height: 4),
                                  
                                  // Descripción
                                  Expanded(
                                    child: Text(
                                      alert.description,
                                      maxLines: 2,
                                      overflow: TextOverflow.ellipsis,
                                      style: GoogleFonts.outfit(
                                        fontSize: 11,
                                        color: const Color(0xFF475569),
                                      ),
                                    ),
                                  ),
                                  const SizedBox(height: 4),
                                  
                                  // Dirección / Ubicación
                                  Row(
                                    children: [
                                      const Icon(Icons.location_on_outlined, color: Color(0xFF64748B), size: 12),
                                      const SizedBox(width: 4),
                                      Expanded(
                                        child: Text(
                                          alert.address,
                                          overflow: TextOverflow.ellipsis,
                                          style: GoogleFonts.outfit(
                                            fontSize: 9,
                                            color: const Color(0xFF64748B),
                                            fontStyle: FontStyle.italic,
                                          ),
                                        ),
                                      ),
                                    ],
                                  ),
                                ],
                              ),
                            ),
                          );
                        },
                      ),
                    ),
              ),
            ],
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildLegendToggleItem(
    BuildContext context,
    AppState appState,
    EcoCategory category,
    String label,
    IconData icon,
    Color color,
  ) {
    final isSelected = appState.selectedCategories.contains(category);

    return GestureDetector(
      onTap: () {
        appState.toggleCategory(category);
      },
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        child: AnimatedOpacity(
          duration: const Duration(milliseconds: 150),
          opacity: isSelected ? 1.0 : 0.4,
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 2.0),
            child: Row(
              children: [
                Icon(icon, color: color, size: 14),
                const SizedBox(width: 6),
                Expanded(
                  child: Text(
                    label,
                    style: GoogleFonts.outfit(
                      fontSize: 11,
                      fontWeight: isSelected ? FontWeight.bold : FontWeight.w500,
                      color: const Color(0xFF334155),
                    ),
                  ),
                ),
                Icon(
                  isSelected ? Icons.check_box_rounded : Icons.check_box_outline_blank_rounded,
                  color: isSelected ? color : const Color(0xFF94A3B8),
                  size: 14,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildLegendSeverityToggleItem(
    BuildContext context,
    AppState appState,
    EcoSeverity severity,
    String label,
    Color color,
  ) {
    final isSelected = appState.selectedSeverities.contains(severity);

    return GestureDetector(
      onTap: () {
        appState.toggleSeverity(severity);
      },
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        child: AnimatedOpacity(
          duration: const Duration(milliseconds: 150),
          opacity: isSelected ? 1.0 : 0.4,
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 2.0),
            child: Row(
              children: [
                Container(
                  width: 8,
                  height: 8,
                  decoration: BoxDecoration(
                    color: color,
                    shape: BoxShape.circle,
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    label,
                    style: GoogleFonts.outfit(
                      fontSize: 11,
                      fontWeight: isSelected ? FontWeight.bold : FontWeight.w500,
                      color: const Color(0xFF334155),
                    ),
                  ),
                ),
                Icon(
                  isSelected ? Icons.check_box_rounded : Icons.check_box_outline_blank_rounded,
                  color: isSelected ? color : const Color(0xFF94A3B8),
                  size: 14,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

// Pintado del Marcador de Mapa Personalizado
class _MapPin extends StatelessWidget {
  final EcoCategory category;
  final EcoSeverity severity;
  final String status;
  final bool isSelected;

  const _MapPin({
    required this.category,
    required this.severity,
    required this.status,
    required this.isSelected,
  });

  @override
  Widget build(BuildContext context) {
    // Definir colores
    Color color;
    if (status == 'solved') {
      color = const Color(0xFF10B981); // Emerald Green for Solved Alerts
    } else {
      switch (severity) {
        case EcoSeverity.critico:
          color = const Color(0xFFD32F2F);
          break;
        case EcoSeverity.medio:
          color = const Color(0xFFFFA000);
          break;
        case EcoSeverity.bajo:
          color = const Color(0xFF388E3C);
          break;
      }
    }

    // Definir iconos
    IconData icon;
    switch (category) {
      case EcoCategory.mineria:
        icon = Icons.terrain_rounded;
        break;
      case EcoCategory.basura:
        icon = Icons.delete_outline_rounded;
        break;
      case EcoCategory.agua:
        icon = Icons.water_drop_rounded;
        break;
      case EcoCategory.aire:
        icon = Icons.air_rounded;
        break;
    }

    return Center(
      child: Stack(
        alignment: Alignment.center,
        children: [
          if (severity == EcoSeverity.critico && status != 'solved' && status != 'dismissed')
            const PulsingCriticalDot(isRadarRing: true),

          // Sombra o pulso si está seleccionado
          if (isSelected)
            Container(
              width: 42,
              height: 42,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: color.withOpacity(0.3),
                boxShadow: [
                  BoxShadow(
                    color: color,
                    blurRadius: 8,
                    spreadRadius: 2,
                  )
                ],
              ),
            ),
          
          // Cuerpo del Pin (Gota de ubicación)
          Container(
            width: 32,
            height: 32,
            decoration: BoxDecoration(
              color: isSelected ? Colors.white : color,
              shape: BoxShape.circle,
              border: Border.all(
                color: isSelected ? color : Colors.white,
                width: 2.5,
              ),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.18),
                  blurRadius: 4,
                  offset: const Offset(0, 2),
                )
              ],
            ),
            child: Center(
              child: Icon(
                icon,
                color: isSelected ? color : Colors.white,
                size: 16,
              ),
            ),
          ),

          // Punta inferior
          Positioned(
            bottom: 0,
            child: CustomPaint(
              size: const Size(8, 6),
              painter: _TrianglePainter(color: isSelected ? Colors.white : color),
            ),
          )
        ],
      ),
    );
  }
}

class _TrianglePainter extends CustomPainter {
  final Color color;

  _TrianglePainter({required this.color});

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..style = PaintingStyle.fill;

    final path = ui.Path()
      ..moveTo(0, 0)
      ..lineTo(size.width, 0)
      ..lineTo(size.width / 2, size.height)
      ..close();

    canvas.drawPath(path, paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}

// Tarjeta de detalles de alerta
class _AlertDetailCard extends StatelessWidget {
  final EcoAlert alert;
  final VoidCallback onClose;

  const _AlertDetailCard({
    required this.alert,
    required this.onClose,
  });

  void _showHistoryTimelineDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
        title: Row(
          children: [
            const Icon(Icons.history_rounded, color: Color(0xFF0288D1)),
            const SizedBox(width: 8),
            Text(
              'Bitácora y Auditoría de Caso',
              style: GoogleFonts.outfit(fontWeight: FontWeight.bold, fontSize: 16),
            ),
          ],
        ),
        content: SizedBox(
          width: 400,
          child: alert.parsedHistory.isEmpty
              ? Text(
                  'No hay historial registrado para esta alerta.',
                  style: GoogleFonts.outfit(color: const Color(0xFF64748B), fontSize: 13),
                )
              : ListView.builder(
                  shrinkWrap: true,
                  itemCount: alert.parsedHistory.length,
                  itemBuilder: (context, index) {
                    final item = alert.parsedHistory[index];
                    final date = DateTime.tryParse(item['date'] ?? '') ?? DateTime.now();
                    final formattedDate = "${date.day}/${date.month}/${date.year} ${date.hour.toString().padLeft(2, '0')}:${date.minute.toString().padLeft(2, '0')}";
                    final actionText = item['action'] ?? '';
                    
                    // Decidir icono e indicador según el contenido
                    IconData icon = Icons.info_outline;
                    Color color = Colors.blue;
                    if (actionText.contains('Reportado')) {
                      icon = Icons.campaign_rounded;
                      color = Colors.orange;
                    } else if (actionText.contains('Transferido')) {
                      icon = Icons.swap_horiz_rounded;
                      color = Colors.purple;
                    } else if (actionText.contains('Solucionado')) {
                      icon = Icons.check_circle_rounded;
                      color = Colors.green;
                    } else if (actionText.contains('Descartado')) {
                      icon = Icons.delete_outline_rounded;
                      color = Colors.red;
                    }

                    return Padding(
                      padding: const EdgeInsets.symmetric(vertical: 8.0),
                      child: Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Column(
                            children: [
                              Container(
                                padding: const EdgeInsets.all(6),
                                decoration: BoxDecoration(
                                  color: color.withOpacity(0.12),
                                  shape: BoxShape.circle,
                                ),
                                child: Icon(icon, color: color, size: 14),
                              ),
                              if (index < alert.parsedHistory.length - 1)
                                Container(
                                  width: 2,
                                  height: 24,
                                  color: const Color(0xFFCBD5E1), // Slate 300
                                ),
                            ],
                          ),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  actionText,
                                  style: GoogleFonts.outfit(
                                    fontWeight: FontWeight.bold,
                                    fontSize: 12,
                                    color: const Color(0xFF1E293B),
                                  ),
                                ),
                                const SizedBox(height: 2),
                                Text(
                                  formattedDate,
                                  style: GoogleFonts.outfit(
                                    fontSize: 10,
                                    color: const Color(0xFF64748B),
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ],
                      ),
                    );
                  },
                ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(
              'Cerrar',
              style: GoogleFonts.outfit(fontWeight: FontWeight.bold, color: const Color(0xFF0288D1)),
            ),
          ),
        ],
      ),
    );
  }

  void _showResolveDialog(BuildContext context) {
    final commentController = TextEditingController();
    final appState = Provider.of<AppState>(context, listen: false);

    String? selectedFileName;
    List<int>? selectedFileBytes;
    bool isUploading = false;

    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(18)),
          title: Row(
            children: [
              const Icon(Icons.verified_user_rounded, color: Color(0xFF0288D1)),
              const SizedBox(width: 8),
              Text(
                'Resolver Incidente',
                style: GoogleFonts.outfit(fontWeight: FontWeight.bold),
              ),
            ],
          ),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Ingresa la evidencia de los trabajos realizados para solucionar esta alerta ecológica.',
                style: GoogleFonts.outfit(fontSize: 13, color: const Color(0xFF475569)),
              ),
              const SizedBox(height: 16),
              TextField(
                controller: commentController,
                maxLines: 3,
                decoration: InputDecoration(
                  labelText: 'Comentario de solución',
                  labelStyle: GoogleFonts.outfit(fontSize: 13),
                  border: OutlineInputBorder(borderRadius: BorderRadius.circular(12)),
                  alignLabelWithHint: true,
                ),
              ),
              const SizedBox(height: 16),
              Text(
                'FOTO EVIDENCIA (REAL)',
                style: GoogleFonts.outfit(
                  fontSize: 10,
                  fontWeight: FontWeight.bold,
                  color: const Color(0xFF64748B),
                  letterSpacing: 0.5,
                ),
              ),
              const SizedBox(height: 6),
              if (selectedFileName != null) ...[
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                  decoration: BoxDecoration(
                    color: const Color(0xFFF1F5F9),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: const Color(0xFFE2E8F0)),
                  ),
                  child: Row(
                    children: [
                      const Icon(Icons.image_outlined, color: Color(0xFF0288D1), size: 16),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          selectedFileName!,
                          overflow: TextOverflow.ellipsis,
                          style: GoogleFonts.outfit(fontSize: 11, fontWeight: FontWeight.w500),
                        ),
                      ),
                      IconButton(
                        icon: const Icon(Icons.close_rounded, size: 16, color: Color(0xFFEF4444)),
                        onPressed: () {
                          setState(() {
                            selectedFileName = null;
                            selectedFileBytes = null;
                          });
                        },
                      )
                    ],
                  ),
                ),
                const SizedBox(height: 12),
              ],
              SizedBox(
                width: double.infinity,
                child: OutlinedButton.icon(
                  onPressed: isUploading
                      ? null
                      : () async {
                          final result = await FilePicker.platform.pickFiles(type: FileType.image);
                          if (result != null && result.files.first.bytes != null) {
                            setState(() {
                              selectedFileName = result.files.first.name;
                              selectedFileBytes = result.files.first.bytes;
                            });
                          }
                        },
                  icon: const Icon(Icons.camera_alt_rounded, size: 14),
                  label: Text(
                    selectedFileName == null ? 'Seleccionar Imagen Evidencia' : 'Cambiar Imagen',
                    style: GoogleFonts.outfit(fontSize: 12, fontWeight: FontWeight.bold),
                  ),
                  style: OutlinedButton.styleFrom(
                    side: const BorderSide(color: Color(0xFF0288D1)),
                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                  ),
                ),
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: isUploading ? null : () => Navigator.pop(context),
              child: Text('Cancelar', style: GoogleFonts.outfit(color: const Color(0xFF64748B))),
            ),
            ElevatedButton(
              onPressed: (isUploading || commentController.text.trim().isEmpty || selectedFileBytes == null)
                  ? null
                  : () async {
                      setState(() {
                        isUploading = true;
                      });

                      // Subir imagen real al backend de Django
                      final imageUrl = await appState.uploadImage(selectedFileBytes!, selectedFileName!);

                      if (imageUrl == null) {
                        setState(() {
                          isUploading = false;
                        });
                        if (context.mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                              content: Text(
                                'Error al subir la imagen. Inténtelo de nuevo.',
                                style: GoogleFonts.outfit(),
                              ),
                              backgroundColor: const Color(0xFFEF4444),
                            ),
                          );
                        }
                        return;
                      }

                      final success = await appState.resolveAlert(
                        alert.id,
                        commentController.text.trim(),
                        imageUrl,
                      );

                      if (context.mounted) {
                        Navigator.pop(context);
                        onClose();
                        ScaffoldMessenger.of(context).showSnackBar(
                          SnackBar(
                            content: Text(
                              success ? 'Incidente resuelto con evidencia.' : 'Error al guardar la resolución.',
                              style: GoogleFonts.outfit(),
                            ),
                            backgroundColor: success ? const Color(0xFF10B981) : const Color(0xFFEF4444),
                          ),
                        );
                      }
                    },
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF0288D1),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
              ),
              child: isUploading
                  ? const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                    )
                  : Text(
                      'Resolver',
                      style: GoogleFonts.outfit(color: Colors.white, fontWeight: FontWeight.bold),
                    ),
            ),
          ],
        ),
      ),
    );
  }

  void _showTransferDialog(BuildContext context) {
    final appState = Provider.of<AppState>(context, listen: false);
    EcoDistrict selectedTransferDistrict = alert.district;
    bool isSaving = false;

    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
          title: Row(
            children: [
              const Icon(Icons.swap_horiz_rounded, color: Color(0xFF0288D1)),
              const SizedBox(width: 8),
              Text(
                'Reenviar Caso (Transferencia)',
                style: GoogleFonts.outfit(fontWeight: FontWeight.bold),
              ),
            ],
          ),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Si el caso pertenece a la jurisdicción de otro municipio, transfiérelo. La municipalidad asignada se hará responsable del caso.',
                style: GoogleFonts.outfit(fontSize: 13, color: const Color(0xFF475569)),
              ),
              const SizedBox(height: 20),
              Text(
                'SELECCIONAR NUEVA MUNICIPALIDAD:',
                style: GoogleFonts.outfit(fontSize: 10, fontWeight: FontWeight.bold, color: const Color(0xFF64748B)),
              ),
              const SizedBox(height: 8),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 12),
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: const Color(0xFFE2E8F0)),
                ),
                child: DropdownButtonHideUnderline(
                  child: DropdownButton<EcoDistrict>(
                    value: selectedTransferDistrict,
                    isExpanded: true,
                    onChanged: (newVal) {
                      if (newVal != null) {
                        setDialogState(() {
                          selectedTransferDistrict = newVal;
                        });
                      }
                    },
                    items: const [
                      DropdownMenuItem(value: EcoDistrict.chaupimarca, child: Text('🏛️ Municipalidad Provincial de Pasco')),
                      DropdownMenuItem(value: EcoDistrict.yanacancha, child: Text('🏢 Municipalidad Distrital de Yanacancha')),
                      DropdownMenuItem(value: EcoDistrict.simonBolivar, child: Text('🚜 Municipalidad Distrital de Simón Bolívar')),
                    ],
                  ),
                ),
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: isSaving ? null : () => Navigator.pop(context),
              child: Text('Cancelar', style: GoogleFonts.outfit(color: const Color(0xFF64748B))),
            ),
            ElevatedButton(
              onPressed: (isSaving || selectedTransferDistrict == alert.district)
                  ? null
                  : () async {
                      setDialogState(() {
                        isSaving = true;
                      });

                      final success = await appState.transferAlert(alert.id, selectedTransferDistrict);

                      if (context.mounted) {
                        Navigator.pop(context);
                        onClose();
                        ScaffoldMessenger.of(context).showSnackBar(
                          SnackBar(
                            backgroundColor: success ? const Color(0xFF0288D1) : const Color(0xFFEF4444),
                            content: Text(
                              success
                                  ? 'Caso reenviado con éxito.'
                                  : 'Fallo al reenviar el caso.',
                              style: GoogleFonts.outfit(fontWeight: FontWeight.bold),
                            ),
                          ),
                        );
                      }
                    },
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF0288D1),
                foregroundColor: Colors.white,
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
              ),
              child: isSaving
                  ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                  : Text('Reenviar Caso', style: GoogleFonts.outfit(fontWeight: FontWeight.bold, color: Colors.white)),
            ),
          ],
        ),
      ),
    );
  }

  void _showDismissDialog(BuildContext context) {
    final appState = Provider.of<AppState>(context, listen: false);
    final reasonController = TextEditingController();
    bool isSaving = false;

    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
          title: Row(
            children: [
              const Icon(Icons.delete_outline_rounded, color: Color(0xFFEF4444)),
              const SizedBox(width: 8),
              Text(
                'Descartar / Ocultar Reporte',
                style: GoogleFonts.outfit(fontWeight: FontWeight.bold),
              ),
            ],
          ),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Utiliza esta opción si el caso fue enviado por error, es spam o duplicado. El reporte se ocultará de los mapas públicos.',
                style: GoogleFonts.outfit(fontSize: 13, color: const Color(0xFF475569)),
              ),
              const SizedBox(height: 16),
              TextField(
                controller: reasonController,
                maxLines: 2,
                style: GoogleFonts.outfit(fontSize: 13),
                decoration: InputDecoration(
                  labelText: 'Escribe el motivo del descarte',
                  labelStyle: GoogleFonts.outfit(fontSize: 12),
                  border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
                  alignLabelWithHint: true,
                ),
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: isSaving ? null : () => Navigator.pop(context),
              child: Text('Cancelar', style: GoogleFonts.outfit(color: const Color(0xFF64748B))),
            ),
            ElevatedButton(
              onPressed: (isSaving || reasonController.text.trim().isEmpty)
                  ? null
                  : () async {
                      setDialogState(() {
                        isSaving = true;
                      });

                      final success = await appState.dismissAlert(alert.id, reasonController.text.trim());

                      if (context.mounted) {
                        Navigator.pop(context);
                        onClose();
                        ScaffoldMessenger.of(context).showSnackBar(
                          SnackBar(
                            backgroundColor: success ? const Color(0xFF64748B) : const Color(0xFFEF4444),
                            content: Text(
                              success
                                  ? 'Reporte descartado y ocultado.'
                                  : 'Fallo al descartar el reporte.',
                              style: GoogleFonts.outfit(fontWeight: FontWeight.bold),
                            ),
                          ),
                        );
                      }
                    },
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFFEF4444),
                foregroundColor: Colors.white,
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
              ),
              child: isSaving
                  ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                  : Text('Confirmar Descarte', style: GoogleFonts.outfit(fontWeight: FontWeight.bold, color: Colors.white)),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    // Severidad Styling
    Color sevColor;
    String sevText;
    String sevEmoji;
    switch (alert.severity) {
      case EcoSeverity.critico:
        sevColor = const Color(0xFFD32F2F);
        sevText = 'CRÍTICO';
        sevEmoji = '💀';
        break;
      case EcoSeverity.medio:
        sevColor = const Color(0xFFFFA000);
        sevText = 'MEDIO';
        sevEmoji = '⚠️';
        break;
      case EcoSeverity.bajo:
        sevColor = const Color(0xFF388E3C);
        sevText = 'BAJO';
        sevEmoji = '🍃';
        break;
    }

    // Status Styling
    Color statusColor;
    String statusText;
    IconData statusIcon;
    switch (alert.status) {
      case 'solved':
        statusColor = const Color(0xFF10B981);
        statusText = 'RESUELTA';
        statusIcon = Icons.check_circle_rounded;
        break;
      case 'verified':
        statusColor = const Color(0xFF3B82F6);
        statusText = 'VERIFICADA';
        statusIcon = Icons.verified_rounded;
        break;
      case 'pending':
      default:
        statusColor = const Color(0xFFF59E0B);
        statusText = 'PENDIENTE';
        statusIcon = Icons.pending_rounded;
        break;
    }

    // Categoría Text
    String catText;
    IconData catIcon;
    switch (alert.category) {
      case EcoCategory.mineria:
        catText = 'Minería';
        catIcon = Icons.terrain_rounded;
        break;
      case EcoCategory.basura:
        catText = 'Basura';
        catIcon = Icons.delete_outline_rounded;
        break;
      case EcoCategory.agua:
        catText = 'Agua';
        catIcon = Icons.water_drop_rounded;
        break;
      case EcoCategory.aire:
        catText = 'Aire';
        catIcon = Icons.air_rounded;
        break;
    }

    final formattedDate = "${alert.createdAt.day}/${alert.createdAt.month}/${alert.createdAt.year} ${alert.createdAt.hour.toString().padLeft(2, '0')}:${alert.createdAt.minute.toString().padLeft(2, '0')}";

    return Container(
      width: 320,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: const Color(0xFFE2E8F0), width: 1.5),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.1),
            blurRadius: 16,
            offset: const Offset(0, 4),
          )
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header (Severidad y Categoría + botón Cerrar)
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Row(
                children: [
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: sevColor.withOpacity(0.12),
                      borderRadius: BorderRadius.circular(8),
                      border: Border.all(color: sevColor.withOpacity(0.3), width: 1),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text(sevEmoji, style: const TextStyle(fontSize: 10)),
                        const SizedBox(width: 4),
                        Text(
                          sevText,
                          style: GoogleFonts.outfit(
                            color: sevColor,
                            fontSize: 10,
                            fontWeight: FontWeight.w800,
                            letterSpacing: 0.5,
                          ),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(width: 8),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: statusColor.withOpacity(0.12),
                      borderRadius: BorderRadius.circular(8),
                      border: Border.all(color: statusColor.withOpacity(0.3), width: 1),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(statusIcon, color: statusColor, size: 10),
                        const SizedBox(width: 4),
                        Text(
                          statusText,
                          style: GoogleFonts.outfit(
                            color: statusColor,
                            fontSize: 10,
                            fontWeight: FontWeight.w800,
                            letterSpacing: 0.5,
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
              Row(
                children: [
                  IconButton(
                    constraints: const BoxConstraints(),
                    padding: EdgeInsets.zero,
                    icon: const Icon(Icons.history_rounded, color: Color(0xFF64748B), size: 18),
                    tooltip: 'Ver Bitácora de Auditoría',
                    onPressed: () => _showHistoryTimelineDialog(context),
                  ),
                  const SizedBox(width: 8),
                  IconButton(
                    constraints: const BoxConstraints(),
                    padding: EdgeInsets.zero,
                    icon: const Icon(Icons.close_rounded, color: Color(0xFF94A3B8), size: 18),
                    onPressed: onClose,
                  ),
                ],
              ),
            ],
          ),
          const SizedBox(height: 12),
          
          if (alert.imageUrl != null) ...[
            GestureDetector(
              onTap: () {
                showDialog(
                  context: context,
                  builder: (context) => _ImageLightboxDialog(
                    imageName: alert.imageUrl!,
                    severityColor: sevColor,
                    categoryIcon: catIcon,
                  ),
                );
              },
              child: MouseRegion(
                cursor: SystemMouseCursors.click,
                child: Tooltip(
                  message: 'Haz clic para ampliar la imagen',
                  child: Container(
                    height: 120,
                    decoration: BoxDecoration(
                      borderRadius: BorderRadius.circular(12),
                      gradient: LinearGradient(
                        colors: [
                          sevColor.withOpacity(0.35),
                          sevColor.withOpacity(0.12),
                        ],
                        begin: Alignment.topLeft,
                        end: Alignment.bottomRight,
                      ),
                      border: Border.all(color: sevColor.withOpacity(0.2), width: 1.5),
                    ),
                    child: Center(
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(catIcon, color: sevColor, size: 36),
                          const SizedBox(height: 8),
                          Padding(
                            padding: const EdgeInsets.symmetric(horizontal: 16.0),
                            child: Row(
                              mainAxisAlignment: MainAxisAlignment.center,
                              children: [
                                const Icon(Icons.zoom_in, size: 14, color: Color(0xFF64748B)),
                                const SizedBox(width: 4),
                                Expanded(
                                  child: Text(
                                    alert.imageUrl!,
                                    textAlign: TextAlign.center,
                                    maxLines: 1,
                                    overflow: TextOverflow.ellipsis,
                                    style: GoogleFonts.outfit(
                                      color: const Color(0xFF0F172A),
                                      fontSize: 12,
                                      fontWeight: FontWeight.bold,
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ),
            const SizedBox(height: 12),
          ],
          
          // Título
          Text(
            alert.title,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
            style: GoogleFonts.outfit(
              color: const Color(0xFF0F172A),
              fontSize: 16,
              fontWeight: FontWeight.bold,
            ),
          ),
          const SizedBox(height: 6),
          
          // Categoría e Info de Ubicación/Fecha
          Row(
            children: [
              Icon(catIcon, color: const Color(0xFF0288D1), size: 14),
              const SizedBox(width: 4),
              Text(
                catText,
                style: GoogleFonts.outfit(
                  color: const Color(0xFF0288D1),
                  fontSize: 12,
                  fontWeight: FontWeight.w600,
                ),
              ),
              const SizedBox(width: 12),
              const Icon(Icons.schedule_rounded, color: Color(0xFF64748B), size: 14),
              const SizedBox(width: 4),
              Expanded(
                child: Text(
                  formattedDate,
                  overflow: TextOverflow.ellipsis,
                  style: GoogleFonts.outfit(
                    color: const Color(0xFF64748B),
                    fontSize: 11,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          
          // Descripción
          Text(
            alert.description,
            maxLines: 4,
            overflow: TextOverflow.ellipsis,
            style: GoogleFonts.outfit(
              color: const Color(0xFF334155),
              fontSize: 13,
              height: 1.4,
            ),
          ),
          
          // Dirección
          const SizedBox(height: 12),
          Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: const Color(0xFFF1F5F9),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Row(
              children: [
                const Icon(Icons.location_on_rounded, color: Color(0xFF0288D1), size: 14),
                const SizedBox(width: 6),
                Expanded(
                  child: Text(
                    alert.address,
                    style: GoogleFonts.outfit(
                      color: const Color(0xFF475569),
                      fontSize: 11,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
              ],
            ),
          ),

          // Sección de Evidencia de Resolución
          if (alert.status == 'solved') ...[
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: const Color(0xFFECFDF5),
                borderRadius: BorderRadius.circular(12),
                border: Border.all(color: const Color(0xFFA7F3D0)),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.check_circle_rounded, color: Color(0xFF059669), size: 16),
                      const SizedBox(width: 6),
                      Text(
                        'RESUELTA POR AUTORIDAD',
                        style: GoogleFonts.outfit(
                          color: const Color(0xFF065F46),
                          fontSize: 11,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ],
                  ),
                  if (alert.resolutionComment != null && alert.resolutionComment!.isNotEmpty) ...[
                    const SizedBox(height: 6),
                    Text(
                      alert.resolutionComment!,
                      style: GoogleFonts.outfit(
                        color: const Color(0xFF047857),
                        fontSize: 11,
                        height: 1.3,
                      ),
                    ),
                  ],
                  const SizedBox(height: 10),
                  Text(
                    'COMPARACIÓN VISUAL:',
                    style: GoogleFonts.outfit(
                      fontSize: 9,
                      fontWeight: FontWeight.bold,
                      color: const Color(0xFF047857),
                      letterSpacing: 0.5,
                    ),
                  ),
                  const SizedBox(height: 6),
                  Row(
                    children: [
                      // Antes (Denuncia)
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'Antes (Denuncia)',
                              style: GoogleFonts.outfit(
                                fontSize: 9,
                                fontWeight: FontWeight.bold,
                                color: const Color(0xFF64748B),
                              ),
                            ),
                            const SizedBox(height: 4),
                            ClipRRect(
                              borderRadius: BorderRadius.circular(8),
                              child: Container(
                                height: 70,
                                color: const Color(0xFFF1F5F9),
                                width: double.infinity,
                                child: alert.imageUrl != null
                                    ? Image.network(
                                        alert.imageUrl!,
                                        fit: BoxFit.cover,
                                        errorBuilder: (context, e, s) => const Icon(Icons.broken_image, size: 20, color: Color(0xFF94A3B8)),
                                      )
                                    : const Icon(Icons.image_not_supported_rounded, size: 20, color: Color(0xFF94A3B8)),
                              ),
                            ),
                          ],
                        ),
                      ),
                      const SizedBox(width: 8),
                      // Después (Solución)
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'Después (Solución)',
                              style: GoogleFonts.outfit(
                                fontSize: 9,
                                fontWeight: FontWeight.bold,
                                color: const Color(0xFF047857),
                              ),
                            ),
                            const SizedBox(height: 4),
                            ClipRRect(
                              borderRadius: BorderRadius.circular(8),
                              child: Container(
                                height: 70,
                                color: const Color(0xFFD1FAE5),
                                width: double.infinity,
                                child: (alert.resolutionImageUrl != null && alert.resolutionImageUrl!.isNotEmpty)
                                    ? Image.network(
                                        alert.resolutionImageUrl!,
                                        fit: BoxFit.cover,
                                        errorBuilder: (context, e, s) => const Icon(Icons.broken_image, size: 20, color: Color(0xFF059669)),
                                      )
                                    : const Icon(Icons.image_not_supported_rounded, size: 20, color: Color(0xFF059669)),
                              ),
                            ),
                          ],
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ] else if (Provider.of<AppState>(context).isLoggedInAuthority) ...[
            const SizedBox(height: 12),
            Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                if (alert.status != 'solved' && alert.status != 'dismissed') ...[
                  SizedBox(
                    width: double.infinity,
                    height: 36,
                    child: ElevatedButton.icon(
                      onPressed: () => _showResolveDialog(context),
                      icon: const Icon(Icons.verified_user_rounded, size: 14, color: Colors.white),
                      label: Text(
                        'Resolver Alerta (Autoridad)',
                        style: GoogleFonts.outfit(fontSize: 12, fontWeight: FontWeight.bold, color: Colors.white),
                      ),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: const Color(0xFF10B981),
                        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                        elevation: 0,
                      ),
                    ),
                  ),
                  const SizedBox(height: 8),
                ],
                Row(
                  children: [
                    Expanded(
                      child: SizedBox(
                        height: 36,
                        child: OutlinedButton.icon(
                          onPressed: () => _showTransferDialog(context),
                          icon: const Icon(Icons.swap_horiz_rounded, size: 14),
                          label: Text(
                            'Reenviar',
                            style: GoogleFonts.outfit(fontSize: 12, fontWeight: FontWeight.bold),
                          ),
                          style: OutlinedButton.styleFrom(
                            side: const BorderSide(color: Color(0xFF0288D1)),
                            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                          ),
                        ),
                      ),
                    ),
                    if (alert.status != 'dismissed') ...[
                      const SizedBox(width: 8),
                      Expanded(
                        child: SizedBox(
                          height: 36,
                          child: OutlinedButton.icon(
                            onPressed: () => _showDismissDialog(context),
                            icon: const Icon(Icons.delete_outline_rounded, size: 14, color: Color(0xFFEF4444)),
                            label: Text(
                              'Descartar',
                              style: GoogleFonts.outfit(fontSize: 12, fontWeight: FontWeight.bold, color: Color(0xFFEF4444)),
                            ),
                            style: OutlinedButton.styleFrom(
                              side: const BorderSide(color: Color(0xFFEF4444)),
                              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                            ),
                          ),
                        ),
                      ),
                    ],
                  ],
                ),
              ],
            ),
          ] else ...[
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: const Color(0xFFF1F5F9),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: const Color(0xFFE2E8F0)),
              ),
              child: Row(
                children: [
                  const Icon(Icons.info_outline_rounded, color: Color(0xFF64748B), size: 14),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      'Inicia sesión como autoridad en la barra lateral para resolver esta alerta.',
                      style: GoogleFonts.outfit(fontSize: 10, color: const Color(0xFF475569), fontWeight: FontWeight.w500),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class _ImageLightboxDialog extends StatelessWidget {
  final String imageName;
  final Color severityColor;
  final IconData categoryIcon;

  const _ImageLightboxDialog({
    required this.imageName,
    required this.severityColor,
    required this.categoryIcon,
  });

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: Colors.transparent,
      insetPadding: const EdgeInsets.all(16),
      child: Stack(
        alignment: Alignment.center,
        children: [
          GestureDetector(
            onTap: () => Navigator.of(context).pop(),
            child: Container(
              color: Colors.black.withOpacity(0.85),
            ),
          ),
          Container(
            constraints: const BoxConstraints(maxWidth: 600),
            height: 400,
            margin: const EdgeInsets.symmetric(horizontal: 24),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(24),
              border: Border.all(color: severityColor.withOpacity(0.5), width: 2),
              gradient: LinearGradient(
                colors: [
                  severityColor.withOpacity(0.7),
                  const Color(0xFF0F172A),
                ],
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
              ),
            ),
            child: Stack(
              children: [
                Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(categoryIcon, color: Colors.white, size: 80),
                      const SizedBox(height: 16),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 32.0),
                        child: Text(
                          imageName,
                          textAlign: TextAlign.center,
                          style: GoogleFonts.outfit(
                            color: Colors.white,
                            fontSize: 18,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        '🔍 Evidencia fotográfica de campo',
                        style: GoogleFonts.outfit(
                          color: Colors.white.withOpacity(0.7),
                          fontSize: 12,
                        ),
                      ),
                    ],
                  ),
                ),
                Positioned(
                  top: 16,
                  right: 16,
                  child: IconButton(
                    icon: const Icon(Icons.close_rounded, color: Colors.white, size: 28),
                    onPressed: () => Navigator.of(context).pop(),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
