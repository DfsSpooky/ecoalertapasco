import 'dart:typed_data';
import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:provider/provider.dart';
import 'package:file_picker/file_picker.dart';
import '../models/eco_alert.dart';
import '../state/app_state.dart';
import 'pulsing_critical_dot.dart';
import 'eco_notification.dart';

class AuthorityDashboardView extends StatefulWidget {
  const AuthorityDashboardView({super.key});

  @override
  State<AuthorityDashboardView> createState() => _AuthorityDashboardViewState();
}

class _AuthorityDashboardViewState extends State<AuthorityDashboardView> {
  String _searchQuery = '';
  EcoDistrict? _filterDistrict;
  String _filterStatus = 'todos'; // 'todos', 'pending', 'solved', 'dismissed'
  EcoSeverity? _filterSeverity;
  String _filterDateRange = 'todos'; // 'todos', 'hoy', 'semana', 'mes'

  // Controladores de diálogo de transferencia
  EcoDistrict _selectedTransferDistrict = EcoDistrict.chaupimarca;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final appState = Provider.of<AppState>(context, listen: false);
      setState(() {
        _filterDistrict = appState.authorityDistrict;
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);
    final allAlerts = appState.allAlerts;
    final screenWidth = MediaQuery.of(context).size.width;
    final isMobileOrTablet = screenWidth < 1050;

    String scopeText;
    switch (appState.authorityDistrict) {
      case EcoDistrict.yanacancha:
        scopeText = 'Municipalidad Distrital de Yanacancha';
        break;
      case EcoDistrict.simonBolivar:
        scopeText = 'Municipalidad Distrital de Simón Bolívar';
        break;
      case EcoDistrict.chaupimarca:
        scopeText = 'Municipalidad Provincial de Pasco';
        break;
    }

    // Calcular estadísticas generales
    final totalCount = allAlerts.length;
    final pendingCount = allAlerts.where((a) => a.status == 'pending' || a.status == 'verified').length;
    final solvedCount = allAlerts.where((a) => a.status == 'solved').length;
    final dismissedCount = allAlerts.where((a) => a.status == 'dismissed').length;

    // Calcular tiempo promedio de solución
    String avgResolutionTime = 'N/A';
    final solvedAlertsCalculated = allAlerts.where((a) => a.status == 'solved').toList();
    if (solvedAlertsCalculated.isNotEmpty) {
      double totalHours = 0;
      int validCases = 0;
      for (final alert in solvedAlertsCalculated) {
        if (alert.resolvedAt != null) {
          totalHours += alert.resolvedAt!.difference(alert.createdAt).inHours;
          validCases++;
        }
      }
      if (validCases > 0) {
        double avgHours = totalHours / validCases;
        if (avgHours < 24) {
          avgResolutionTime = '${avgHours.toStringAsFixed(1)} h';
        } else {
          double avgDays = avgHours / 24;
          avgResolutionTime = '${avgDays.toStringAsFixed(1)} d';
        }
      }
    }

    // Filtrado local para la tabla
    final displayAlerts = allAlerts.where((alert) {
      final matchesSearch = _searchQuery.isEmpty ||
          alert.title.toLowerCase().contains(_searchQuery.toLowerCase()) ||
          alert.description.toLowerCase().contains(_searchQuery.toLowerCase()) ||
          alert.address.toLowerCase().contains(_searchQuery.toLowerCase());

      final matchesDistrict = _filterDistrict == null || alert.district == _filterDistrict;
      
      bool matchesStatus = true;
      if (_filterStatus == 'pending') {
        matchesStatus = alert.status == 'pending' || alert.status == 'verified';
      } else if (_filterStatus == 'solved') {
        matchesStatus = alert.status == 'solved';
      } else if (_filterStatus == 'dismissed') {
        matchesStatus = alert.status == 'dismissed';
      }

      final matchesSeverity = _filterSeverity == null || alert.severity == _filterSeverity;

      bool matchesDate = true;
      if (_filterDateRange == 'hoy') {
        final now = DateTime.now();
        matchesDate = alert.createdAt.year == now.year &&
            alert.createdAt.month == now.month &&
            alert.createdAt.day == now.day;
      } else if (_filterDateRange == 'semana') {
        final now = DateTime.now();
        matchesDate = now.difference(alert.createdAt).inDays <= 7;
      } else if (_filterDateRange == 'mes') {
        final now = DateTime.now();
        matchesDate = alert.createdAt.year == now.year && alert.createdAt.month == now.month;
      }

      return matchesSearch && matchesDistrict && matchesStatus && matchesSeverity && matchesDate;
    }).toList();

    // Ordenar: Casos críticos pendientes van al principio
    displayAlerts.sort((a, b) {
      bool aCritPending = a.severity == EcoSeverity.critico && a.status != 'solved' && a.status != 'dismissed';
      bool bCritPending = b.severity == EcoSeverity.critico && b.status != 'solved' && b.status != 'dismissed';
      if (aCritPending && !bCritPending) return -1;
      if (!aCritPending && bCritPending) return 1;
      return b.createdAt.compareTo(a.createdAt); // Orden descendente normal por creación
    });

    return Scaffold(
      backgroundColor: const Color(0xFFF1F5F9), // Slate 100
      appBar: AppBar(
        backgroundColor: Colors.white,
        elevation: 0,
        centerTitle: false,
        title: Row(
          children: [
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: const Color(0xFF0288D1).withOpacity(0.1),
                borderRadius: BorderRadius.circular(12),
              ),
              child: const Icon(Icons.admin_panel_settings_rounded, color: Color(0xFF0288D1), size: 28),
            ),
            const SizedBox(width: 12),
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Panel de Control de Autoridades',
                  style: GoogleFonts.outfit(
                    fontSize: isMobileOrTablet ? 16 : 20,
                    fontWeight: FontWeight.bold,
                    color: const Color(0xFF0F172A),
                  ),
                ),
                Text(
                  scopeText,
                  style: GoogleFonts.outfit(
                    fontSize: isMobileOrTablet ? 10 : 12,
                    color: const Color(0xFF64748B),
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ],
            ),
          ],
        ),
        actions: [
          // Botón para volver al mapa
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8),
            child: ElevatedButton.icon(
              onPressed: () {
                appState.showAuthorityDashboard = false;
              },
              icon: const Icon(Icons.map_rounded, size: 18, color: Colors.white),
              label: Text(
                'Ver Mapa de Alertas',
                style: GoogleFonts.outfit(fontWeight: FontWeight.bold, color: Colors.white),
              ),
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF0288D1),
                elevation: 0,
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
              ),
            ),
          ),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(24.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 1. Fila de KPIs (Responsiva)
            if (isMobileOrTablet)
              Column(
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: _buildKpiCard(
                          title: 'Total Reportes',
                          value: '$totalCount',
                          icon: Icons.assignment_rounded,
                          color: const Color(0xFF0288D1),
                          subtitle: 'Acumulados',
                        ),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: _buildKpiCard(
                          title: 'Casos por Resolver',
                          value: '$pendingCount',
                          icon: Icons.pending_actions_rounded,
                          color: const Color(0xFFFFA000),
                          subtitle: 'Pendientes',
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Row(
                    children: [
                      Expanded(
                        child: _buildKpiCard(
                          title: 'Casos Resueltos',
                          value: '$solvedCount',
                          icon: Icons.task_alt_rounded,
                          color: const Color(0xFF10B981),
                          subtitle: 'Con evidencia',
                        ),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: _buildKpiCard(
                          title: 'Casos Descartados',
                          value: '$dismissedCount',
                          icon: Icons.block_flipped,
                          color: const Color(0xFF64748B),
                          subtitle: 'Ocultados',
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Row(
                    children: [
                      Expanded(
                        child: _buildKpiCard(
                          title: 'Tiempo de Solución',
                          value: avgResolutionTime,
                          icon: Icons.timer_rounded,
                          color: const Color(0xFF673AB7),
                          subtitle: 'Promedio de respuesta',
                        ),
                      ),
                    ],
                  ),
                ],
              )
            else
              Row(
                children: [
                  Expanded(
                    child: _buildKpiCard(
                      title: 'Total Reportes',
                      value: '$totalCount',
                      icon: Icons.assignment_rounded,
                      color: const Color(0xFF0288D1), // Azul
                      subtitle: 'Acumulados en el sistema',
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: _buildKpiCard(
                      title: 'Casos por Resolver',
                      value: '$pendingCount',
                      icon: Icons.pending_actions_rounded,
                      color: const Color(0xFFFFA000), // Naranja
                      subtitle: 'Requieren atención urgente',
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: _buildKpiCard(
                      title: 'Casos Resueltos',
                      value: '$solvedCount',
                      icon: Icons.task_alt_rounded,
                      color: const Color(0xFF10B981), // Verde
                      subtitle: 'Solucionados con evidencia',
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: _buildKpiCard(
                      title: 'Casos Descartados',
                      value: '$dismissedCount',
                      icon: Icons.block_flipped,
                      color: const Color(0xFF64748B), // Gris
                      subtitle: 'Errores / Ocultados del mapa',
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: _buildKpiCard(
                      title: 'Tiempo de Solución',
                      value: avgResolutionTime,
                      icon: Icons.timer_rounded,
                      color: const Color(0xFF673AB7), // Morado
                      subtitle: 'Promedio de respuesta',
                    ),
                  ),
                ],
              ),
            const SizedBox(height: 24),

            // 2. Fila de Gráficos Analíticos (Responsiva)
            if (isMobileOrTablet)
              Column(
                children: [
                  // Gráfico 1: Categorías
                  Container(
                    height: 220,
                    padding: const EdgeInsets.all(20),
                    decoration: BoxDecoration(
                      color: Colors.white,
                      borderRadius: BorderRadius.circular(16),
                      border: Border.all(color: const Color(0xFFE2E8F0)),
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Distribución por Categoría',
                          style: GoogleFonts.outfit(
                            fontWeight: FontWeight.bold,
                            fontSize: 14,
                            color: const Color(0xFF0F172A),
                          ),
                        ),
                        const SizedBox(height: 16),
                        Expanded(
                          child: _buildCategoryPieChart(allAlerts),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 16),
                  // Gráfico 2: Gravedad
                  Container(
                    height: 220,
                    padding: const EdgeInsets.all(20),
                    decoration: BoxDecoration(
                      color: Colors.white,
                      borderRadius: BorderRadius.circular(16),
                      border: Border.all(color: const Color(0xFFE2E8F0)),
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Casos por Nivel de Gravedad',
                          style: GoogleFonts.outfit(
                            fontWeight: FontWeight.bold,
                            fontSize: 14,
                            color: const Color(0xFF0F172A),
                          ),
                        ),
                        const SizedBox(height: 16),
                        Expanded(
                          child: _buildSeverityBarChart(allAlerts),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 16),
                  // Gráfico 3: Municipio Competente
                  Container(
                    height: 220,
                    padding: const EdgeInsets.all(20),
                    decoration: BoxDecoration(
                      color: Colors.white,
                      borderRadius: BorderRadius.circular(16),
                      border: Border.all(color: const Color(0xFFE2E8F0)),
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Carga de Trabajo por Municipalidad',
                          style: GoogleFonts.outfit(
                            fontWeight: FontWeight.bold,
                            fontSize: 14,
                            color: const Color(0xFF0F172A),
                          ),
                        ),
                        const SizedBox(height: 12),
                        Expanded(
                          child: _buildMunicipalityLoadList(allAlerts),
                        ),
                      ],
                    ),
                  ),
                ],
              )
            else
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Gráfico 1: Categorías
                  Expanded(
                    flex: 3,
                    child: Container(
                      height: 220,
                      padding: const EdgeInsets.all(20),
                      decoration: BoxDecoration(
                        color: Colors.white,
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(color: const Color(0xFFE2E8F0)),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            'Distribución por Categoría',
                            style: GoogleFonts.outfit(
                              fontWeight: FontWeight.bold,
                              fontSize: 14,
                              color: const Color(0xFF0F172A),
                            ),
                          ),
                          const SizedBox(height: 16),
                          Expanded(
                            child: _buildCategoryPieChart(allAlerts),
                          ),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(width: 16),
                  // Gráfico 2: Gravedad
                  Expanded(
                    flex: 3,
                    child: Container(
                      height: 220,
                      padding: const EdgeInsets.all(20),
                      decoration: BoxDecoration(
                        color: Colors.white,
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(color: const Color(0xFFE2E8F0)),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            'Casos por Nivel de Gravedad',
                            style: GoogleFonts.outfit(
                              fontWeight: FontWeight.bold,
                              fontSize: 14,
                              color: const Color(0xFF0F172A),
                            ),
                          ),
                          const SizedBox(height: 16),
                          Expanded(
                            child: _buildSeverityBarChart(allAlerts),
                          ),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(width: 16),
                  // Gráfico 3: Municipio Competente
                  Expanded(
                    flex: 4,
                    child: Container(
                      height: 220,
                      padding: const EdgeInsets.all(20),
                      decoration: BoxDecoration(
                        color: Colors.white,
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(color: const Color(0xFFE2E8F0)),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            'Carga de Trabajo por Municipalidad',
                            style: GoogleFonts.outfit(
                              fontWeight: FontWeight.bold,
                              fontSize: 14,
                              color: const Color(0xFF0F172A),
                            ),
                          ),
                          const SizedBox(height: 12),
                          Expanded(
                            child: _buildMunicipalityLoadList(allAlerts),
                          ),
                        ],
                      ),
                    ),
                  ),
                ],
              ),
            const SizedBox(height: 24),

            // 3. Tabla y Filtros de Denuncias
            Container(
              padding: const EdgeInsets.all(24),
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(16),
                border: Border.all(color: const Color(0xFFE2E8F0)),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  // Fila de Filtros (Responsiva)
                  if (isMobileOrTablet)
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Bandeja de Casos Ambientales',
                          style: GoogleFonts.outfit(
                            fontSize: 15,
                            fontWeight: FontWeight.bold,
                            color: const Color(0xFF0F172A),
                          ),
                        ),
                        const SizedBox(height: 12),
                        SizedBox(
                          width: double.infinity,
                          height: 38,
                          child: TextField(
                            onChanged: (value) {
                              setState(() {
                                _searchQuery = value;
                              });
                            },
                            style: GoogleFonts.outfit(fontSize: 12),
                            decoration: InputDecoration(
                              hintText: 'Buscar caso...',
                              prefixIcon: const Icon(Icons.search_rounded, size: 16),
                              contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                              filled: true,
                              fillColor: const Color(0xFFF8FAFC),
                              enabledBorder: OutlineInputBorder(
                                borderRadius: BorderRadius.circular(8),
                                borderSide: const BorderSide(color: Color(0xFFE2E8F0)),
                              ),
                              focusedBorder: OutlineInputBorder(
                                borderRadius: BorderRadius.circular(8),
                                borderSide: const BorderSide(color: Color(0xFF0288D1)),
                              ),
                            ),
                          ),
                        ),
                        const SizedBox(height: 10),
                        Wrap(
                          spacing: 8,
                          runSpacing: 8,
                          children: [
                            _buildFilterDropdown<String>(
                              value: _filterStatus,
                              items: const [
                                DropdownMenuItem(value: 'todos', child: Text('Todos los Estados')),
                                DropdownMenuItem(value: 'pending', child: Text('⏳ Pendientes')),
                                DropdownMenuItem(value: 'solved', child: Text('✅ Resueltos')),
                                DropdownMenuItem(value: 'dismissed', child: Text('🚫 Descartados')),
                              ],
                              onChanged: (val) {
                                setState(() {
                                  _filterStatus = val ?? 'todos';
                                });
                              },
                            ),
                            _buildFilterDropdown<EcoDistrict?>(
                              value: _filterDistrict,
                              items: const [
                                DropdownMenuItem(value: null, child: Text('Todas las Munis')),
                                DropdownMenuItem(value: EcoDistrict.chaupimarca, child: Text('🏛️ M. Prov. Pasco')),
                                DropdownMenuItem(value: EcoDistrict.yanacancha, child: Text('🏢 M. Dist. Yanacancha')),
                                DropdownMenuItem(value: EcoDistrict.simonBolivar, child: Text('🚜 M. Dist. Simón Bolívar')),
                              ],
                              onChanged: (val) {
                                setState(() {
                                  _filterDistrict = val;
                                });
                              },
                            ),
                            _buildFilterDropdown<EcoSeverity?>(
                              value: _filterSeverity,
                              items: const [
                                DropdownMenuItem(value: null, child: Text('Todas las Gravedades')),
                                DropdownMenuItem(value: EcoSeverity.critico, child: Text('💀 Crítico')),
                                DropdownMenuItem(value: EcoSeverity.medio, child: Text('⚠️ Medio')),
                                DropdownMenuItem(value: EcoSeverity.bajo, child: Text('🍃 Bajo')),
                              ],
                              onChanged: (val) {
                                setState(() {
                                  _filterSeverity = val;
                                });
                              },
                            ),
                            _buildFilterDropdown<String>(
                              value: _filterDateRange,
                              items: const [
                                DropdownMenuItem(value: 'todos', child: Text('Todas las Fechas')),
                                DropdownMenuItem(value: 'hoy', child: Text('📅 Hoy')),
                                DropdownMenuItem(value: 'semana', child: Text('📅 Últimos 7 días')),
                                DropdownMenuItem(value: 'mes', child: Text('📅 Este mes')),
                              ],
                              onChanged: (val) {
                                setState(() {
                                  _filterDateRange = val ?? 'todos';
                                });
                              },
                            ),
                          ],
                        ),
                      ],
                    )
                  else
                    Row(
                      children: [
                        Text(
                          'Bandeja de Casos Ambientales',
                          style: GoogleFonts.outfit(
                            fontSize: 16,
                            fontWeight: FontWeight.bold,
                            color: const Color(0xFF0F172A),
                          ),
                        ),
                        const Spacer(),
                        // Campo de Búsqueda
                        SizedBox(
                          width: 200,
                          height: 38,
                          child: TextField(
                            onChanged: (value) {
                              setState(() {
                                _searchQuery = value;
                              });
                            },
                            style: GoogleFonts.outfit(fontSize: 12),
                            decoration: InputDecoration(
                              hintText: 'Buscar caso...',
                              prefixIcon: const Icon(Icons.search_rounded, size: 16),
                              contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                              filled: true,
                              fillColor: const Color(0xFFF8FAFC),
                              enabledBorder: OutlineInputBorder(
                                borderRadius: BorderRadius.circular(8),
                                borderSide: const BorderSide(color: Color(0xFFE2E8F0)),
                              ),
                              focusedBorder: OutlineInputBorder(
                                borderRadius: BorderRadius.circular(8),
                                borderSide: const BorderSide(color: Color(0xFF0288D1)),
                              ),
                            ),
                          ),
                        ),
                        const SizedBox(width: 8),
                        // Filtro Estado
                        _buildFilterDropdown<String>(
                          value: _filterStatus,
                          items: const [
                            DropdownMenuItem(value: 'todos', child: Text('Todos los Estados')),
                            DropdownMenuItem(value: 'pending', child: Text('⏳ Pendientes')),
                            DropdownMenuItem(value: 'solved', child: Text('✅ Resueltos')),
                            DropdownMenuItem(value: 'dismissed', child: Text('🚫 Descartados')),
                          ],
                          onChanged: (val) {
                            setState(() {
                              _filterStatus = val ?? 'todos';
                            });
                          },
                        ),
                        const SizedBox(width: 8),
                        // Filtro Distrito / Municipio
                        _buildFilterDropdown<EcoDistrict?>(
                          value: _filterDistrict,
                          items: const [
                            DropdownMenuItem(value: null, child: Text('Todas las Munis')),
                            DropdownMenuItem(value: EcoDistrict.chaupimarca, child: Text('🏛️ M. Prov. Pasco')),
                            DropdownMenuItem(value: EcoDistrict.yanacancha, child: Text('🏢 M. Dist. Yanacancha')),
                            DropdownMenuItem(value: EcoDistrict.simonBolivar, child: Text('🚜 M. Dist. Simón Bolívar')),
                          ],
                          onChanged: (val) {
                            setState(() {
                              _filterDistrict = val;
                            });
                          },
                        ),
                        const SizedBox(width: 8),
                        // Filtro Gravedad
                        _buildFilterDropdown<EcoSeverity?>(
                          value: _filterSeverity,
                          items: const [
                            DropdownMenuItem(value: null, child: Text('Todas las Gravedades')),
                            DropdownMenuItem(value: EcoSeverity.critico, child: Text('💀 Crítico')),
                            DropdownMenuItem(value: EcoSeverity.medio, child: Text('⚠️ Medio')),
                            DropdownMenuItem(value: EcoSeverity.bajo, child: Text('🍃 Bajo')),
                          ],
                          onChanged: (val) {
                            setState(() {
                              _filterSeverity = val;
                            });
                          },
                        ),
                        const SizedBox(width: 8),
                        _buildFilterDropdown<String>(
                          value: _filterDateRange,
                          items: const [
                            DropdownMenuItem(value: 'todos', child: Text('Todas las Fechas')),
                            DropdownMenuItem(value: 'hoy', child: Text('📅 Hoy')),
                            DropdownMenuItem(value: 'semana', child: Text('📅 Últimos 7 días')),
                            DropdownMenuItem(value: 'mes', child: Text('📅 Este mes')),
                          ],
                          onChanged: (val) {
                            setState(() {
                              _filterDateRange = val ?? 'todos';
                            });
                          },
                        ),
                      ],
                    ),
                  const SizedBox(height: 20),

                  // Tabla de Contenido (Responsiva)
                  displayAlerts.isEmpty
                      ? Container(
                          padding: const EdgeInsets.symmetric(vertical: 40),
                          child: Column(
                            children: [
                              const Icon(Icons.folder_open_rounded, size: 48, color: Color(0xFF94A3B8)),
                              const SizedBox(height: 12),
                              Text(
                                'No se encontraron denuncias con los filtros actuales.',
                                style: GoogleFonts.outfit(
                                  color: const Color(0xFF64748B),
                                  fontSize: 13,
                                  fontWeight: FontWeight.w500,
                                ),
                              ),
                            ],
                          ),
                        )
                      : isMobileOrTablet
                          ? ListView.builder(
                              shrinkWrap: true,
                              physics: const NeverScrollableScrollPhysics(),
                              itemCount: displayAlerts.length,
                              itemBuilder: (context, index) {
                                final alert = displayAlerts[index];
                                return Card(
                                  color: Colors.white,
                                  elevation: 0,
                                  shape: RoundedRectangleBorder(
                                    borderRadius: BorderRadius.circular(12),
                                    side: const BorderSide(color: Color(0xFFE2E8F0)),
                                  ),
                                  margin: const EdgeInsets.only(bottom: 12),
                                  child: Padding(
                                    padding: const EdgeInsets.all(16.0),
                                    child: Column(
                                      crossAxisAlignment: CrossAxisAlignment.start,
                                      children: [
                                        Row(
                                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                                          children: [
                                            _buildCategoryBadge(alert.category),
                                            _buildStatusBadge(alert.status),
                                          ],
                                        ),
                                        const SizedBox(height: 12),
                                        Text(
                                          alert.title,
                                          style: GoogleFonts.outfit(
                                            fontWeight: FontWeight.bold,
                                            fontSize: 14,
                                            color: const Color(0xFF0F172A),
                                          ),
                                        ),
                                        const SizedBox(height: 4),
                                        Row(
                                          children: [
                                            const Icon(Icons.location_on_rounded, size: 12, color: Color(0xFF64748B)),
                                            const SizedBox(width: 4),
                                            Expanded(
                                              child: Text(
                                                alert.address,
                                                style: GoogleFonts.outfit(fontSize: 11, color: const Color(0xFF64748B)),
                                              ),
                                            ),
                                          ],
                                        ),
                                        const SizedBox(height: 12),
                                        Row(
                                          children: [
                                            Row(
                                              mainAxisSize: MainAxisSize.min,
                                              children: [
                                                if (alert.severity == EcoSeverity.critico && alert.status != 'solved' && alert.status != 'dismissed') ...[
                                                  const PulsingCriticalDot(),
                                                  const SizedBox(width: 6),
                                                ],
                                                _buildSeverityBadge(alert.severity),
                                              ],
                                            ),
                                            const SizedBox(width: 8),
                                            _buildMunicipalityBadge(alert.district),
                                          ],
                                        ),
                                        const SizedBox(height: 16),
                                        const Divider(height: 1, color: Color(0xFFE2E8F0)),
                                        const SizedBox(height: 12),
                                        Wrap(
                                          spacing: 6,
                                          runSpacing: 4,
                                          children: [
                                            if (alert.status != 'solved' && alert.status != 'dismissed')
                                              _buildMiniActionButton(
                                                label: 'Resolver',
                                                icon: Icons.check_circle_rounded,
                                                color: const Color(0xFF10B981),
                                                onPressed: () => _showResolveDialog(context, alert, appState),
                                              ),
                                            _buildMiniActionButton(
                                              label: 'Reenviar',
                                              icon: Icons.swap_horiz_rounded,
                                              color: const Color(0xFF0288D1),
                                              onPressed: () => _showTransferDialog(context, alert, appState),
                                            ),
                                            if (alert.status != 'dismissed')
                                              _buildMiniActionButton(
                                                label: 'Descartar',
                                                icon: Icons.delete_outline_rounded,
                                                color: const Color(0xFFEF4444),
                                                onPressed: () => _showDismissDialog(context, alert, appState),
                                              ),
                                            _buildMiniActionButton(
                                              label: 'Mapa',
                                              icon: Icons.gps_fixed_rounded,
                                              color: const Color(0xFF64748B),
                                              onPressed: () {
                                                appState.showAuthorityDashboard = false;
                                                appState.setDistrict(alert.district);
                                                appState.selectedAlert = alert;
                                                EcoNotification.show(
                                                  context,
                                                  title: 'Mapa Centrado',
                                                  message: 'Enfocando en: ${alert.title}',
                                                  type: EcoNotificationType.info,
                                                );
                                              },
                                            ),
                                            _buildMiniActionButton(
                                              label: 'Bitácora',
                                              icon: Icons.history_rounded,
                                              color: const Color(0xFF7C3AED),
                                              onPressed: () => _showHistoryTimelineDialog(context, alert),
                                            ),
                                            if (alert.status == 'solved')
                                              _buildMiniActionButton(
                                                label: 'Evidencia',
                                                icon: Icons.compare_rounded,
                                                color: const Color(0xFF10B981),
                                                onPressed: () => _showComparisonDialog(context, alert),
                                              ),
                                          ],
                                        ),
                                      ],
                                    ),
                                  ),
                                );
                              },
                            )
                          : Container(
                              decoration: BoxDecoration(
                                border: Border.all(color: const Color(0xFFE2E8F0)),
                                borderRadius: BorderRadius.circular(12),
                              ),
                              child: ClipRRect(
                                borderRadius: BorderRadius.circular(12),
                                child: Table(
                                  columnWidths: const {
                                    0: FlexColumnWidth(2.2), // Título
                                    1: FlexColumnWidth(1.0), // Categoría
                                    2: FlexColumnWidth(0.9), // Gravedad
                                    3: FlexColumnWidth(1.4), // Municipio
                                    4: FlexColumnWidth(0.8), // Estado
                                    5: FlexColumnWidth(3.0), // Acciones (Ampliado para soportar bitácora y evidencia)
                                  },
                                  defaultVerticalAlignment: TableCellVerticalAlignment.middle,
                                  children: [
                                    // Fila Header
                                    TableRow(
                                      decoration: const BoxDecoration(color: Color(0xFFF8FAFC)),
                                      children: [
                                        _buildTableHeader('Denuncia / Dirección'),
                                        _buildTableHeader('Categoría'),
                                        _buildTableHeader('Gravedad'),
                                        _buildTableHeader('Muni Responsable'),
                                        _buildTableHeader('Estado'),
                                        _buildTableHeader('Acciones y Gestión'),
                                      ],
                                    ),
                                // Filas de Contenido
                                ...displayAlerts.map((alert) {
                                  return TableRow(
                                    decoration: const BoxDecoration(
                                      border: Border(bottom: BorderSide(color: Color(0xFFE2E8F0))),
                                    ),
                                    children: [
                                      // Denuncia / Dirección
                                      Padding(
                                        padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 12.0),
                                        child: Column(
                                          crossAxisAlignment: CrossAxisAlignment.start,
                                          children: [
                                            Text(
                                              alert.title,
                                              style: GoogleFonts.outfit(
                                                fontWeight: FontWeight.bold,
                                                fontSize: 13,
                                                color: const Color(0xFF0F172A),
                                              ),
                                            ),
                                            const SizedBox(height: 2),
                                            Row(
                                              children: [
                                                const Icon(Icons.location_on_rounded, size: 10, color: Color(0xFF64748B)),
                                                const SizedBox(width: 4),
                                                Expanded(
                                                  child: Text(
                                                    alert.address,
                                                    overflow: TextOverflow.ellipsis,
                                                    style: GoogleFonts.outfit(
                                                      fontSize: 11,
                                                      color: const Color(0xFF64748B),
                                                    ),
                                                  ),
                                                ),
                                              ],
                                            ),
                                          ],
                                        ),
                                      ),
                                      // Categoría
                                      Padding(
                                        padding: const EdgeInsets.symmetric(horizontal: 12.0),
                                        child: _buildCategoryBadge(alert.category),
                                      ),
                                      // Gravedad
                                      Padding(
                                        padding: const EdgeInsets.symmetric(horizontal: 12.0),
                                        child: Row(
                                          mainAxisSize: MainAxisSize.min,
                                          children: [
                                            if (alert.severity == EcoSeverity.critico && alert.status != 'solved' && alert.status != 'dismissed') ...[
                                              const PulsingCriticalDot(),
                                              const SizedBox(width: 6),
                                            ],
                                            _buildSeverityBadge(alert.severity),
                                          ],
                                        ),
                                      ),
                                      // Municipio
                                      Padding(
                                        padding: const EdgeInsets.symmetric(horizontal: 12.0),
                                        child: _buildMunicipalityBadge(alert.district),
                                      ),
                                      // Estado
                                      Padding(
                                        padding: const EdgeInsets.symmetric(horizontal: 12.0),
                                        child: _buildStatusBadge(alert.status),
                                      ),
                                      // Acciones
                                      Padding(
                                        padding: const EdgeInsets.symmetric(horizontal: 12.0, vertical: 8.0),
                                        child: Wrap(
                                          spacing: 6,
                                          runSpacing: 4,
                                          children: [
                                            // Resolver (solo si no está resuelto ni descartado)
                                            if (alert.status != 'solved' && alert.status != 'dismissed')
                                              _buildMiniActionButton(
                                                label: 'Resolver',
                                                icon: Icons.check_circle_rounded,
                                                color: const Color(0xFF10B981),
                                                onPressed: () => _showResolveDialog(context, alert, appState),
                                              ),
                                            // Reenviar / Transferir (siempre disponible)
                                            _buildMiniActionButton(
                                              label: 'Reenviar',
                                              icon: Icons.swap_horiz_rounded,
                                              color: const Color(0xFF0288D1),
                                              onPressed: () => _showTransferDialog(context, alert, appState),
                                            ),
                                            // Ocultar / Descartar (solo si no está descartado)
                                            if (alert.status != 'dismissed')
                                              _buildMiniActionButton(
                                                label: 'Descartar',
                                                icon: Icons.delete_outline_rounded,
                                                color: const Color(0xFFEF4444),
                                                onPressed: () => _showDismissDialog(context, alert, appState),
                                              ),
                                            // Enfocar Mapa
                                            _buildMiniActionButton(
                                              label: 'Mapa',
                                              icon: Icons.gps_fixed_rounded,
                                              color: const Color(0xFF64748B),
                                              onPressed: () {
                                                // Cambiar vista al mapa, configurar el distrito y enfocar en las coordenadas
                                                appState.showAuthorityDashboard = false;
                                                appState.setDistrict(alert.district);
                                                appState.selectedAlert = alert;
                                                // Opcionalmente mostrar notificación
                                                ScaffoldMessenger.of(context).showSnackBar(
                                                  SnackBar(
                                                    backgroundColor: const Color(0xFF0288D1),
                                                    content: Text('Enfocando en: ${alert.title}', style: GoogleFonts.outfit()),
                                                  ),
                                                );
                                              },
                                            ),
                                            // Bitácora de Auditoría
                                            _buildMiniActionButton(
                                              label: 'Bitácora',
                                              icon: Icons.history_rounded,
                                              color: const Color(0xFF7C3AED),
                                              onPressed: () => _showHistoryTimelineDialog(context, alert),
                                            ),
                                            // Evidencia de Solución
                                            if (alert.status == 'solved')
                                              _buildMiniActionButton(
                                                label: 'Evidencia',
                                                icon: Icons.compare_rounded,
                                                color: const Color(0xFF10B981),
                                                onPressed: () => _showComparisonDialog(context, alert),
                                              ),
                                          ],
                                        ),
                                      ),
                                    ],
                                  );
                                })
                              ],
                            ),
                          ),
                        )
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  // --- Helper Widgets ---

  Widget _buildKpiCard({
    required String title,
    required String value,
    required IconData icon,
    required Color color,
    required String subtitle,
  }) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: const Color(0xFFE2E8F0)),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.01),
            blurRadius: 10,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: color.withOpacity(0.1),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Icon(icon, color: color, size: 26),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  value,
                  style: GoogleFonts.outfit(
                    fontSize: 22,
                    fontWeight: FontWeight.bold,
                    color: const Color(0xFF0F172A),
                  ),
                ),
                Text(
                  title,
                  style: GoogleFonts.outfit(
                    fontSize: 13,
                    fontWeight: FontWeight.bold,
                    color: const Color(0xFF64748B),
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  subtitle,
                  style: GoogleFonts.outfit(
                    fontSize: 10,
                    color: const Color(0xFF94A3B8),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildFilterDropdown<T>({
    required T value,
    required List<DropdownMenuItem<T>> items,
    required ValueChanged<T?> onChanged,
  }) {
    return Container(
      height: 38,
      padding: const EdgeInsets.symmetric(horizontal: 10),
      decoration: BoxDecoration(
        color: const Color(0xFFF8FAFC),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: const Color(0xFFE2E8F0)),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<T>(
          value: value,
          items: items,
          onChanged: onChanged,
          icon: const Icon(Icons.arrow_drop_down_rounded, color: Color(0xFF64748B)),
          style: GoogleFonts.outfit(fontSize: 12, color: const Color(0xFF0F172A), fontWeight: FontWeight.w500),
          dropdownColor: Colors.white,
          borderRadius: BorderRadius.circular(10),
        ),
      ),
    );
  }

  Widget _buildTableHeader(String text) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 12.0),
      child: Text(
        text.toUpperCase(),
        style: GoogleFonts.outfit(
          fontSize: 10,
          fontWeight: FontWeight.bold,
          color: const Color(0xFF64748B),
          letterSpacing: 0.5,
        ),
      ),
    );
  }

  Widget _buildCategoryBadge(EcoCategory category) {
    Color color;
    IconData icon;
    String text;

    switch (category) {
      case EcoCategory.mineria:
        color = const Color(0xFFE65100);
        icon = Icons.terrain_rounded;
        text = 'Minería';
        break;
      case EcoCategory.basura:
        color = const Color(0xFF607D8B);
        icon = Icons.delete_outline_rounded;
        text = 'Basura';
        break;
      case EcoCategory.agua:
        color = const Color(0xFF0288D1);
        icon = Icons.water_drop_rounded;
        text = 'Agua';
        break;
      case EcoCategory.aire:
        color = const Color(0xFF00897B);
        icon = Icons.air_rounded;
        text = 'Aire';
        break;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, color: color, size: 12),
          const SizedBox(width: 4),
          Text(
            text,
            style: GoogleFonts.outfit(
              color: color,
              fontSize: 11,
              fontWeight: FontWeight.bold,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSeverityBadge(EcoSeverity severity) {
    Color color;
    String text;

    switch (severity) {
      case EcoSeverity.critico:
        color = const Color(0xFFD32F2F);
        text = 'Crítico';
        break;
      case EcoSeverity.medio:
        color = const Color(0xFFFFA000);
        text = 'Medio';
        break;
      case EcoSeverity.bajo:
        color = const Color(0xFF388E3C);
        text = 'Bajo';
        break;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.12),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Text(
        text,
        style: GoogleFonts.outfit(
          color: color,
          fontSize: 11,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }

  Widget _buildMunicipalityBadge(EcoDistrict district) {
    String name = '';
    Color color = Colors.black;

    switch (district) {
      case EcoDistrict.chaupimarca:
        name = 'M. Prov. Pasco';
        color = const Color(0xFF6366F1); // Indigo
        break;
      case EcoDistrict.yanacancha:
        name = 'M. Dist. Yanacancha';
        color = const Color(0xFF0288D1); // Blue
        break;
      case EcoDistrict.simonBolivar:
        name = 'M. Dist. Simón Bolívar';
        color = const Color(0xFF8B5CF6); // Violet
        break;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.08),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withOpacity(0.2), width: 1),
      ),
      child: Text(
        name,
        overflow: TextOverflow.ellipsis,
        style: GoogleFonts.outfit(
          color: color,
          fontSize: 10,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }

  Widget _buildStatusBadge(String status) {
    Color color;
    String text;
    IconData icon;

    if (status == 'solved') {
      color = const Color(0xFF10B981);
      text = 'Resuelto';
      icon = Icons.check_circle_outline_rounded;
    } else if (status == 'dismissed') {
      color = const Color(0xFF64748B);
      text = 'Descartado';
      icon = Icons.block_rounded;
    } else {
      color = const Color(0xFFF59E0B);
      text = 'Pendiente';
      icon = Icons.pending_rounded;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, color: color, size: 12),
          const SizedBox(width: 4),
          Text(
            text,
            style: GoogleFonts.outfit(
              color: color,
              fontSize: 11,
              fontWeight: FontWeight.bold,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildMiniActionButton({
    required String label,
    required IconData icon,
    required Color color,
    required VoidCallback onPressed,
  }) {
    return InkWell(
      onTap: onPressed,
      borderRadius: BorderRadius.circular(6),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          border: Border.all(color: color.withOpacity(0.5)),
          borderRadius: BorderRadius.circular(6),
          color: color.withOpacity(0.04),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 12, color: color),
            const SizedBox(width: 4),
            Text(
              label,
              style: GoogleFonts.outfit(
                color: color,
                fontSize: 10,
                fontWeight: FontWeight.bold,
              ),
            ),
          ],
        ),
      ),
    );
  }

  // --- Analíticos: fl_chart logic ---

  Widget _buildCategoryPieChart(List<EcoAlert> alerts) {
    int mineria = alerts.where((a) => a.category == EcoCategory.mineria).length;
    int basura = alerts.where((a) => a.category == EcoCategory.basura).length;
    int agua = alerts.where((a) => a.category == EcoCategory.agua).length;
    int aire = alerts.where((a) => a.category == EcoCategory.aire).length;
    int total = alerts.length;

    if (total == 0) {
      return const Center(child: Text('Sin datos', style: TextStyle(fontSize: 12)));
    }

    final sections = [
      if (mineria > 0)
        PieChartSectionData(
          color: const Color(0xFFE65100),
          value: mineria.toDouble(),
          title: '${(mineria / total * 100).toStringAsFixed(0)}%',
          radius: 35,
          titleStyle: const TextStyle(fontSize: 10, fontWeight: FontWeight.bold, color: Colors.white),
        ),
      if (basura > 0)
        PieChartSectionData(
          color: const Color(0xFF607D8B),
          value: basura.toDouble(),
          title: '${(basura / total * 100).toStringAsFixed(0)}%',
          radius: 35,
          titleStyle: const TextStyle(fontSize: 10, fontWeight: FontWeight.bold, color: Colors.white),
        ),
      if (agua > 0)
        PieChartSectionData(
          color: const Color(0xFF0288D1),
          value: agua.toDouble(),
          title: '${(agua / total * 100).toStringAsFixed(0)}%',
          radius: 35,
          titleStyle: const TextStyle(fontSize: 10, fontWeight: FontWeight.bold, color: Colors.white),
        ),
      if (aire > 0)
        PieChartSectionData(
          color: const Color(0xFF00897B),
          value: aire.toDouble(),
          title: '${(aire / total * 100).toStringAsFixed(0)}%',
          radius: 35,
          titleStyle: const TextStyle(fontSize: 10, fontWeight: FontWeight.bold, color: Colors.white),
        ),
    ];

    return Row(
      children: [
        Expanded(
          flex: 2,
          child: PieChart(
            PieChartData(
              sectionsSpace: 2,
              centerSpaceRadius: 28,
              sections: sections,
            ),
          ),
        ),
        const SizedBox(width: 8),
        Expanded(
          flex: 3,
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildChartLegendItem('Minería', const Color(0xFFE65100), mineria),
              _buildChartLegendItem('Basura', const Color(0xFF607D8B), basura),
              _buildChartLegendItem('Agua', const Color(0xFF0288D1), agua),
              _buildChartLegendItem('Aire', const Color(0xFF00897B), aire),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildChartLegendItem(String title, Color color, int count) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2.0),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(width: 8, height: 8, decoration: BoxDecoration(color: color, shape: BoxShape.circle)),
          const SizedBox(width: 6),
          Text(title, style: const TextStyle(fontSize: 11, color: Color(0xFF475569))),
          const Spacer(),
          Text('$count', style: const TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: Color(0xFF0F172A))),
        ],
      ),
    );
  }

  Widget _buildSeverityBarChart(List<EcoAlert> alerts) {
    int critico = alerts.where((a) => a.severity == EcoSeverity.critico).length;
    int medio = alerts.where((a) => a.severity == EcoSeverity.medio).length;
    int bajo = alerts.where((a) => a.severity == EcoSeverity.bajo).length;

    double maxVal = [critico, medio, bajo].map((e) => e.toDouble()).reduce((curr, next) => curr > next ? curr : next);
    double limitY = maxVal == 0 ? 5 : (maxVal + 2);

    return BarChart(
      BarChartData(
        alignment: BarChartAlignment.spaceAround,
        maxY: limitY,
        barTouchData: BarTouchData(enabled: false),
        gridData: const FlGridData(show: false),
        borderData: FlBorderData(show: false),
        titlesData: FlTitlesData(
          show: true,
          bottomTitles: AxisTitles(
            sideTitles: SideTitles(
              showTitles: true,
              getTitlesWidget: (value, meta) {
                const style = TextStyle(fontSize: 9, fontWeight: FontWeight.bold, color: Color(0xFF64748B));
                switch (value.toInt()) {
                  case 0: return SideTitleWidget(meta: meta, space: 4, child: Text('💀 CRÍTICO', style: style));
                  case 1: return SideTitleWidget(meta: meta, space: 4, child: Text('⚠️ MEDIO', style: style));
                  case 2: return SideTitleWidget(meta: meta, space: 4, child: Text('🍃 BAJO', style: style));
                }
                return const Text('');
              },
            ),
          ),
          leftTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
          topTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
          rightTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
        ),
        barGroups: [
          BarChartGroupData(
            x: 0,
            barRods: [BarChartRodData(toY: critico.toDouble(), color: const Color(0xFFD32F2F), width: 24, borderRadius: const BorderRadius.vertical(top: Radius.circular(4)))],
          ),
          BarChartGroupData(
            x: 1,
            barRods: [BarChartRodData(toY: medio.toDouble(), color: const Color(0xFFFFA000), width: 24, borderRadius: const BorderRadius.vertical(top: Radius.circular(4)))],
          ),
          BarChartGroupData(
            x: 2,
            barRods: [BarChartRodData(toY: bajo.toDouble(), color: const Color(0xFF388E3C), width: 24, borderRadius: const BorderRadius.vertical(top: Radius.circular(4)))],
          ),
        ],
      ),
    );
  }

  Widget _buildMunicipalityLoadList(List<EcoAlert> alerts) {
    int chaupimarca = alerts.where((a) => a.district == EcoDistrict.chaupimarca).length;
    int yanacancha = alerts.where((a) => a.district == EcoDistrict.yanacancha).length;
    int simon = alerts.where((a) => a.district == EcoDistrict.simonBolivar).length;
    int total = alerts.isEmpty ? 1 : alerts.length;

    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        _buildProgressBarItem(
          label: 'Municipalidad Provincial de Pasco',
          count: chaupimarca,
          percentage: chaupimarca / total,
          color: const Color(0xFF6366F1),
        ),
        const SizedBox(height: 12),
        _buildProgressBarItem(
          label: 'Municipalidad Distrital de Yanacancha',
          count: yanacancha,
          percentage: yanacancha / total,
          color: const Color(0xFF0288D1),
        ),
        const SizedBox(height: 12),
        _buildProgressBarItem(
          label: 'Municipalidad Distrital de Simón Bolívar',
          count: simon,
          percentage: simon / total,
          color: const Color(0xFF8B5CF6),
        ),
      ],
    );
  }

  Widget _buildProgressBarItem({
    required String label,
    required int count,
    required double percentage,
    required Color color,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Expanded(
              child: Text(
                label,
                style: GoogleFonts.outfit(fontSize: 11, fontWeight: FontWeight.bold, color: const Color(0xFF475569)),
                overflow: TextOverflow.ellipsis,
              ),
            ),
            Text(
              '$count casos (${(percentage * 100).toStringAsFixed(0)}%)',
              style: GoogleFonts.outfit(fontSize: 11, fontWeight: FontWeight.bold, color: const Color(0xFF0F172A)),
            ),
          ],
        ),
        const SizedBox(height: 4),
        ClipRRect(
          borderRadius: BorderRadius.circular(4),
          child: LinearProgressIndicator(
            value: percentage,
            backgroundColor: const Color(0xFFE2E8F0),
            valueColor: AlwaysStoppedAnimation<Color>(color),
            minHeight: 8,
          ),
        ),
      ],
    );
  }

  // --- Diálogos de Gestión ---

  // Diálogo: Bitácora de Historial / Timeline
  void _showHistoryTimelineDialog(BuildContext context, EcoAlert alert) {
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

  // Diálogo: Evidencia Antes y Después
  void _showComparisonDialog(BuildContext context, EcoAlert alert) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
        title: Row(
          children: [
            const Icon(Icons.compare_rounded, color: Color(0xFF10B981)),
            const SizedBox(width: 8),
            Text(
              'Evidencia Antes y Después',
              style: GoogleFonts.outfit(fontWeight: FontWeight.bold, fontSize: 16),
            ),
          ],
        ),
        content: SizedBox(
          width: 500,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                'Denuncia solucionada: ${alert.title}',
                style: GoogleFonts.outfit(fontWeight: FontWeight.bold, fontSize: 13, color: const Color(0xFF1E293B)),
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  Expanded(
                    child: Column(
                      children: [
                        Text(
                          'Antes (Denuncia)',
                          style: GoogleFonts.outfit(fontSize: 11, fontWeight: FontWeight.bold, color: const Color(0xFF64748B)),
                        ),
                        const SizedBox(height: 6),
                        ClipRRect(
                          borderRadius: BorderRadius.circular(12),
                          child: Container(
                            height: 150,
                            color: const Color(0xFFF1F5F9),
                            width: double.infinity,
                            child: alert.imageUrl != null
                                ? Image.network(
                                    alert.imageUrl!,
                                    fit: BoxFit.cover,
                                    errorBuilder: (c, e, s) => const Icon(Icons.broken_image, size: 24, color: Color(0xFF94A3B8)),
                                  )
                                : const Icon(Icons.image_not_supported_rounded, size: 24, color: Color(0xFF94A3B8)),
                          ),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: Column(
                      children: [
                        Text(
                          'Después (Solución)',
                          style: GoogleFonts.outfit(fontSize: 11, fontWeight: FontWeight.bold, color: const Color(0xFF10B981)),
                        ),
                        const SizedBox(height: 6),
                        ClipRRect(
                          borderRadius: BorderRadius.circular(12),
                          child: Container(
                            height: 150,
                            color: const Color(0xFFD1FAE5),
                            width: double.infinity,
                            child: alert.resolutionImageUrl != null && alert.resolutionImageUrl!.isNotEmpty
                                ? Image.network(
                                    alert.resolutionImageUrl!,
                                    fit: BoxFit.cover,
                                    errorBuilder: (c, e, s) => const Icon(Icons.broken_image, size: 24, color: Color(0xFF059669)),
                                  )
                                : const Icon(Icons.image_not_supported_rounded, size: 24, color: Color(0xFF059669)),
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
              if (alert.resolutionComment != null && alert.resolutionComment!.isNotEmpty) ...[
                const SizedBox(height: 16),
                Container(
                  width: double.infinity,
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: const Color(0xFFECFDF5),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: const Color(0xFFA7F3D0)),
                  ),
                  child: Text(
                    alert.resolutionComment!,
                    style: GoogleFonts.outfit(fontSize: 12, color: const Color(0xFF047857), height: 1.4),
                  ),
                ),
              ],
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(
              'Cerrar',
              style: GoogleFonts.outfit(fontWeight: FontWeight.bold, color: const Color(0xFF10B981)),
            ),
          ),
        ],
      ),
    );
  }

  // --- Diálogos de Gestión ---

  // Diálogo: Resolver Alerta
  void _showResolveDialog(BuildContext context, EcoAlert alert, AppState appState) {
    final commentController = TextEditingController();
    String? selectedFileName;
    Uint8List? selectedFileBytes;
    bool isUploading = false;

    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
          title: Row(
            children: [
              const Icon(Icons.check_circle_rounded, color: Color(0xFF10B981)),
              const SizedBox(width: 8),
              Text(
                'Registrar Resolución',
                style: GoogleFonts.outfit(fontWeight: FontWeight.bold),
              ),
            ],
          ),
          content: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Registra las acciones y adjunta foto de evidencia de los trabajos realizados por la municipalidad.',
                  style: GoogleFonts.outfit(fontSize: 13, color: const Color(0xFF475569)),
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: commentController,
                  maxLines: 3,
                  style: GoogleFonts.outfit(fontSize: 13),
                  decoration: InputDecoration(
                    labelText: 'Detalles de los trabajos',
                    labelStyle: GoogleFonts.outfit(fontSize: 12),
                    border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
                    alignLabelWithHint: true,
                  ),
                ),
                const SizedBox(height: 16),
                Text(
                  'IMAGEN DE EVIDENCIA (OBLIGATORIA)',
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
                    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                    decoration: BoxDecoration(
                      color: const Color(0xFFF1F5F9),
                      borderRadius: BorderRadius.circular(8),
                      border: Border.all(color: const Color(0xFFE2E8F0)),
                    ),
                    child: Row(
                      children: [
                        const Icon(Icons.image_outlined, color: Color(0xFF10B981), size: 14),
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
                          padding: EdgeInsets.zero,
                          constraints: const BoxConstraints(),
                          onPressed: () {
                            setDialogState(() {
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
                              setDialogState(() {
                                selectedFileName = result.files.first.name;
                                selectedFileBytes = result.files.first.bytes;
                              });
                            }
                          },
                    icon: const Icon(Icons.camera_alt_rounded, size: 14),
                    label: Text(
                      selectedFileName == null ? 'Seleccionar Foto Evidencia' : 'Cambiar Foto',
                      style: GoogleFonts.outfit(fontSize: 12, fontWeight: FontWeight.bold),
                    ),
                    style: OutlinedButton.styleFrom(
                      side: const BorderSide(color: Color(0xFF10B981)),
                      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                    ),
                  ),
                ),
              ],
            ),
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
                      setDialogState(() {
                        isUploading = true;
                      });

                      // Subir imagen a Django
                      final imageUrl = await appState.uploadImage(selectedFileBytes!, selectedFileName!);
                      
                      final success = await appState.resolveAlert(
                        alert.id,
                        commentController.text.trim(),
                        imageUrl ?? '',
                      );

                      if (context.mounted) {
                        Navigator.pop(context);
                        EcoNotification.show(
                          context,
                          title: success ? 'Incidente Resuelto' : 'Error al Resolver',
                          message: success ? 'Caso marcado como Solucionado con éxito.' : 'Fallo al registrar la resolución.',
                          type: success ? EcoNotificationType.success : EcoNotificationType.error,
                        );
                      }
                    },
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF10B981),
                foregroundColor: Colors.white,
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
              ),
              child: isUploading
                  ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, valueColor: AlwaysStoppedAnimation<Color>(Colors.white)))
                  : Text('Registrar Solución', style: GoogleFonts.outfit(fontWeight: FontWeight.bold, color: Colors.white)),
            ),
          ],
        ),
      ),
    );
  }

  // Diálogo: Reenviar (Transferir) Alerta
  void _showTransferDialog(BuildContext context, EcoAlert alert, AppState appState) {
    _selectedTransferDistrict = alert.district;
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
                'MUNICIPALIDAD RESPONSABLE ACTUAL:',
                style: GoogleFonts.outfit(fontSize: 10, fontWeight: FontWeight.bold, color: const Color(0xFF64748B)),
              ),
              const SizedBox(height: 4),
              _buildMunicipalityBadge(alert.district),
              const SizedBox(height: 16),
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
                    value: _selectedTransferDistrict,
                    isExpanded: true,
                    onChanged: (newVal) {
                      if (newVal != null) {
                        setDialogState(() {
                          _selectedTransferDistrict = newVal;
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
              onPressed: (isSaving || _selectedTransferDistrict == alert.district)
                  ? null
                  : () async {
                      setDialogState(() {
                        isSaving = true;
                      });

                      final success = await appState.transferAlert(alert.id, _selectedTransferDistrict);

                      if (context.mounted) {
                        Navigator.pop(context);
                        EcoNotification.show(
                          context,
                          title: success ? 'Caso Reenviado' : 'Error de Reenvío',
                          message: success ? 'Caso reenviado con éxito.' : 'Fallo al reenviar el caso.',
                          type: success ? EcoNotificationType.success : EcoNotificationType.error,
                        );
                      }
                    },
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF0288D1),
                foregroundColor: Colors.white,
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
              ),
              child: isSaving
                  ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, valueColor: AlwaysStoppedAnimation<Color>(Colors.white)))
                  : Text('Reenviar Caso', style: GoogleFonts.outfit(fontWeight: FontWeight.bold, color: Colors.white)),
            ),
          ],
        ),
      ),
    );
  }

  // Diálogo: Ocultar / Descartar Alerta
  void _showDismissDialog(BuildContext context, EcoAlert alert, AppState appState) {
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
                'Utiliza esta opción si el caso fue enviado por error, es spam, duplicado o contiene información falsa. El reporte se ocultará de los mapas públicos.',
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
                        EcoNotification.show(
                          context,
                          title: success ? 'Reporte Descartado' : 'Error al Descartar',
                          message: success ? 'Reporte descartado y ocultado.' : 'Fallo al descartar el reporte.',
                          type: success ? EcoNotificationType.info : EcoNotificationType.error,
                        );
                      }
                    },
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFFEF4444),
                foregroundColor: Colors.white,
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
              ),
              child: isSaving
                  ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, valueColor: AlwaysStoppedAnimation<Color>(Colors.white)))
                  : Text('Confirmar Descarte', style: GoogleFonts.outfit(fontWeight: FontWeight.bold, color: Colors.white)),
            ),
          ],
        ),
      ),
    );
  }
}
