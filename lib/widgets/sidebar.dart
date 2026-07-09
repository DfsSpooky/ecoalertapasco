import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:provider/provider.dart';
import '../models/eco_alert.dart';
import '../state/app_state.dart';
import 'stat_charts.dart';
import 'auth_dialog.dart';

class LeftSidebar extends StatelessWidget {
  const LeftSidebar({super.key});

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);

    return Container(
      width: 320,
      decoration: const BoxDecoration(
        color: Colors.white,
        border: Border(
          right: BorderSide(
            color: Color(0xFFE2E8F0), // Slate 200
            width: 1.5,
          ),
        ),
      ),
      child: Column(
        children: [
          // Logo & Header
          Container(
            padding: const EdgeInsets.only(top: 24, left: 20, right: 20, bottom: 16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Row(
                      children: [
                        Text(
                          '🌱',
                          style: GoogleFonts.outfit(fontSize: 28),
                        ),
                        const SizedBox(width: 8),
                        Text(
                          'EcoAlerta',
                          style: GoogleFonts.outfit(
                            fontSize: 24,
                            fontWeight: FontWeight.bold,
                            color: const Color(0xFF0F172A), // Slate 900
                            letterSpacing: 0.5,
                          ),
                        ),
                      ],
                    ),
                    IconButton(
                      icon: const Icon(Icons.help_outline_rounded, color: Color(0xFF64748B), size: 22),
                      tooltip: 'Guía de Ayuda',
                      constraints: const BoxConstraints(),
                      padding: EdgeInsets.zero,
                      onPressed: () {
                        showDialog(
                          context: context,
                          builder: (context) => const HelpGuideDialog(),
                        );
                      },
                    )
                  ],
                ),
                const SizedBox(width: 4),
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Text(
                      'DASHBOARD AMBIENTAL',
                      style: GoogleFonts.outfit(
                        fontSize: 10,
                        fontWeight: FontWeight.w700,
                        color: const Color(0xFF0288D1), // Blue
                        letterSpacing: 1.5,
                      ),
                    ),
                    Consumer<AppState>(
                      builder: (context, appState, child) {
                        return Container(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                          decoration: BoxDecoration(
                            color: const Color(0xFFD1FAE5), // Green 100
                            borderRadius: BorderRadius.circular(10),
                            border: Border.all(color: const Color(0xFFA7F3D0), width: 1), // Green 200
                          ),
                          child: Text(
                            'Activos: ${appState.filteredAlerts.length}',
                            style: GoogleFonts.outfit(
                              fontSize: 10,
                              fontWeight: FontWeight.w600,
                              color: const Color(0xFF065F46), // Green 800
                            ),
                          ),
                        );
                      },
                    )
                  ],
                ),
              ],
            ),
          ),
          const Divider(color: Color(0xFFE2E8F0), height: 1),
          
          // Contenido con scroll
          Expanded(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(16.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Acceso de Usuarios / Autoridad Section
                  const _AuthPanelCard(),
                  const SizedBox(height: 20),

                  // Buscador Section
                  const _SearchField(),
                  const SizedBox(height: 20),

                  // KPI Highlights
                  const _KpiHighlights(),
                  const SizedBox(height: 20),

                  // Distrito Section
                  _buildSectionHeader('DISTRITO DE CERRO DE PASCO'),
                  const SizedBox(height: 12),
                  const _DistrictFilterRow(),
                  const SizedBox(height: 24),

                  // Filtro de Cercanía Section
                  const _ProximityFilterSection(),
                  const SizedBox(height: 24),

                  // Rango Temporal Section
                  _buildSectionHeader('RANGO TEMPORAL'),
                  const SizedBox(height: 12),
                  const _DaysFilterRow(),
                  const SizedBox(height: 24),

                  // Estado del Reporte Section
                  _buildSectionHeader('ESTADO DEL REPORTE'),
                  const SizedBox(height: 12),
                  const _StatusFilterRow(),
                  if (appState.isLoggedInAuthority) ...[
                    const SizedBox(height: 8),
                    const _DismissedFilterRow(),
                  ],
                  const SizedBox(height: 24),
                  
                  // Categoría Section
                  _buildSectionHeader('CATEGORÍA'),
                  const SizedBox(height: 12),
                  const _CategoryFilterGrid(),
                  const SizedBox(height: 24),
                  
                  // Nivel Section
                  _buildSectionHeader('NIVEL DE GRAVEDAD'),
                  const SizedBox(height: 12),
                  const _SeverityFilterRow(),
                  const SizedBox(height: 28),
                  
                  // Charts Section
                  const Divider(color: Color(0xFFE2E8F0), height: 1),
                  const SizedBox(height: 20),
                  
                  _buildSectionHeader('RESUMEN MENSUAL'),
                  const SizedBox(height: 8),
                  const MonthlyDonutChart(),
                  const SizedBox(height: 24),
                  
                  _buildSectionHeader('ALERTAS POR SEVERIDAD'),
                  const SizedBox(height: 16),
                  const SeverityBarChart(),
                  const SizedBox(height: 24),

                  // Simulador de Emergencias Section
                  const Divider(color: Color(0xFFE2E8F0), height: 1),
                  const SizedBox(height: 20),
                  const _EmergencySimulatorCard(),
                  const SizedBox(height: 20),

                  const SizedBox(height: 12),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
    return Text(
      title,
      style: GoogleFonts.outfit(
        fontSize: 11,
        fontWeight: FontWeight.w800,
        color: const Color(0xFF64748B), // Slate 500
        letterSpacing: 1.0,
      ),
    );
  }
}

