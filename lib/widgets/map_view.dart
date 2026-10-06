import 'package:flutter/material.dart';
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
  String? _lastSelectedAlertId;
  bool _isDarkMap = false; // MODO CLARO por defecto según pedido del usuario
  bool _isListExpanded = true;
  bool _showWasteLayer = false;

  // Datos simulados de Contenedores y Botaderos en Cerro de Pasco
  final List<WastePoint> _wastePoints = [
    const WastePoint(
      id: 'waste-1',
      name: 'Punto Limpio de Reciclaje - Plaza Yanacancha',
      description: 'Estación ecológica municipal con módulos diferenciados para reciclaje de botellas plásticas (PET), latas de aluminio y cartón.',
      address: 'Plaza Principal de Yanacancha s/n, Yanacancha',
      location: LatLng(-10.6625, -76.2555),
      type: WasteType.recyclable,
      truckSchedule: 'Lunes, Miércoles y Viernes de 18:00 a 20:00',
      imageUrl: 'https://images.unsplash.com/photo-1532996122724-e3c354a0b15b?q=80&w=800&auto=format&fit=crop',
      recommendations: 'Depositar botellas plásticas enjuagadas y aplastadas. No arrojar residuos orgánicos ni desechos sanitarios en estos módulos.',
    ),
    const WastePoint(
      id: 'waste-2',
      name: 'Punto de Acopio Orgánico - Av. Los Próceres',
      description: 'Contenedores marrones especiales para residuos biodegradables y restos orgánicos domiciliarios para el programa de compostaje local.',
      address: 'Av. Los Próceres cuadra 4 (Frente al Mercado El Minero), Yanacancha',
      location: LatLng(-10.6640, -76.2530),
      type: WasteType.organic,
      truckSchedule: 'Martes, Jueves y Sábado de 06:00 a 08:30',
      imageUrl: 'https://images.unsplash.com/photo-1542601906990-b4d3fb778b09?q=80&w=800&auto=format&fit=crop',
      recommendations: 'Apto solo para cáscaras de frutas, verduras, hojas secas y restos de café. Usar bolsas de papel o biodegradables.',
    ),
    const WastePoint(
      id: 'waste-3',
      name: 'Contenedor Comunal - Hospital Huariaca',
      description: 'Gran contenedor cerrado para residuos sólidos comunes de vecinos de la zona residencial y comercial.',
      address: 'Jirón Progreso 230 (Costado del Centro de Salud), Yanacancha',
      location: LatLng(-10.6685, -76.2580),
      type: WasteType.general,
      truckSchedule: 'Diario (Lunes a Domingo) de 07:00 a 09:00',
      imageUrl: 'https://images.unsplash.com/photo-1611284446314-60a58ac0deb9?q=80&w=800&auto=format&fit=crop',
      recommendations: 'Amarrar bien las bolsas de basura. Terminantemente prohibido depositar escombros de construcción o residuos bioinfecciosos.',
    ),
    const WastePoint(
      id: 'waste-4',
      name: 'Botadero Municipal Controlado - Plaza Quiulacocha',
      description: 'Punto de disposición de residuos domésticos para vecinos del distrito Simón Bolívar antes del paso de la unidad compactadora.',
      address: 'Plaza Principal de Quiulacocha s/n, Simón Bolívar',
      location: LatLng(-10.7020, -76.2870),
      type: WasteType.general,
      truckSchedule: 'Lunes y Jueves de 14:00 a 16:00',
      imageUrl: 'https://images.unsplash.com/photo-1595278069441-2cf29f8005a4?q=80&w=800&auto=format&fit=crop',
      recommendations: 'Sacar la basura máximo 30 minutos antes del horario del camión para evitar la dispersión por canes o vientos.',
    ),
    const WastePoint(
      id: 'waste-5',
      name: 'Estación Ecológica y Pilas - Av. Bolívar Central',
      description: 'Módulo de acopio integral de residuos reciclables secos y contenedor tubular de seguridad para recolección de pilas y baterías usadas.',
      address: 'Avenida Bolívar Central 105, Chaupimarca',
      location: LatLng(-10.6705, -76.2600),
      type: WasteType.recyclable,
      truckSchedule: 'Atención permanente (Lunes a Sábado de 08:00 a 18:00)',
      imageUrl: 'https://images.unsplash.com/photo-1604187351574-c75ca79f5807?q=80&w=800&auto=format&fit=crop',
      recommendations: 'Las pilas y baterías alcalinas deben colocarse en el tubo transparente de acopio para prevenir lixiviados tóxicos.',
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

  String _formatDistance(double meters) {
    if (meters < 1000) {
      return '${meters.round()} m';
    } else {
      return '${(meters / 1000).toStringAsFixed(1)} km';
    }
  }

  void _showWasteDetailDialog(BuildContext context, WastePoint point) {
    final appState = Provider.of<AppState>(context, listen: false);
    
    String wasteText;
    Color wasteColor;
    IconData wasteIcon;
    switch (point.type) {
      case WasteType.recyclable:
        wasteText = 'Reciclables ♻️ (Plástico, Papel, Cartón, Vidrio, Pilas)';
        wasteColor = const Color(0xFF1B5E20);
        wasteIcon = Icons.recycling_rounded;
        break;
      case WasteType.organic:
        wasteText = 'Orgánicos 🍂 (Cáscaras, Restos Vegetales, Compost)';
        wasteColor = const Color(0xFFE65100);
        wasteIcon = Icons.eco_rounded;
        break;
      case WasteType.general:
        wasteText = 'Residuos Generales 🗑️ (Desechos Domésticos No Peligrosos)';
        wasteColor = const Color(0xFF37474F);
        wasteIcon = Icons.delete_outline_rounded;
        break;
    }

    String? distanceStr;
    if (appState.userLocation != null) {
      final meters = const Distance()(appState.userLocation!, point.location);
      distanceStr = _formatDistance(meters);
    }

    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) {
        return Container(
          constraints: BoxConstraints(
            maxHeight: MediaQuery.of(context).size.height * 0.88,
          ),
          decoration: const BoxDecoration(
            color: Colors.white,
            borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
          ),
          child: SingleChildScrollView(
            padding: const EdgeInsets.fromLTRB(24, 16, 24, 24),
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
                
                // Foto referencial del punto de acopio / botadero
                if (point.imageUrl.isNotEmpty) ...[
                  ClipRRect(
                    borderRadius: BorderRadius.circular(16),
                    child: Container(
                      height: 190,
                      width: double.infinity,
                      decoration: const BoxDecoration(
                        color: Color(0xFFF1F5F9),
                      ),
                      child: Stack(
                        fit: StackFit.expand,
                        children: [
                          Image.network(
                            point.imageUrl,
                            fit: BoxFit.cover,
                            loadingBuilder: (context, child, progress) {
                              if (progress == null) return child;
                              return const Center(child: CircularProgressIndicator(strokeWidth: 2));
                            },
                            errorBuilder: (context, error, stackTrace) => Container(
                              color: const Color(0xFFE2E8F0),
                              child: const Column(
                                mainAxisAlignment: MainAxisAlignment.center,
                                children: [
                                  Icon(Icons.image_not_supported_outlined, color: Color(0xFF94A3B8), size: 36),
                                  SizedBox(height: 6),
                                  Text(
                                    'Foto referencial no disponible',
                                    style: TextStyle(color: Color(0xFF64748B), fontSize: 12),
                                  ),
                                ],
                              ),
                            ),
                          ),
                          Positioned(
                            top: 10,
                            right: 10,
                            child: Container(
                              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                              decoration: BoxDecoration(
                                color: Colors.black.withValues(alpha: 0.65),
                                borderRadius: BorderRadius.circular(20),
                              ),
                              child: Row(
                                mainAxisSize: MainAxisSize.min,
                                children: [
                                  const Icon(Icons.photo_camera_rounded, color: Colors.white, size: 12),
                                  const SizedBox(width: 4),
                                  Text(
                                    'Foto Referencial',
                                    style: GoogleFonts.outfit(
                                      color: Colors.white,
                                      fontSize: 11,
                                      fontWeight: FontWeight.w600,
                                    ),
                                  ),
                                ],
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(height: 16),
                ],

                // Título y distancia
                Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Icon(wasteIcon, color: wasteColor, size: 28),
                    const SizedBox(width: 10),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            point.name,
                            style: GoogleFonts.outfit(
                              fontSize: 19,
                              fontWeight: FontWeight.bold,
                              color: const Color(0xFF0F172A),
                            ),
                          ),
                          if (point.address.isNotEmpty) ...[
                            const SizedBox(height: 4),
                            Row(
                              children: [
                                const Icon(Icons.location_on_rounded, size: 14, color: Color(0xFF64748B)),
                                const SizedBox(width: 4),
                                Expanded(
                                  child: Text(
                                    point.address,
                                    style: GoogleFonts.outfit(
                                      fontSize: 13,
                                      fontWeight: FontWeight.w500,
                                      color: const Color(0xFF475569),
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          ],
                        ],
                      ),
                    ),
                    if (distanceStr != null) ...[
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                        decoration: BoxDecoration(
                          color: const Color(0xFFE0F2FE),
                          borderRadius: BorderRadius.circular(12),
                          border: Border.all(color: const Color(0xFFBAE6FD)),
                        ),
                        child: Column(
                          children: [
                            const Icon(Icons.navigation_rounded, color: Color(0xFF0284C7), size: 14),
                            Text(
                              distanceStr,
                              style: GoogleFonts.outfit(
                                fontSize: 11,
                                fontWeight: FontWeight.bold,
                                color: const Color(0xFF0284C7),
                              ),
                            ),
                          ],
                        ),
                      ),
                    ],
                  ],
                ),
                const SizedBox(height: 12),

                // Descripción
                Text(
                  point.description,
                  style: GoogleFonts.outfit(
                    fontSize: 13.5,
                    color: const Color(0xFF64748B),
                    height: 1.4,
                  ),
                ),
                const SizedBox(height: 16),
                const Divider(color: Color(0xFFE2E8F0)),
                const SizedBox(height: 12),

                // Tipo de residuo
                Text(
                  'CLASIFICACIÓN DE RESIDUOS ACEPTADOS',
                  style: GoogleFonts.outfit(
                    fontSize: 11,
                    fontWeight: FontWeight.w800,
                    color: const Color(0xFF94A3B8),
                    letterSpacing: 0.5,
                  ),
                ),
                const SizedBox(height: 6),
                Container(
                  width: double.infinity,
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 9),
                  decoration: BoxDecoration(
                    color: wasteColor.withValues(alpha: 0.08),
                    borderRadius: BorderRadius.circular(10),
                    border: Border.all(color: wasteColor.withValues(alpha: 0.3)),
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
                const SizedBox(height: 14),

                // Horario del camión
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: const Color(0xFFF8FAFC),
                    borderRadius: BorderRadius.circular(12),
                    border: Border.all(color: const Color(0xFFE2E8F0)),
                  ),
                  child: Row(
                    children: [
                      const Icon(Icons.local_shipping_rounded, color: Color(0xFF2563EB), size: 24),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'Paso del Camión Recolector Municipal:',
                              style: GoogleFonts.outfit(
                                fontSize: 11.5,
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
                const SizedBox(height: 14),

                // Recomendaciones de uso ciudadano
                if (point.recommendations.isNotEmpty) ...[
                  Container(
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: const Color(0xFFFFFBEB),
                      borderRadius: BorderRadius.circular(12),
                      border: Border.all(color: const Color(0xFFFDE68A)),
                    ),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Icon(Icons.tips_and_updates_rounded, color: Color(0xFFD97706), size: 20),
                        const SizedBox(width: 10),
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                'Recomendación Municipal de Disposición:',
                                style: GoogleFonts.outfit(
                                  fontSize: 11.5,
                                  color: const Color(0xFF92400E),
                                  fontWeight: FontWeight.bold,
                                ),
                              ),
                              const SizedBox(height: 2),
                              Text(
                                point.recommendations,
                                style: GoogleFonts.outfit(
                                  fontSize: 12.5,
                                  color: const Color(0xFF78350F),
                                  height: 1.35,
                                ),
                              ),
                            ],
                          ),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 20),
                ],

                // Botones de acción
                Row(
                  children: [
                    Expanded(
                      child: ElevatedButton.icon(
                        style: ElevatedButton.styleFrom(
                          backgroundColor: const Color(0xFF0284C7),
                          padding: const EdgeInsets.symmetric(vertical: 13),
                          shape: RoundedRectangleBorder(
                            borderRadius: BorderRadius.circular(12),
                          ),
                        ),
                        icon: const Icon(Icons.gps_fixed_rounded, color: Colors.white, size: 16),
                        label: Text(
                          'Centrar en el Mapa',
                          style: GoogleFonts.outfit(
                            color: Colors.white,
                            fontSize: 13.5,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                        onPressed: () {
                          Navigator.pop(context);
                          _mapController.move(point.location, 16.5);
                        },
                      ),
                    ),
                    const SizedBox(width: 10),
                    OutlinedButton(
                      style: OutlinedButton.styleFrom(
                        padding: const EdgeInsets.symmetric(vertical: 13, horizontal: 18),
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(12),
                        ),
                        side: const BorderSide(color: Color(0xFFCBD5E1)),
                      ),
                      onPressed: () => Navigator.pop(context),
                      child: Text(
                        'Cerrar',
                        style: GoogleFonts.outfit(
                          color: const Color(0xFF475569),
                          fontSize: 13.5,
                          fontWeight: FontWeight.bold,
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
    );
  }

  void _showNearestWastePointsModal(BuildContext context) {
    final appState = Provider.of<AppState>(context, listen: false);
    final points = appState.wastePoints.isNotEmpty ? appState.wastePoints : _wastePoints;

    List<WastePoint> sortedPoints = List.from(points);
    if (appState.userLocation != null) {
      sortedPoints.sort((a, b) {
        final distA = const Distance()(appState.userLocation!, a.location);
        final distB = const Distance()(appState.userLocation!, b.location);
        return distA.compareTo(distB);
      });
    }

    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) {
        return Container(
          constraints: BoxConstraints(
            maxHeight: MediaQuery.of(context).size.height * 0.85,
          ),
          decoration: const BoxDecoration(
            color: Colors.white,
            borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
          ),
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 20),
          child: Column(
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
                  Container(
                    padding: const EdgeInsets.all(8),
                    decoration: BoxDecoration(
                      color: const Color(0xFFE8F5E9),
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: const Icon(Icons.delete_outline_rounded, color: Color(0xFF2E7D32), size: 22),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Botaderos y Puntos de Acopio',
                          style: GoogleFonts.outfit(
                            fontSize: 18,
                            fontWeight: FontWeight.bold,
                            color: const Color(0xFF0F172A),
                          ),
                        ),
                        Text(
                          appState.userLocation != null
                              ? 'Ordenados por cercanía a tu ubicación GPS'
                              : 'Puntos oficiales autorizados en Cerro de Pasco',
                          style: GoogleFonts.outfit(
                            fontSize: 12,
                            color: const Color(0xFF64748B),
                          ),
                        ),
                      ],
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.close_rounded, color: Color(0xFF64748B)),
                    onPressed: () => Navigator.pop(context),
                  ),
                ],
              ),
              const SizedBox(height: 14),
              const Divider(color: Color(0xFFE2E8F0)),
              const SizedBox(height: 8),

              Expanded(
                child: ListView.separated(
                  itemCount: sortedPoints.length,
                  separatorBuilder: (_, _) => const SizedBox(height: 12),
                  itemBuilder: (context, index) {
                    final p = sortedPoints[index];
                    String? dist;
                    if (appState.userLocation != null) {
                      dist = _formatDistance(const Distance()(appState.userLocation!, p.location));
                    }

                    Color badgeColor;
                    String badgeLabel;
                    switch (p.type) {
                      case WasteType.recyclable:
                        badgeColor = const Color(0xFF1B5E20);
                        badgeLabel = 'Reciclable';
                        break;
                      case WasteType.organic:
                        badgeColor = const Color(0xFFE65100);
                        badgeLabel = 'Orgánico';
                        break;
                      case WasteType.general:
                        badgeColor = const Color(0xFF37474F);
                        badgeLabel = 'General';
                        break;
                    }

                    return InkWell(
                      borderRadius: BorderRadius.circular(14),
                      onTap: () {
                        Navigator.pop(context);
                        setState(() {
                          _showWasteLayer = true;
                        });
                        _mapController.move(p.location, 16.5);
                        _showWasteDetailDialog(context, p);
                      },
                      child: Container(
                        padding: const EdgeInsets.all(12),
                        decoration: BoxDecoration(
                          color: const Color(0xFFF8FAFC),
                          borderRadius: BorderRadius.circular(14),
                          border: Border.all(color: const Color(0xFFE2E8F0)),
                        ),
                        child: Row(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            // Foto referencial miniatura
                            ClipRRect(
                              borderRadius: BorderRadius.circular(10),
                              child: Container(
                                width: 75,
                                height: 75,
                                color: const Color(0xFFE2E8F0),
                                child: p.imageUrl.isNotEmpty
                                    ? Image.network(
                                        p.imageUrl,
                                        fit: BoxFit.cover,
                                        errorBuilder: (_, _, _) => const Icon(
                                          Icons.delete_outline_rounded,
                                          color: Color(0xFF94A3B8),
                                        ),
                                      )
                                    : const Icon(
                                        Icons.delete_outline_rounded,
                                        color: Color(0xFF94A3B8),
                                      ),
                              ),
                            ),
                            const SizedBox(width: 12),

                            Expanded(
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Row(
                                    children: [
                                      Container(
                                        padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 2),
                                        decoration: BoxDecoration(
                                          color: badgeColor.withValues(alpha: 0.12),
                                          borderRadius: BorderRadius.circular(6),
                                        ),
                                        child: Text(
                                          badgeLabel,
                                          style: GoogleFonts.outfit(
                                            color: badgeColor,
                                            fontSize: 10.5,
                                            fontWeight: FontWeight.bold,
                                          ),
                                        ),
                                      ),
                                      const Spacer(),
                                      if (dist != null)
                                        Container(
                                          padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 2),
                                          decoration: BoxDecoration(
                                            color: const Color(0xFFE0F2FE),
                                            borderRadius: BorderRadius.circular(6),
                                          ),
                                          child: Text(
                                            '📍 $dist',
                                            style: GoogleFonts.outfit(
                                              color: const Color(0xFF0284C7),
                                              fontSize: 11,
                                              fontWeight: FontWeight.bold,
                                            ),
                                          ),
                                        ),
                                    ],
                                  ),
                                  const SizedBox(height: 4),
                                  Text(
                                    p.name,
                                    style: GoogleFonts.outfit(
                                      fontSize: 14,
                                      fontWeight: FontWeight.bold,
                                      color: const Color(0xFF0F172A),
                                    ),
                                    maxLines: 1,
                                    overflow: TextOverflow.ellipsis,
                                  ),
                                  if (p.address.isNotEmpty) ...[
                                    const SizedBox(height: 2),
                                    Text(
                                      p.address,
                                      style: GoogleFonts.outfit(
                                        fontSize: 12,
                                        color: const Color(0xFF64748B),
                                      ),
                                      maxLines: 1,
                                      overflow: TextOverflow.ellipsis,
                                    ),
                                  ],
                                  const SizedBox(height: 4),
                                  Text(
                                    '🚚 ${p.truckSchedule}',
                                    style: GoogleFonts.outfit(
                                      fontSize: 11,
                                      color: const Color(0xFF0284C7),
                                      fontWeight: FontWeight.w600,
                                    ),
                                    maxLines: 1,
                                    overflow: TextOverflow.ellipsis,
                                  ),
                                ],
                              ),
                            ),
                          ],
                        ),
                      ),
                    );
                  },
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
        : 'https://tile.openstreetmap.org/{z}/{x}/{y}.png';

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
                  color: const Color(0xFF0288D1).withValues(alpha: 0.25),
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
                      color: Colors.black.withValues(alpha: 0.2),
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
      final pointsToUse = appState.wastePoints.isNotEmpty ? appState.wastePoints : _wastePoints;
      for (final wp in pointsToUse) {
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
                      color: itemColor.withValues(alpha: 0.2),
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
                          color: Colors.black.withValues(alpha: 0.15),
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
                          color: const Color(0xFF2E7D32).withValues(alpha: 0.55),
                          strokeWidth: 4.5,
                        ),
                      ],
                    ),

                  // Capa de Mapa de Calor (Densidad de Contaminación Ambiental)
                  if (appState.showHeatmap)
                    CircleLayer(
                      circles: appState.filteredAlerts.map((alert) {
                        Color circleColor;
                        double radius;
                        switch (alert.severity) {
                          case EcoSeverity.critico:
                            circleColor = const Color(0xFFEF4444).withValues(alpha: 0.38);
                            radius = 160.0;
                            break;
                          case EcoSeverity.medio:
                            circleColor = const Color(0xFFF59E0B).withValues(alpha: 0.32);
                            radius = 100.0;
                            break;
                          case EcoSeverity.bajo:
                            circleColor = const Color(0xFF10B981).withValues(alpha: 0.28);
                            radius = 60.0;
                            break;
                        }
                        return CircleMarker(
                          point: LatLng(alert.latitude, alert.longitude),
                          color: circleColor,
                          borderColor: circleColor.withValues(alpha: 0.75),
                          borderStrokeWidth: 1.5,
                          useRadiusInMeter: true,
                          radius: radius,
                        );
                      }).toList(),
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
                            color: Colors.black.withValues(alpha: 0.12),
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
                                color: Colors.black.withValues(alpha: 0.25),
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
                                color: const Color(0xFF0288D1).withValues(alpha: 0.08),
                                border: Border.all(
                                  color: const Color(0xFF0288D1).withValues(alpha: 0.35),
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
                                color: Colors.white.withValues(alpha: 0.85),
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
                            color: Colors.black.withValues(alpha: 0.08),
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
                            color: Colors.black.withValues(alpha: 0.08),
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
                            color: Colors.black.withValues(alpha: 0.08),
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
                              color: Colors.black.withValues(alpha: 0.08),
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
                    if (!appState.isReportMode) ...[
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
                              color: Colors.black.withValues(alpha: 0.08),
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
                            '♻️ Botaderos y Acopio',
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
                      Container(
                        margin: const EdgeInsets.only(right: 8),
                        decoration: BoxDecoration(
                          color: Colors.white,
                          borderRadius: BorderRadius.circular(12),
                          border: Border.all(
                            color: const Color(0xFF0284C7),
                            width: 1.5,
                          ),
                          boxShadow: [
                            BoxShadow(
                              color: Colors.black.withValues(alpha: 0.08),
                              blurRadius: 8,
                              offset: const Offset(0, 2),
                            )
                          ],
                        ),
                        child: TextButton.icon(
                          icon: const Icon(
                            Icons.near_me_rounded,
                            color: Color(0xFF0284C7),
                            size: 16,
                          ),
                          label: Text(
                            '📍 Más Cercanos',
                            style: GoogleFonts.outfit(
                              color: const Color(0xFF0284C7),
                              fontSize: 12,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          style: TextButton.styleFrom(
                            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                          ),
                          onPressed: () {
                            _showNearestWastePointsModal(context);
                          },
                        ),
                      ),
                    ],

                    // Botón de Mapa de Calor (Heatmap)
                    if (!appState.isReportMode)
                      Container(
                        margin: const EdgeInsets.only(right: 8),
                        decoration: BoxDecoration(
                          color: appState.showHeatmap ? const Color(0xFFFEF2F2) : Colors.white,
                          borderRadius: BorderRadius.circular(12),
                          border: Border.all(
                            color: appState.showHeatmap ? const Color(0xFFEF4444) : const Color(0xFFE2E8F0),
                            width: 1.5,
                          ),
                          boxShadow: [
                            BoxShadow(
                              color: Colors.black.withValues(alpha: 0.08),
                              blurRadius: 8,
                              offset: const Offset(0, 2),
                            )
                          ],
                        ),
                        child: TextButton.icon(
                          icon: Icon(
                            Icons.local_fire_department_rounded,
                            color: appState.showHeatmap ? const Color(0xFFEF4444) : const Color(0xFF64748B),
                            size: 16,
                          ),
                          label: Text(
                            '🔥 Mapa de Calor',
                            style: GoogleFonts.outfit(
                              color: appState.showHeatmap ? const Color(0xFFDC2626) : const Color(0xFF475569),
                              fontSize: 12,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          style: TextButton.styleFrom(
                            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                          ),
                          onPressed: () {
                            appState.showHeatmap = !appState.showHeatmap;
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
                            color: Colors.black.withValues(alpha: 0.08),
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
                            color: Colors.black.withValues(alpha: 0.08),
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
                          color: Colors.black.withValues(alpha: 0.08),
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
                            color: Colors.black.withValues(alpha: 0.12),
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
                    : ListView.builder(
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
                                    color: Colors.black.withValues(alpha: 0.04),
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
                                          color: sevColor.withValues(alpha: 0.12),
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
                color: color.withValues(alpha: 0.3),
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
                  color: Colors.black.withValues(alpha: 0.18),
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
                                  color: color.withValues(alpha: 0.12),
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

  void _showAIDiagnosisDialog(BuildContext context) {
    final appState = Provider.of<AppState>(context, listen: false);
    if (!appState.isLoggedIn) {
      showDialog(
        context: context,
        builder: (context) => const AuthDialog(),
      );
      return;
    }

    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) {
        return FutureBuilder<Map<String, dynamic>>(
          future: appState.analyzeEvidenceWithGemini(imageUrl: alert.imageUrl),
          builder: (context, snapshot) {
            if (snapshot.connectionState == ConnectionState.waiting) {
              return AlertDialog(
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
                content: Padding(
                  padding: const EdgeInsets.all(20.0),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      const CircularProgressIndicator(color: Color(0xFF0288D1)),
                      const SizedBox(height: 16),
                      Text(
                        'Gemini AI está analizando la evidencia...',
                        style: GoogleFonts.outfit(fontWeight: FontWeight.bold),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        'Evaluando impacto ambiental para Cerro de Pasco',
                        style: GoogleFonts.outfit(fontSize: 12, color: const Color(0xFF64748B)),
                      ),
                    ],
                  ),
                ),
              );
            }

            if (snapshot.hasError) {
              return AlertDialog(
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
                title: Text('Error de Análisis', style: GoogleFonts.outfit(fontWeight: FontWeight.bold)),
                content: Text(
                  snapshot.error.toString().replaceAll('Exception: ', ''),
                  style: GoogleFonts.outfit(fontSize: 13),
                ),
                actions: [
                  TextButton(
                    onPressed: () => Navigator.pop(context),
                    child: Text('Cerrar', style: GoogleFonts.outfit(fontWeight: FontWeight.bold)),
                  ),
                ],
              );
            }

            final data = snapshot.data ?? {};
            return AlertDialog(
              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(18)),
              title: Row(
                children: [
                  const Icon(Icons.auto_awesome_rounded, color: Color(0xFF0288D1)),
                  const SizedBox(width: 8),
                  Text(
                    'Peritaje Técnico Gemini AI',
                    style: GoogleFonts.outfit(fontWeight: FontWeight.bold, fontSize: 16),
                  ),
                ],
              ),
              content: SizedBox(
                width: 420,
                child: SingleChildScrollView(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      if (data['technical_analysis'] != null) ...[
                        Text(
                          'DIAGNÓSTICO VISUAL:',
                          style: GoogleFonts.outfit(fontSize: 10, fontWeight: FontWeight.bold, color: const Color(0xFF64748B)),
                        ),
                        const SizedBox(height: 4),
                        Text(
                          data['technical_analysis'].toString(),
                          style: GoogleFonts.outfit(fontSize: 13, color: const Color(0xFF1E293B)),
                        ),
                        const SizedBox(height: 12),
                      ],
                      if (data['municipal_action'] != null) ...[
                        Text(
                          'ACCIÓN MUNICIPAL RECOMENDADA:',
                          style: GoogleFonts.outfit(fontSize: 10, fontWeight: FontWeight.bold, color: const Color(0xFF64748B)),
                        ),
                        const SizedBox(height: 4),
                        Text(
                          data['municipal_action'].toString(),
                          style: GoogleFonts.outfit(fontSize: 13, color: const Color(0xFF065F46), fontWeight: FontWeight.w600),
                        ),
                      ],
                    ],
                  ),
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: Text('Entendido', style: GoogleFonts.outfit(fontWeight: FontWeight.bold, color: const Color(0xFF0288D1))),
                ),
              ],
            );
          },
        );
      },
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
            color: Colors.black.withValues(alpha: 0.1),
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
                      color: sevColor.withValues(alpha: 0.12),
                      borderRadius: BorderRadius.circular(8),
                      border: Border.all(color: sevColor.withValues(alpha: 0.3), width: 1),
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
                      color: statusColor.withValues(alpha: 0.12),
                      borderRadius: BorderRadius.circular(8),
                      border: Border.all(color: statusColor.withValues(alpha: 0.3), width: 1),
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
                  if (alert.imageUrl != null) ...[
                    IconButton(
                      constraints: const BoxConstraints(),
                      padding: EdgeInsets.zero,
                      icon: const Icon(Icons.auto_awesome_rounded, color: Color(0xFF0288D1), size: 18),
                      tooltip: 'Diagnóstico IA Gemini',
                      onPressed: () => _showAIDiagnosisDialog(context),
                    ),
                    const SizedBox(width: 8),
                  ],
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
          
          // Evidencias Fotográficas Reales
          if (alert.evidenceImages.isNotEmpty) ...[
            Container(
              height: alert.evidenceImages.length > 1 ? 130 : 160,
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: const Color(0xFFE2E8F0)),
              ),
              child: ClipRRect(
                borderRadius: BorderRadius.circular(8),
                child: alert.evidenceImages.length == 1
                    ? GestureDetector(
                        onTap: () {
                          showDialog(
                            context: context,
                            builder: (context) => _ImageLightboxDialog(
                              imageName: alert.evidenceImages.first,
                              severityColor: sevColor,
                              categoryIcon: catIcon,
                            ),
                          );
                        },
                        child: Stack(
                          fit: StackFit.expand,
                          children: [
                            Image.network(
                              alert.evidenceImages.first,
                              fit: BoxFit.cover,
                              loadingBuilder: (context, child, progress) {
                                if (progress == null) return child;
                                return Center(
                                  child: SizedBox(
                                    width: 24,
                                    height: 24,
                                    child: CircularProgressIndicator(
                                      strokeWidth: 2,
                                      value: progress.expectedTotalBytes != null
                                          ? progress.cumulativeBytesLoaded / progress.expectedTotalBytes!
                                          : null,
                                      color: const Color(0xFF0288D1),
                                    ),
                                  ),
                                );
                              },
                              errorBuilder: (context, error, stackTrace) => Container(
                                color: const Color(0xFFF1F5F9),
                                child: Center(
                                  child: Column(
                                    mainAxisSize: MainAxisSize.min,
                                    children: [
                                      const Icon(Icons.broken_image_rounded, size: 28, color: Color(0xFF94A3B8)),
                                      const SizedBox(height: 4),
                                      Text(
                                        'No se pudo cargar la imagen',
                                        style: GoogleFonts.outfit(fontSize: 11, color: const Color(0xFF64748B)),
                                      ),
                                    ],
                                  ),
                                ),
                              ),
                            ),
                            Positioned(
                              top: 8,
                              right: 8,
                              child: Container(
                                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                                decoration: BoxDecoration(
                                  color: Colors.black.withValues(alpha: 0.65),
                                  borderRadius: BorderRadius.circular(4),
                                ),
                                child: Row(
                                  mainAxisSize: MainAxisSize.min,
                                  children: [
                                    const Icon(Icons.zoom_in, color: Colors.white, size: 14),
                                    const SizedBox(width: 4),
                                    Text(
                                      'Ampliar',
                                      style: GoogleFonts.outfit(color: Colors.white, fontSize: 10, fontWeight: FontWeight.bold),
                                    ),
                                  ],
                                ),
                              ),
                            ),
                          ],
                        ),
                      )
                    : Row(
                        children: alert.evidenceImages.asMap().entries.map((entry) {
                          final idx = entry.key;
                          final imgUrl = entry.value;
                          return Expanded(
                            child: Padding(
                              padding: EdgeInsets.only(left: idx == 0 ? 0 : 4),
                              child: GestureDetector(
                                onTap: () {
                                  showDialog(
                                    context: context,
                                    builder: (context) => _ImageLightboxDialog(
                                      imageName: imgUrl,
                                      severityColor: sevColor,
                                      categoryIcon: catIcon,
                                    ),
                                  );
                                },
                                child: Stack(
                                  fit: StackFit.expand,
                                  children: [
                                    Image.network(
                                      imgUrl,
                                      fit: BoxFit.cover,
                                      loadingBuilder: (context, child, progress) {
                                        if (progress == null) return child;
                                        return const Center(
                                          child: SizedBox(
                                            width: 20,
                                            height: 20,
                                            child: CircularProgressIndicator(strokeWidth: 2, color: Color(0xFF0288D1)),
                                          ),
                                        );
                                      },
                                      errorBuilder: (context, error, stackTrace) => Container(
                                        color: const Color(0xFFF1F5F9),
                                        child: const Center(
                                          child: Icon(Icons.broken_image_rounded, size: 24, color: Color(0xFF94A3B8)),
                                        ),
                                      ),
                                    ),
                                    Positioned(
                                      bottom: 6,
                                      left: 6,
                                      child: Container(
                                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                                        decoration: BoxDecoration(
                                          color: Colors.black.withValues(alpha: 0.65),
                                          borderRadius: BorderRadius.circular(3),
                                        ),
                                        child: Text(
                                          'Foto ${idx + 1}',
                                          style: GoogleFonts.outfit(color: Colors.white, fontSize: 9.5, fontWeight: FontWeight.bold),
                                        ),
                                      ),
                                    ),
                                    Positioned(
                                      top: 6,
                                      right: 6,
                                      child: Container(
                                        padding: const EdgeInsets.all(3),
                                        decoration: BoxDecoration(
                                          color: Colors.black.withValues(alpha: 0.55),
                                          shape: BoxShape.circle,
                                        ),
                                        child: const Icon(Icons.zoom_in, color: Colors.white, size: 12),
                                      ),
                                    ),
                                  ],
                                ),
                              ),
                            ),
                          );
                        }).toList(),
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
          const SizedBox(height: 10),

          // Trazabilidad del Ciudadano
          Row(
            children: [
              const Icon(Icons.account_circle_outlined, color: Color(0xFF0288D1), size: 14),
              const SizedBox(width: 5),
              Text(
                'Reportado por: ',
                style: GoogleFonts.outfit(
                  color: const Color(0xFF64748B),
                  fontSize: 11,
                  fontWeight: FontWeight.w500,
                ),
              ),
              Text(
                alert.reporterName,
                style: GoogleFonts.outfit(
                  color: const Color(0xFF0F172A),
                  fontSize: 11,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ],
          ),
          
          // Dirección
          const SizedBox(height: 10),
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

          // Validación Ciudadana (+1 Confirmación Vecinal)
          const SizedBox(height: 10),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
            decoration: BoxDecoration(
              color: alert.hasUserConfirmed ? const Color(0xFFEFF6FF) : const Color(0xFFF8FAFC),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(
                color: alert.hasUserConfirmed ? const Color(0xFF93C5FD) : const Color(0xFFE2E8F0),
              ),
            ),
            child: Row(
              children: [
                Icon(
                  Icons.people_alt_rounded,
                  color: alert.hasUserConfirmed ? const Color(0xFF2563EB) : const Color(0xFF64748B),
                  size: 16,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    alert.confirmationsCount == 0
                        ? 'Sé el primero en validar este caso'
                        : '${alert.confirmationsCount} vecinos confirman este problema',
                    style: GoogleFonts.outfit(
                      fontSize: 11,
                      fontWeight: FontWeight.w600,
                      color: alert.hasUserConfirmed ? const Color(0xFF1D4ED8) : const Color(0xFF475569),
                    ),
                  ),
                ),
                InkWell(
                  onTap: () async {
                    final appState = Provider.of<AppState>(context, listen: false);
                    if (!appState.isLoggedIn) {
                      showDialog(
                        context: context,
                        builder: (context) => const AuthDialog(),
                      );
                      return;
                    }
                    final ok = await appState.confirmAlert(alert.id);
                    if (!ok && context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(
                          content: Text('No se pudo actualizar la confirmación.', style: GoogleFonts.outfit()),
                          backgroundColor: const Color(0xFFEF4444),
                        ),
                      );
                    }
                  },
                  borderRadius: BorderRadius.circular(8),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: alert.hasUserConfirmed ? const Color(0xFF2563EB) : const Color(0xFF0288D1).withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(6),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(
                          alert.hasUserConfirmed ? Icons.check_circle_rounded : Icons.thumb_up_alt_outlined,
                          size: 13,
                          color: alert.hasUserConfirmed ? Colors.white : const Color(0xFF0288D1),
                        ),
                        const SizedBox(width: 4),
                        Text(
                          alert.hasUserConfirmed ? 'Confirmado' : '+1 Validar',
                          style: GoogleFonts.outfit(
                            fontSize: 10.5,
                            fontWeight: FontWeight.bold,
                            color: alert.hasUserConfirmed ? Colors.white : const Color(0xFF0288D1),
                          ),
                        ),
                      ],
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
              color: Colors.black.withValues(alpha: 0.85),
            ),
          ),
          Container(
            constraints: const BoxConstraints(maxWidth: 820, maxHeight: 620),
            margin: const EdgeInsets.symmetric(horizontal: 16),
            decoration: BoxDecoration(
              color: const Color(0xFF0F172A),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: severityColor.withValues(alpha: 0.6), width: 1.5),
              boxShadow: const [
                BoxShadow(
                  color: Colors.black54,
                  blurRadius: 24,
                  offset: Offset(0, 8),
                ),
              ],
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(10),
              child: Stack(
                fit: StackFit.expand,
                children: [
                  Center(
                    child: InteractiveViewer(
                      panEnabled: true,
                      minScale: 0.8,
                      maxScale: 3.5,
                      child: Image.network(
                        imageName,
                        fit: BoxFit.contain,
                        loadingBuilder: (context, child, progress) {
                          if (progress == null) return child;
                          return Center(
                            child: Column(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                const CircularProgressIndicator(color: Colors.white, strokeWidth: 2),
                                const SizedBox(height: 12),
                                Text(
                                  'Cargando fotografía en alta resolución...',
                                  style: GoogleFonts.outfit(color: Colors.white70, fontSize: 12),
                                ),
                              ],
                            ),
                          );
                        },
                        errorBuilder: (context, error, stackTrace) => Center(
                          child: Column(
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              const Icon(Icons.broken_image_rounded, color: Colors.white60, size: 54),
                              const SizedBox(height: 12),
                              Text(
                                'No se pudo cargar la imagen',
                                style: GoogleFonts.outfit(color: Colors.white, fontSize: 14, fontWeight: FontWeight.bold),
                              ),
                              const SizedBox(height: 4),
                              Padding(
                                padding: const EdgeInsets.symmetric(horizontal: 32.0),
                                child: Text(
                                  imageName,
                                  textAlign: TextAlign.center,
                                  style: GoogleFonts.outfit(color: Colors.white38, fontSize: 11),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                    ),
                  ),
                  Positioned(
                    top: 12,
                    right: 12,
                    child: Container(
                      decoration: BoxDecoration(
                        color: Colors.black.withValues(alpha: 0.65),
                        shape: BoxShape.circle,
                      ),
                      child: IconButton(
                        icon: const Icon(Icons.close_rounded, color: Colors.white, size: 24),
                        onPressed: () => Navigator.of(context).pop(),
                        tooltip: 'Cerrar visor',
                      ),
                    ),
                  ),
                  Positioned(
                    bottom: 12,
                    left: 16,
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                      decoration: BoxDecoration(
                        color: Colors.black.withValues(alpha: 0.7),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Row(
                        children: [
                          Icon(categoryIcon, color: severityColor, size: 14),
                          const SizedBox(width: 6),
                          Text(
                            'Evidencia Fotográfica de Campo',
                            style: GoogleFonts.outfit(color: Colors.white, fontSize: 11, fontWeight: FontWeight.w600),
                          ),
                        ],
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
