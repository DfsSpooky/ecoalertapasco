import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:provider/provider.dart';
import '../state/app_state.dart';
import '../models/eco_alert.dart';

class TransparencyPortalView extends StatelessWidget {
  const TransparencyPortalView({super.key});

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);
    final allAlerts = appState.allAlerts;
    final screenWidth = MediaQuery.of(context).size.width;
    final isMobileOrTablet = screenWidth < 1050;

    // Filtrar casos resueltos
    final solvedAlerts = allAlerts.where((a) => a.status == 'solved').toList();

    // Calcular estadísticas de logros
    final totalSolved = solvedAlerts.length;

    // Calcular tiempo promedio de solución
    String avgResolutionTime = 'N/A';
    if (solvedAlerts.isNotEmpty) {
      double totalHours = 0;
      int validCases = 0;
      for (final alert in solvedAlerts) {
        if (alert.resolvedAt != null) {
          totalHours += alert.resolvedAt!.difference(alert.createdAt).inHours;
          validCases++;
        }
      }
      if (validCases > 0) {
        double avgHours = totalHours / validCases;
        if (avgHours < 24) {
          avgResolutionTime = '${avgHours.toStringAsFixed(1)} horas';
        } else {
          double avgDays = avgHours / 24;
          avgResolutionTime = '${avgDays.toStringAsFixed(1)} días';
        }
      }
    }

    // Calcular categoría con más soluciones
    String topCategoryText = 'Ninguna';
    if (solvedAlerts.isNotEmpty) {
      final counts = <EcoCategory, int>{};
      for (final alert in solvedAlerts) {
        counts[alert.category] = (counts[alert.category] ?? 0) + 1;
      }
      var maxVal = 0;
      EcoCategory? topCat;
      counts.forEach((cat, val) {
        if (val > maxVal) {
          maxVal = val;
          topCat = cat;
        }
      });
      if (topCat != null) {
        switch (topCat!) {
          case EcoCategory.mineria:
            topCategoryText = 'Minería ⛏️';
            break;
          case EcoCategory.basura:
            topCategoryText = 'Residuos Sólidos 🗑️';
            break;
          case EcoCategory.agua:
            topCategoryText = 'Recursos Hídricos 💧';
            break;
          case EcoCategory.aire:
            topCategoryText = 'Calidad del Aire 🌬️';
            break;
        }
      }
    }

    return Scaffold(
      backgroundColor: const Color(0xFFF8FAFC), // Slate 50
      appBar: AppBar(
        backgroundColor: Colors.white,
        elevation: 0,
        centerTitle: false,
        title: Row(
          children: [
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: const Color(0xFF10B981).withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(12),
              ),
              child: const Icon(Icons.emoji_events_rounded, color: Color(0xFF10B981), size: 28),
            ),
            const SizedBox(width: 12),
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Portal de Transparencia Ciudadana',
                  style: GoogleFonts.outfit(
                    fontSize: isMobileOrTablet ? 15 : 20,
                    fontWeight: FontWeight.bold,
                    color: const Color(0xFF0F172A),
                  ),
                ),
                Text(
                  'Monitoreo de logros e impacto ambiental',
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
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8),
            child: ElevatedButton.icon(
              onPressed: () {
                appState.showTransparencyPortal = false;
              },
              icon: const Icon(Icons.map_rounded, size: 18, color: Colors.white),
              label: Text(
                'Volver al Mapa',
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
            // 1. Banner de Presentación
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(24),
              decoration: BoxDecoration(
                gradient: const LinearGradient(
                  colors: [Color(0xFF059669), Color(0xFF10B981)],
                  begin: Alignment.topLeft,
                  end: Alignment.bottomRight,
                ),
                borderRadius: BorderRadius.circular(16),
                boxShadow: [
                  BoxShadow(
                    color: const Color(0xFF059669).withValues(alpha: 0.2),
                    blurRadius: 16,
                    offset: const Offset(0, 4),
                  )
                ],
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    '¡Hacia una provincia limpia y sustentable! 🌿',
                    style: GoogleFonts.outfit(
                      fontSize: isMobileOrTablet ? 18 : 24,
                      fontWeight: FontWeight.bold,
                      color: Colors.white,
                    ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'En este portal reportamos con absoluta transparencia las denuncias ecológicas resueltas por las municipalidades de Cerro de Pasco. Consulta la evidencia física de los trabajos realizados.',
                    style: GoogleFonts.outfit(
                      fontSize: isMobileOrTablet ? 12 : 14,
                      color: const Color(0xFFECFDF5),
                      height: 1.4,
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 24),

            // 2. Indicadores de Logro
            if (isMobileOrTablet)
              Column(
                children: [
                  _buildStatCard('Casos Resueltos', '$totalSolved', Icons.check_circle_rounded, const Color(0xFF10B981), 'Atendidos con evidencia física'),
                  const SizedBox(height: 12),
                  _buildStatCard('Tiempo de Respuesta', avgResolutionTime, Icons.timer_rounded, const Color(0xFFFFA000), 'Promedio desde el reporte'),
                  const SizedBox(height: 12),
                  _buildStatCard('Mayor Foco de Atención', topCategoryText, Icons.terrain_rounded, const Color(0xFF0288D1), 'Categoría con más soluciones'),
                ],
              )
            else
              Row(
                children: [
                  Expanded(
                    child: _buildStatCard('Casos Resueltos', '$totalSolved', Icons.check_circle_rounded, const Color(0xFF10B981), 'Atendidos con evidencia física'),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: _buildStatCard('Tiempo de Respuesta', avgResolutionTime, Icons.timer_rounded, const Color(0xFFFFA000), 'Promedio desde el reporte'),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: _buildStatCard('Mayor Foco de Atención', topCategoryText, Icons.terrain_rounded, const Color(0xFF0288D1), 'Categoría con más soluciones'),
                  ),
                ],
              ),
            const SizedBox(height: 32),

            // 3. Título de Galería
            Text(
              '🏆 Galería de Éxitos Ambientales',
              style: GoogleFonts.outfit(
                fontSize: 18,
                fontWeight: FontWeight.bold,
                color: const Color(0xFF0F172A),
              ),
            ),
            Text(
              'Comparativa física (Antes y Después) de la limpieza y mitigación en Cerro de Pasco',
              style: GoogleFonts.outfit(
                fontSize: 12,
                color: const Color(0xFF64748B),
                fontWeight: FontWeight.w500,
              ),
            ),
            const SizedBox(height: 16),

            // 4. Listado de Tarjetas Antes y Después
            solvedAlerts.isEmpty
                ? Center(
                    child: Container(
                      padding: const EdgeInsets.symmetric(vertical: 60),
                      child: Column(
                        children: [
                          const Icon(Icons.check_circle_outline_rounded, size: 56, color: Color(0xFF94A3B8)),
                          const SizedBox(height: 16),
                          Text(
                            'Aún no hay casos marcados como resueltos con evidencia.',
                            style: GoogleFonts.outfit(color: const Color(0xFF64748B), fontSize: 13),
                          ),
                        ],
                      ),
                    ),
                  )
                : ListView.builder(
                    shrinkWrap: true,
                    physics: const NeverScrollableScrollPhysics(),
                    itemCount: solvedAlerts.length,
                    itemBuilder: (context, index) {
                      final alert = solvedAlerts[index];
                      final resolvedDate = alert.resolvedAt != null
                          ? "${alert.resolvedAt!.day}/${alert.resolvedAt!.month}/${alert.resolvedAt!.year}"
                          : 'Reciente';

                      String mName = '';
                      switch (alert.district) {
                        case EcoDistrict.yanacancha:
                          mName = 'Muni Distrital de Yanacancha';
                          break;
                        case EcoDistrict.simonBolivar:
                          mName = 'Muni Distrital de Simón Bolívar';
                          break;
                        case EcoDistrict.chaupimarca:
                          mName = 'Muni Provincial de Pasco';
                          break;
                      }

                      return Container(
                        margin: const EdgeInsets.only(bottom: 20),
                        padding: const EdgeInsets.all(20),
                        decoration: BoxDecoration(
                          color: Colors.white,
                          borderRadius: BorderRadius.circular(16),
                          border: Border.all(color: const Color(0xFFE2E8F0)),
                        ),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            // Header de la Alerta Resuelta
                            Row(
                              children: [
                                _buildCategoryBadge(alert.category),
                                const SizedBox(width: 8),
                                Container(
                                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                                  decoration: BoxDecoration(
                                    color: const Color(0xFFECFDF5),
                                    borderRadius: BorderRadius.circular(8),
                                    border: Border.all(color: const Color(0xFFA7F3D0)),
                                  ),
                                  child: Row(
                                    children: [
                                      const Icon(Icons.check_circle_rounded, color: Color(0xFF059669), size: 12),
                                      const SizedBox(width: 4),
                                      Text(
                                        'RESUELTO: $resolvedDate',
                                        style: GoogleFonts.outfit(fontSize: 10, fontWeight: FontWeight.bold, color: const Color(0xFF065F46)),
                                      ),
                                    ],
                                  ),
                                ),
                                const Spacer(),
                                Text(
                                  mName,
                                  style: GoogleFonts.outfit(fontSize: 11, fontWeight: FontWeight.bold, color: const Color(0xFF64748B)),
                                ),
                              ],
                            ),
                            const SizedBox(height: 12),
                            Text(
                              alert.title,
                              style: GoogleFonts.outfit(fontSize: 15, fontWeight: FontWeight.bold, color: const Color(0xFF0F172A)),
                            ),
                            const SizedBox(height: 4),
                            Text(
                              alert.description,
                              style: GoogleFonts.outfit(fontSize: 13, color: const Color(0xFF475569)),
                            ),
                            const SizedBox(height: 16),

                            // Comparador visual lado a lado
                            if (isMobileOrTablet)
                              Column(
                                children: [
                                  _buildPhotoBox('ANTES (Denuncia del Ciudadano)', alert.imageUrl, const Color(0xFFF1F5F9), const Color(0xFF94A3B8)),
                                  const SizedBox(height: 12),
                                  _buildPhotoBox('DESPUÉS (Trabajos de la Municipalidad)', alert.resolutionImageUrl, const Color(0xFFE6FDF4), const Color(0xFF10B981)),
                                ],
                              )
                            else
                              Row(
                                children: [
                                  Expanded(
                                    child: _buildPhotoBox('ANTES (Denuncia del Ciudadano)', alert.imageUrl, const Color(0xFFF1F5F9), const Color(0xFF94A3B8)),
                                  ),
                                  const SizedBox(width: 16),
                                  Expanded(
                                    child: _buildPhotoBox('DESPUÉS (Trabajos de la Municipalidad)', alert.resolutionImageUrl, const Color(0xFFE6FDF4), const Color(0xFF10B981)),
                                  ),
                                ],
                              ),
                            
                            // Comentario de resolución
                            if (alert.resolutionComment != null && alert.resolutionComment!.isNotEmpty) ...[
                              const SizedBox(height: 16),
                              Container(
                                width: double.infinity,
                                padding: const EdgeInsets.all(16),
                                decoration: BoxDecoration(
                                  color: const Color(0xFFF0FDF4),
                                  borderRadius: BorderRadius.circular(12),
                                  border: Border.all(color: const Color(0xFFDCFCE7)),
                                ),
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    Text(
                                      'TRABAJO REALIZADO:',
                                      style: GoogleFonts.outfit(fontSize: 10, fontWeight: FontWeight.bold, color: const Color(0xFF166534)),
                                    ),
                                    const SizedBox(height: 4),
                                    Text(
                                      alert.resolutionComment!,
                                      style: GoogleFonts.outfit(fontSize: 12, color: const Color(0xFF15803D), height: 1.4),
                                    ),
                                  ],
                                ),
                              ),
                            ],
                          ],
                        ),
                      );
                    },
                  ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatCard(String title, String value, IconData icon, Color color, String subtitle) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: const Color(0xFFE2E8F0)),
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: color.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Icon(icon, color: color, size: 24),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: GoogleFonts.outfit(fontSize: 12, color: const Color(0xFF64748B), fontWeight: FontWeight.w500),
                ),
                const SizedBox(height: 4),
                Text(
                  value,
                  style: GoogleFonts.outfit(fontSize: 18, color: const Color(0xFF0F172A), fontWeight: FontWeight.bold),
                ),
                const SizedBox(height: 2),
                Text(
                  subtitle,
                  style: GoogleFonts.outfit(fontSize: 10, color: const Color(0xFF94A3B8)),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildPhotoBox(String title, String? imageUrl, Color bgColor, Color textColor) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: GoogleFonts.outfit(fontSize: 10, fontWeight: FontWeight.bold, color: textColor, letterSpacing: 0.5),
        ),
        const SizedBox(height: 6),
        ClipRRect(
          borderRadius: BorderRadius.circular(12),
          child: Container(
            height: 180,
            color: bgColor,
            width: double.infinity,
            child: imageUrl != null && imageUrl.isNotEmpty
                ? Image.network(
                    imageUrl,
                    fit: BoxFit.cover,
                    errorBuilder: (c, e, s) => Center(
                      child: Icon(Icons.broken_image_rounded, color: textColor.withValues(alpha: 0.5), size: 32),
                    ),
                  )
                : Center(
                    child: Icon(Icons.image_not_supported_rounded, color: textColor.withValues(alpha: 0.5), size: 32),
                  ),
          ),
        ),
      ],
    );
  }

  Widget _buildCategoryBadge(EcoCategory category) {
    String text;
    IconData icon;
    Color color;

    switch (category) {
      case EcoCategory.mineria:
        text = 'Minería';
        icon = Icons.terrain_rounded;
        color = const Color(0xFF9A3412); // Naranja oscuro
        break;
      case EcoCategory.basura:
        text = 'Basura';
        icon = Icons.delete_outline_rounded;
        color = const Color(0xFF334155); // Slate
        break;
      case EcoCategory.agua:
        text = 'Agua';
        icon = Icons.water_drop_rounded;
        color = const Color(0xFF0369A1); // Azul
        break;
      case EcoCategory.aire:
        text = 'Aire';
        icon = Icons.air_rounded;
        color = const Color(0xFF047857); // Esmeralda
        break;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.1),
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
              fontSize: 10,
              fontWeight: FontWeight.bold,
            ),
          ),
        ],
      ),
    );
  }


}