class _CategoryFilterGrid extends StatelessWidget {
  const _CategoryFilterGrid();

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);

    // Mapeo de categorías con su estilo
    final categories = [
      _CategoryItem(
        category: EcoCategory.mineria,
        label: 'Minería',
        icon: Icons.terrain_rounded,
        color: const Color(0xFFE65100),
      ),
      _CategoryItem(
        category: EcoCategory.basura,
        label: 'Basura',
        icon: Icons.delete_outline_rounded,
        color: const Color(0xFF607D8B),
      ),
      _CategoryItem(
        category: EcoCategory.agua,
        label: 'Agua',
        icon: Icons.water_drop_rounded,
        color: const Color(0xFF0288D1),
      ),
      _CategoryItem(
        category: EcoCategory.aire,
        label: 'Aire',
        icon: Icons.air_rounded,
        color: const Color(0xFF00897B),
      ),
    ];

    return GridView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 2,
        crossAxisSpacing: 10,
        mainAxisSpacing: 10,
        childAspectRatio: 1.3,
      ),
      itemCount: categories.length,
      itemBuilder: (context, index) {
        final item = categories[index];
        final isSelected = appState.selectedCategories.contains(item.category);

        return GestureDetector(
          onTap: () => appState.toggleCategory(item.category),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 200),
            decoration: BoxDecoration(
              color: isSelected 
                  ? item.color.withOpacity(0.12) 
                  : const Color(0xFFF1F5F9), // Slate 100
              borderRadius: BorderRadius.circular(12),
              border: Border.all(
                color: isSelected ? item.color : const Color(0xFFE2E8F0),
                width: 1.5,
              ),
              boxShadow: isSelected
                  ? [
                      BoxShadow(
                        color: item.color.withOpacity(0.1),
                        blurRadius: 6,
                        offset: const Offset(0, 2),
                      )
                    ]
                  : [],
            ),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(
                  item.icon,
                  color: isSelected ? item.color : const Color(0xFF64748B),
                  size: 24,
                ),
                const SizedBox(height: 6),
                Text(
                  item.label,
                  style: GoogleFonts.outfit(
                    fontSize: 12,
                    fontWeight: isSelected ? FontWeight.bold : FontWeight.w500,
                    color: isSelected ? const Color(0xFF0F172A) : const Color(0xFF64748B),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}

class _CategoryItem {
  final EcoCategory category;
  final String label;
  final IconData icon;
  final Color color;

  const _CategoryItem({
    required this.category,
    required this.label,
    required this.icon,
    required this.color,
  });
}

class _SeverityFilterRow extends StatelessWidget {
  const _SeverityFilterRow();

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);

    final levels = [
      _SeverityItem(
        severity: EcoSeverity.critico,
        label: 'Crítico',
        emoji: '💀',
        color: const Color(0xFFD32F2F),
      ),
      _SeverityItem(
        severity: EcoSeverity.medio,
        label: 'Medio',
        emoji: '⚠️',
        color: const Color(0xFFFFA000),
      ),
      _SeverityItem(
        severity: EcoSeverity.bajo,
        label: 'Bajo',
        emoji: '🍃',
        color: const Color(0xFF388E3C),
      ),
    ];

    return Row(
      children: levels.map((item) {
        final isSelected = appState.selectedSeverities.contains(item.severity);

        return Expanded(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 4.0),
            child: GestureDetector(
              onTap: () => appState.toggleSeverity(item.severity),
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 200),
                padding: const EdgeInsets.symmetric(vertical: 8),
                decoration: BoxDecoration(
                  color: isSelected 
                      ? item.color.withOpacity(0.12) 
                      : const Color(0xFFF1F5F9), // Slate 100
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(
                    color: isSelected ? item.color : const Color(0xFFE2E8F0),
                    width: 1.5,
                  ),
                ),
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text(
                      item.emoji,
                      style: const TextStyle(fontSize: 16),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      item.label,
                      style: GoogleFonts.outfit(
                        fontSize: 11,
                        fontWeight: isSelected ? FontWeight.bold : FontWeight.w500,
                        color: isSelected ? const Color(0xFF0F172A) : const Color(0xFF64748B),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        );
      }).toList(),
    );
  }
}

class _SeverityItem {
  final EcoSeverity severity;
  final String label;
  final String emoji;
  final Color color;

  const _SeverityItem({
    required this.severity,
    required this.label,
    required this.emoji,
    required this.color,
  });
}

class _SearchField extends StatefulWidget {
  const _SearchField();

  @override
  State<_SearchField> createState() => _SearchFieldState();
}

class _SearchFieldState extends State<_SearchField> {
  late final TextEditingController _controller;

  @override
  void initState() {
    super.initState();
    final appState = Provider.of<AppState>(context, listen: false);
    _controller = TextEditingController(text: appState.searchQuery);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);
    
    if (_controller.text != appState.searchQuery) {
      _controller.text = appState.searchQuery;
    }

    return TextField(
      controller: _controller,
      onChanged: (val) => appState.setSearchQuery(val),
      style: GoogleFonts.outfit(fontSize: 13, color: const Color(0xFF0F172A)),
      decoration: InputDecoration(
        hintText: 'Buscar alertas...',
        hintStyle: GoogleFonts.outfit(color: const Color(0xFF94A3B8), fontSize: 13),
        prefixIcon: const Icon(Icons.search_rounded, color: Color(0xFF64748B), size: 18),
        suffixIcon: appState.searchQuery.isNotEmpty
            ? IconButton(
                icon: const Icon(Icons.clear_rounded, color: Color(0xFF64748B), size: 18),
                onPressed: () {
                  _controller.clear();
                  appState.setSearchQuery('');
                },
              )
            : null,
        filled: true,
        fillColor: const Color(0xFFF1F5F9),
        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(10),
          borderSide: const BorderSide(color: Color(0xFFE2E8F0), width: 1.5),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(10),
          borderSide: const BorderSide(color: Color(0xFF0288D1), width: 1.5),
        ),
      ),
    );
  }
}

class _KpiHighlights extends StatelessWidget {
  const _KpiHighlights();

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);
    final alerts = appState.filteredAlerts;

    final total = alerts.length;
    final critical = alerts.where((a) => a.severity == EcoSeverity.critico).length;
    
    final solvedCount = alerts.where((a) => a.status == 'solved').length;
    final solvedPercentage = total > 0 ? (solvedCount / total * 100).round() : 0;
    
    String recentText = 'N/A';
    if (alerts.isNotEmpty) {
      final latest = alerts.reduce((curr, next) => curr.createdAt.isAfter(next.createdAt) ? curr : next);
      final diff = DateTime.now().difference(latest.createdAt);
      if (diff.inDays > 0) {
        recentText = '${diff.inDays}d';
      } else if (diff.inHours > 0) {
        recentText = '${diff.inHours}h';
      } else {
        recentText = '${diff.inMinutes}m';
      }
    }

    return Row(
      children: [
        Expanded(
          child: _buildKpiCard('Total', total, const Color(0xFF0288D1), true),
        ),
        const SizedBox(width: 6),
        Expanded(
          child: _buildKpiCard('Críticos', critical, const Color(0xFFD32F2F), true),
        ),
        const SizedBox(width: 6),
        Expanded(
          child: _buildKpiCard('Resueltas', '$solvedPercentage%', const Color(0xFF10B981), false),
        ),
        const SizedBox(width: 6),
        Expanded(
          child: _buildKpiCard('Reciente', recentText, const Color(0xFFFFA000), false),
        ),
      ],
    );
  }

  Widget _buildKpiCard(String label, dynamic value, Color color, bool isNumeric) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 10, horizontal: 4),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: const Color(0xFFE2E8F0), width: 1.5),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.02),
            blurRadius: 4,
            offset: const Offset(0, 2),
          )
        ],
      ),
      child: Column(
        children: [
          Text(
            label.toUpperCase(),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: GoogleFonts.outfit(
              fontSize: 9,
              fontWeight: FontWeight.w800,
              color: const Color(0xFF94A3B8),
            ),
          ),
          const SizedBox(height: 4),
          isNumeric
              ? _AnimatedCounter(
                  value: value as int,
                  style: GoogleFonts.outfit(
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                    color: color,
                  ),
                )
              : Text(
                  value.toString(),
                  style: GoogleFonts.outfit(
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                    color: color,
                  ),
                ),
        ],
      ),
    );
  }
}

class _DistrictFilterRow extends StatelessWidget {
  const _DistrictFilterRow();

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);
    final selected = appState.selectedDistrict;

    final districts = [
      {'val': null, 'label': 'Todos'},
      {'val': EcoDistrict.chaupimarca, 'label': 'Chaupi.'},
      {'val': EcoDistrict.yanacancha, 'label': 'Yana.'},
      {'val': EcoDistrict.simonBolivar, 'label': 'Rancas'},
    ];

    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      physics: const BouncingScrollPhysics(),
      child: Row(
        children: districts.map((item) {
          final val = item['val'] as EcoDistrict?;
          final label = item['label'] as String;
          final isSelected = selected == val;

          return Padding(
            padding: const EdgeInsets.only(right: 6.0),
            child: GestureDetector(
              onTap: () {
                appState.setDistrict(val);
              },
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 150),
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                decoration: BoxDecoration(
                  color: isSelected ? const Color(0xFF0288D1) : const Color(0xFFF1F5F9),
                  borderRadius: BorderRadius.circular(16),
                  border: Border.all(
                    color: isSelected ? const Color(0xFF0288D1) : const Color(0xFFE2E8F0),
                    width: 1.5,
                  ),
                ),
                child: Text(
                  label,
                  style: GoogleFonts.outfit(
                    color: isSelected ? Colors.white : const Color(0xFF475569),
                    fontSize: 11,
                    fontWeight: isSelected ? FontWeight.bold : FontWeight.w500,
                  ),
                ),
              ),
            ),
          );
        }).toList(),
      ),
    );
  }
}

class _AnimatedCounter extends StatelessWidget {
  final int value;
  final TextStyle style;

  const _AnimatedCounter({required this.value, required this.style});

  @override
  Widget build(BuildContext context) {
    return TweenAnimationBuilder<double>(
      tween: Tween<double>(begin: 0, end: value.toDouble()),
      duration: const Duration(milliseconds: 300),
      builder: (context, val, child) {
        return Text(
          val.toInt().toString(),
          style: style,
        );
      },
    );
  }
}

class _DaysFilterRow extends StatelessWidget {
  const _DaysFilterRow();

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);
    final selectedDays = appState.daysFilter;

    final options = [
      {'val': 0, 'label': 'Historial'},
      {'val': 1, 'label': '24h'},
      {'val': 7, 'label': '7 días'},
      {'val': 30, 'label': '30 días'},
    ];

    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      physics: const BouncingScrollPhysics(),
      child: Row(
        children: options.map((item) {
          final val = item['val'] as int;
          final label = item['label'] as String;
          final isSelected = selectedDays == val;

          return Padding(
            padding: const EdgeInsets.only(right: 6.0),
            child: GestureDetector(
              onTap: () {
                appState.setDaysFilter(val);
              },
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 150),
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                decoration: BoxDecoration(
                  color: isSelected ? const Color(0xFF0288D1) : const Color(0xFFF1F5F9),
                  borderRadius: BorderRadius.circular(16),
                  border: Border.all(
                    color: isSelected ? const Color(0xFF0288D1) : const Color(0xFFE2E8F0),
                    width: 1.5,
                  ),
                ),
                child: Text(
                  label,
                  style: GoogleFonts.outfit(
                    color: isSelected ? Colors.white : const Color(0xFF475569),
                    fontSize: 11,
                    fontWeight: isSelected ? FontWeight.bold : FontWeight.w500,
                  ),
                ),
              ),
            ),
          );
        }).toList(),
      ),
    );
  }
}

class _EmergencySimulatorCard extends StatelessWidget {
  const _EmergencySimulatorCard();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: const Color(0xFFFEF2F2), // Red 50
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: const Color(0xFFFCA5A5), width: 1.5), // Red 300
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              const Icon(Icons.campaign_rounded, color: Color(0xFFDC2626), size: 20),
              const SizedBox(width: 8),
              Text(
                'SIMULADOR DE CRISIS',
                style: GoogleFonts.outfit(
                  fontSize: 11,
                  fontWeight: FontWeight.w800,
                  color: const Color(0xFF991B1B), // Red 800
                  letterSpacing: 0.5,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            'Simula un incidente crítico en tiempo real para verificar los protocolos de alerta reactiva del panel.',
            style: GoogleFonts.outfit(
              fontSize: 11,
              color: const Color(0xFF7F1D1D),
              height: 1.3,
            ),
          ),
          const SizedBox(height: 12),
          ElevatedButton.icon(
            onPressed: () {
              Provider.of<AppState>(context, listen: false).simulateEmergency();
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  backgroundColor: const Color(0xFFDC2626),
                  duration: const Duration(seconds: 4),
                  content: Row(
                    children: [
                      const Icon(Icons.warning_amber_rounded, color: Colors.white),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          '¡Simulación de Emergencia Crítica lanzada en Cerro de Pasco!',
                          style: GoogleFonts.outfit(color: Colors.white, fontWeight: FontWeight.bold),
                        ),
                      ),
                    ],
                  ),
                ),
              );
            },
            icon: const Icon(Icons.emergency_rounded, color: Colors.white, size: 16),
            label: Text(
              'Lanzar Alerta Crítica',
              style: GoogleFonts.outfit(fontWeight: FontWeight.bold, fontSize: 12),
            ),
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFFDC2626),
              foregroundColor: Colors.white,
              elevation: 0,
              padding: const EdgeInsets.symmetric(vertical: 12),
              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
            ),
          ),
        ],
      ),
    );
  }
}

class HelpGuideDialog extends StatelessWidget {
  const HelpGuideDialog({super.key});

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: Colors.transparent,
      child: Center(
        child: SingleChildScrollView(
          child: Container(
            width: 460,
            decoration: BoxDecoration(
              color: Colors.white,
              borderRadius: BorderRadius.circular(24),
              border: Border.all(color: const Color(0xFFE2E8F0), width: 2),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.12),
                  blurRadius: 24,
                  offset: const Offset(0, 10),
                )
              ],
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Padding(
                  padding: const EdgeInsets.only(top: 24, left: 24, right: 24, bottom: 16),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Row(
                        children: [
                          const Text('🌱', style: TextStyle(fontSize: 22)),
                          const SizedBox(width: 8),
                          Text(
                            'Guía de Uso EcoAlerta',
                            style: GoogleFonts.outfit(
                              color: const Color(0xFF0F172A),
                              fontSize: 18,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ],
                      ),
                      IconButton(
                        icon: const Icon(Icons.close_rounded, color: Color(0xFF94A3B8)),
                        onPressed: () => Navigator.of(context).pop(),
                      ),
                    ],
                  ),
                ),
                const Divider(color: Color(0xFFE2E8F0), height: 1),
                Padding(
                  padding: const EdgeInsets.all(24),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      _buildGuideItem(
                        '🔍 Buscar y Filtrar Alertas',
                        'Utiliza el buscador por palabras, selecciona distritos específicos de Cerro de Pasco o escoge el rango temporal (24 horas, semana, mes) en la barra lateral para acotar la visualización.',
                      ),
                      const SizedBox(height: 16),
                      _buildGuideItem(
                        '📍 Explorar e Interactuar',
                        'Haz clic en cualquier pin del mapa o tarjeta del listado inferior para enfocar e inspeccionar los detalles de la denuncia. Si tiene foto, presiona la miniatura para verla en pantalla completa.',
                      ),
                      const SizedBox(height: 16),
                      _buildGuideItem(
                        '📢 Reportar Nuevo Incidente',
                        'Presiona "+ Reportar Alerta" en el mapa, desplaza la mira central para apuntar al lugar exacto de contaminación, haz clic en "Confirmar" y llena el formulario con descripción y foto.',
                      ),
                      const SizedBox(height: 16),
                      Text(
                        'LEYENDA DE SEVERIDAD',
                        style: GoogleFonts.outfit(
                          fontSize: 10,
                          fontWeight: FontWeight.bold,
                          color: const Color(0xFF64748B),
                          letterSpacing: 1.0,
                        ),
                      ),
                      const SizedBox(height: 8),
                      Row(
                        children: [
                          _buildLegendChip('💀 Crítico', const Color(0xFFD32F2F)),
                          const SizedBox(width: 8),
                          _buildLegendChip('⚠️ Medio', const Color(0xFFFFA000)),
                          const SizedBox(width: 8),
                          _buildLegendChip('🍃 Bajo', const Color(0xFF388E3C)),
                        ],
                      ),
                    ],
                  ),
                ),
                const Divider(color: Color(0xFFE2E8F0), height: 1),
                Padding(
                  padding: const EdgeInsets.all(20),
                  child: ElevatedButton(
                    onPressed: () => Navigator.of(context).pop(),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: const Color(0xFF0288D1),
                      foregroundColor: Colors.white,
                      padding: const EdgeInsets.symmetric(vertical: 14),
                      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                      elevation: 0,
                    ),
                    child: Text('Entendido, ¡Comenzar!', style: GoogleFonts.outfit(fontWeight: FontWeight.bold)),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildGuideItem(String title, String desc) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: GoogleFonts.outfit(
            fontSize: 14,
            fontWeight: FontWeight.bold,
            color: const Color(0xFF0F172A),
          ),
        ),
        const SizedBox(height: 4),
        Text(
          desc,
          style: GoogleFonts.outfit(
            fontSize: 12,
            color: const Color(0xFF475569),
            height: 1.4,
          ),
        ),
      ],
    );
  }

  Widget _buildLegendChip(String label, Color color) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: color.withOpacity(0.12),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withOpacity(0.3)),
      ),
      child: Text(
        label,
        style: GoogleFonts.outfit(
          fontSize: 11,
          fontWeight: FontWeight.bold,
          color: color,
        ),
      ),
    );
  }
}

class _StatusFilterRow extends StatelessWidget {
  const _StatusFilterRow();

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);

    return Row(
      children: [
        // Sin Resolver
        Expanded(
          child: Padding(
            padding: const EdgeInsets.only(right: 4.0),
            child: GestureDetector(
              onTap: () => appState.toggleShowUnresolved(),
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 200),
                padding: const EdgeInsets.symmetric(vertical: 8),
                decoration: BoxDecoration(
                  color: appState.showUnresolved
                      ? const Color(0xFFF59E0B).withOpacity(0.12)
                      : const Color(0xFFF1F5F9),
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(
                    color: appState.showUnresolved ? const Color(0xFFF59E0B) : const Color(0xFFE2E8F0),
                    width: 1.5,
                  ),
                ),
                child: Column(
                  children: [
                    const Icon(Icons.pending_rounded, color: Color(0xFFF59E0B), size: 14),
                    const SizedBox(height: 4),
                    Text(
                      'Sin Resolver',
                      style: GoogleFonts.outfit(
                        fontSize: 10,
                        fontWeight: FontWeight.bold,
                        color: appState.showUnresolved ? const Color(0xFFD97706) : const Color(0xFF64748B),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
        
        // Resueltas
        Expanded(
          child: Padding(
            padding: const EdgeInsets.only(left: 4.0),
            child: GestureDetector(
              onTap: () => appState.toggleShowResolved(),
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 200),
                padding: const EdgeInsets.symmetric(vertical: 8),
                decoration: BoxDecoration(
                  color: appState.showResolved
                      ? const Color(0xFF10B981).withOpacity(0.12)
                      : const Color(0xFFF1F5F9),
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(
                    color: appState.showResolved ? const Color(0xFF10B981) : const Color(0xFFE2E8F0),
                    width: 1.5,
                  ),
                ),
                child: Column(
                  children: [
                    const Icon(Icons.check_circle_rounded, color: Color(0xFF10B981), size: 14),
                    const SizedBox(height: 4),
                    Text(
                      'Resueltas',
                      style: GoogleFonts.outfit(
                        fontSize: 10,
                        fontWeight: FontWeight.bold,
                        color: appState.showResolved ? const Color(0xFF047857) : const Color(0xFF64748B),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _AuthPanelCard extends StatelessWidget {
  const _AuthPanelCard();

  void _showAuthDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => const AuthDialog(),
    );
  }

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);

    if (appState.isLoggedIn) {
      final isAuth = appState.isLoggedInAuthority;
      return Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: isAuth ? const Color(0xFFECFDF5) : const Color(0xFFF0F9FF),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: isAuth ? const Color(0xFFA7F3D0) : const Color(0xFFBEE3F8)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  isAuth ? Icons.verified_user_rounded : Icons.person_rounded,
                  color: isAuth ? const Color(0xFF059669) : const Color(0xFF0288D1),
                  size: 16,
                ),
                const SizedBox(width: 6),
                Text(
                  isAuth ? 'SESIÓN DE AUTORIDAD ACTIVA' : 'SESIÓN DE CIUDADANO ACTIVA',
                  style: GoogleFonts.outfit(
                    color: isAuth ? const Color(0xFF065F46) : const Color(0xFF0369A1),
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                    letterSpacing: 0.5,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              'Usuario: ${appState.loggedUsername}',
              style: GoogleFonts.outfit(
                fontSize: 12,
                fontWeight: FontWeight.bold,
                color: isAuth ? const Color(0xFF047857) : const Color(0xFF075985),
              ),
            ),
            const SizedBox(height: 10),
            if (isAuth) ...[
              SizedBox(
                width: double.infinity,
                height: 32,
                child: ElevatedButton.icon(
                  onPressed: () {
                    appState.showAuthorityDashboard = !appState.showAuthorityDashboard;
                  },
                  icon: Icon(
                    appState.showAuthorityDashboard ? Icons.map_rounded : Icons.admin_panel_settings_rounded,
                    size: 14,
                    color: Colors.white,
                  ),
                  label: Text(
                    appState.showAuthorityDashboard ? 'Ver Mapa de Alertas' : 'Ir al Panel de Control',
                    style: GoogleFonts.outfit(
                      fontSize: 11,
                      fontWeight: FontWeight.bold,
                      color: Colors.white,
                    ),
                  ),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: const Color(0xFF0288D1),
                    elevation: 0,
                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                  ),
                ),
              ),
              const SizedBox(height: 8),
            ],
            SizedBox(
              width: double.infinity,
              height: 32,
              child: ElevatedButton.icon(
                onPressed: () {
                  appState.showTransparencyPortal = !appState.showTransparencyPortal;
                },
                icon: Icon(
                  appState.showTransparencyPortal ? Icons.map_rounded : Icons.emoji_events_rounded,
                  size: 14,
                  color: Colors.white,
                ),
                label: Text(
                  appState.showTransparencyPortal ? 'Ver Mapa de Alertas' : 'Portal de Transparencia',
                  style: GoogleFonts.outfit(
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                    color: Colors.white,
                  ),
                ),
                style: ElevatedButton.styleFrom(
                  backgroundColor: const Color(0xFF10B981),
                  elevation: 0,
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                ),
              ),
            ),
            const SizedBox(height: 8),
            SizedBox(
              width: double.infinity,
              height: 32,
              child: OutlinedButton(
                onPressed: () => appState.logout(),
                style: OutlinedButton.styleFrom(
                  side: const BorderSide(color: Color(0xFFEF4444)),
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                ),
                child: Text(
                  'Cerrar Sesión',
                  style: GoogleFonts.outfit(
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                    color: const Color(0xFFEF4444),
                  ),
                ),
              ),
            ),
          ],
        ),
      );
    } else {
      return Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: const Color(0xFFF8FAFC),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: const Color(0xFFE2E8F0)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.lock_rounded, color: Color(0xFF64748B), size: 16),
                const SizedBox(width: 6),
                Text(
                  'PARTICIPA EN ECOALERTA',
                  style: GoogleFonts.outfit(
                    color: const Color(0xFF64748B),
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                    letterSpacing: 0.5,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 6),
            Text(
              'Inicia sesión o regístrate para reportar alertas ambientales o resolver incidencias.',
              style: GoogleFonts.outfit(fontSize: 11, color: const Color(0xFF475569), height: 1.3),
            ),
            const SizedBox(height: 10),
            SizedBox(
              width: double.infinity,
              height: 32,
              child: ElevatedButton.icon(
                onPressed: () => _showAuthDialog(context),
                icon: const Icon(Icons.login_rounded, size: 12, color: Colors.white),
                label: Text(
                  'Ingresar / Registrarse',
                  style: GoogleFonts.outfit(
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                    color: Colors.white,
                  ),
                ),
                style: ElevatedButton.styleFrom(
                  backgroundColor: const Color(0xFF0288D1),
                  elevation: 0,
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                ),
              ),
            ),
            const SizedBox(height: 8),
            SizedBox(
              width: double.infinity,
              height: 32,
              child: ElevatedButton.icon(
                onPressed: () {
                  appState.showTransparencyPortal = !appState.showTransparencyPortal;
                },
                icon: Icon(
                  appState.showTransparencyPortal ? Icons.map_rounded : Icons.emoji_events_rounded,
                  size: 14,
                  color: Colors.white,
                ),
                label: Text(
                  appState.showTransparencyPortal ? 'Ver Mapa de Alertas' : 'Portal de Transparencia',
                  style: GoogleFonts.outfit(
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                    color: Colors.white,
                  ),
                ),
                style: ElevatedButton.styleFrom(
                  backgroundColor: const Color(0xFF10B981),
                  elevation: 0,
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                ),
              ),
            ),
          ],
        ),
      );
    }
  }
}

class _DismissedFilterRow extends StatelessWidget {
  const _DismissedFilterRow();

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);

    return GestureDetector(
      onTap: () => appState.showDismissed = !appState.showDismissed,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 12),
        decoration: BoxDecoration(
          color: appState.showDismissed
              ? const Color(0xFF64748B).withOpacity(0.12)
              : const Color(0xFFF1F5F9),
          borderRadius: BorderRadius.circular(10),
          border: Border.all(
            color: appState.showDismissed ? const Color(0xFF64748B) : const Color(0xFFE2E8F0),
            width: 1.5,
          ),
        ),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.block_rounded, color: Color(0xFF64748B), size: 14),
            const SizedBox(width: 6),
            Text(
              'Mostrar Descartados',
              style: GoogleFonts.outfit(
                fontSize: 10,
                fontWeight: FontWeight.bold,
                color: appState.showDismissed ? const Color(0xFF475569) : const Color(0xFF64748B),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _ProximityFilterSection extends StatelessWidget {
  const _ProximityFilterSection();

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);
    final isActive = appState.isProximityFilterActive;
    final radius = appState.proximityRadius;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(
              'FILTRO DE CERCANÍA',
              style: GoogleFonts.outfit(
                fontSize: 11,
                fontWeight: FontWeight.w800,
                color: const Color(0xFF64748B),
                letterSpacing: 1.0,
              ),
            ),
            Transform.scale(
              scale: 0.8,
              child: Switch(
                value: isActive,
                activeColor: const Color(0xFF0288D1),
                onChanged: (val) async {
                  if (val) {
                    final pos = await appState.getUserLatLng();
                    if (pos == null) {
                      if (context.mounted) {
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(
                            content: Text(
                              'No se pudo acceder a tu ubicación GPS. Verifica los permisos de tu navegador.',
                            ),
                          ),
                        );
                      }
                      appState.setProximityFilterActive(false);
                      return;
                    }
                  }
                  appState.setProximityFilterActive(val);
                },
              ),
            ),
          ],
        ),
        if (isActive) ...[
          const SizedBox(height: 8),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Mostrar alertas en un radio de:',
                style: GoogleFonts.outfit(
                  color: const Color(0xFF64748B),
                  fontSize: 12,
                ),
              ),
              Text(
                radius >= 1000
                    ? '${(radius / 1000).toStringAsFixed(1)} km'
                    : '${radius.round()} m',
                style: GoogleFonts.outfit(
                  color: const Color(0xFF0288D1),
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ],
          ),
          SliderTheme(
            data: SliderTheme.of(context).copyWith(
              activeTrackColor: const Color(0xFF0288D1),
              inactiveTrackColor: const Color(0xFFE2E8F0),
              thumbColor: const Color(0xFF0288D1),
              overlayColor: const Color(0xFF0288D1).withOpacity(0.12),
              trackHeight: 4,
            ),
            child: Slider(
              value: radius,
              min: 100.0,
              max: 5000.0,
              divisions: 49,
              onChanged: (val) {
                appState.setProximityRadius(val);
              },
            ),
          ),
        ],
      ],
    );
  }
}
