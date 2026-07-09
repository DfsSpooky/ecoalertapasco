import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/eco_alert.dart';
import '../state/app_state.dart';

class MonthlyDonutChart extends StatelessWidget {
  const MonthlyDonutChart({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<AppState>(
      builder: (context, appState, child) {
        final alerts = appState.filteredAlerts;

        // Contar por categoría
        int mineriaCount = 0;
        int basuraCount = 0;
        int aguaCount = 0;
        int aireCount = 0;

        for (var alert in alerts) {
          switch (alert.category) {
            case EcoCategory.mineria:
              mineriaCount++;
              break;
            case EcoCategory.basura:
              basuraCount++;
              break;
            case EcoCategory.agua:
              aguaCount++;
              break;
            case EcoCategory.aire:
              aireCount++;
              break;
          }
        }

        final total = alerts.length;

        // Configuración de secciones
        List<PieChartSectionData> sections = [];
        
        if (total == 0) {
          sections = [
            PieChartSectionData(
              color: const Color(0xFFE2E8F0), // Slate 200
              value: 1,
              title: '0%',
              radius: 20,
              titleStyle: const TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.bold,
                color: Color(0xFF94A3B8),
              ),
            )
          ];
        } else {
          if (mineriaCount > 0) {
            sections.add(
              PieChartSectionData(
                color: const Color(0xFFE65100),
                value: mineriaCount.toDouble(),
                title: '${((mineriaCount / total) * 100).toStringAsFixed(0)}%',
                radius: 22,
                titleStyle: const TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.bold,
                  color: Colors.white,
                ),
              ),
            );
          }
          if (basuraCount > 0) {
            sections.add(
              PieChartSectionData(
                color: const Color(0xFF607D8B),
                value: basuraCount.toDouble(),
                title: '${((basuraCount / total) * 100).toStringAsFixed(0)}%',
                radius: 22,
                titleStyle: const TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.bold,
                  color: Colors.white,
                ),
              ),
            );
          }
          if (aguaCount > 0) {
            sections.add(
              PieChartSectionData(
                color: const Color(0xFF0288D1),
                value: aguaCount.toDouble(),
                title: '${((aguaCount / total) * 100).toStringAsFixed(0)}%',
                radius: 22,
                titleStyle: const TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.bold,
                  color: Colors.white,
                ),
              ),
            );
          }
          if (aireCount > 0) {
            sections.add(
              PieChartSectionData(
                color: const Color(0xFF00897B),
                value: aireCount.toDouble(),
                title: '${((aireCount / total) * 100).toStringAsFixed(0)}%',
                radius: 22,
                titleStyle: const TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.bold,
                  color: Colors.white,
                ),
              ),
            );
          }
        }

        return SizedBox(
          height: 140,
          child: Row(
            children: [
              Expanded(
                flex: 3,
                child: PieChart(
                  PieChartData(
                    sectionsSpace: 3,
                    centerSpaceRadius: 35,
                    sections: sections,
                  ),
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                flex: 4,
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    _buildLegendItem('Minería', const Color(0xFFE65100), mineriaCount),
                    _buildLegendItem('Basura', const Color(0xFF607D8B), basuraCount),
                    _buildLegendItem('Agua', const Color(0xFF0288D1), aguaCount),
                    _buildLegendItem('Aire', const Color(0xFF00897B), aireCount),
                  ],
                ),
              )
            ],
          ),
        );
      },
    );
  }

  Widget _buildLegendItem(String title, Color color, int count) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2.0),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 10,
            height: 10,
            decoration: BoxDecoration(
              color: color,
              shape: BoxShape.circle,
            ),
          ),
          const SizedBox(width: 6),
          Expanded(
            child: Text(
              title,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(color: Color(0xFF475569), fontSize: 11), // Slate 600
            ),
          ),
          Text(
            '$count',
            style: const TextStyle(
              color: Color(0xFF0F172A), // Slate 900
              fontSize: 11,
              fontWeight: FontWeight.bold,
            ),
          ),
        ],
      ),
    );
  }
}

class SeverityBarChart extends StatelessWidget {
  const SeverityBarChart({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<AppState>(
      builder: (context, appState, child) {
        final alerts = appState.filteredAlerts;

        // Contar por severidad
        int criticoCount = 0;
        int medioCount = 0;
        int bajoCount = 0;

        for (var alert in alerts) {
          switch (alert.severity) {
            case EcoSeverity.critico:
              criticoCount++;
              break;
            case EcoSeverity.medio:
              medioCount++;
              break;
            case EcoSeverity.bajo:
              bajoCount++;
              break;
          }
        }

        final double maxVal = [criticoCount, medioCount, bajoCount]
            .map((e) => e.toDouble())
            .reduce((curr, next) => curr > next ? curr : next);
        
        final double limitY = maxVal == 0 ? 5 : (maxVal + 2);

        return SizedBox(
          height: 140,
          child: BarChart(
            BarChartData(
              alignment: BarChartAlignment.spaceAround,
              maxY: limitY,
              barTouchData: BarTouchData(
                enabled: true,
                touchTooltipData: BarTouchTooltipData(
                  getTooltipColor: (_) => const Color(0xFF0F172A).withOpacity(0.95),
                  tooltipPadding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  tooltipMargin: 4,
                  getTooltipItem: (group, groupIndex, rod, rodIndex) {
                    String name = '';
                    switch (group.x) {
                      case 0:
                        name = 'Crítico';
                        break;
                      case 1:
                        name = 'Medio';
                        break;
                      case 2:
                        name = 'Bajo';
                        break;
                    }
                    return BarTooltipItem(
                      '$name\n',
                      const TextStyle(
                        color: Colors.white,
                        fontWeight: FontWeight.bold,
                        fontSize: 11,
                      ),
                      children: <TextSpan>[
                        TextSpan(
                          text: '${rod.toY.toInt()} reportes',
                          style: TextStyle(
                            color: rod.color,
                            fontSize: 11,
                            fontWeight: FontWeight.w500,
                          ),
                        ),
                      ],
                    );
                  },
                ),
              ),
              titlesData: FlTitlesData(
                show: true,
                bottomTitles: AxisTitles(
                  sideTitles: SideTitles(
                    showTitles: true,
                    getTitlesWidget: (double value, TitleMeta meta) {
                      const style = TextStyle(
                        color: Color(0xFF64748B), // Slate 500
                        fontWeight: FontWeight.bold,
                        fontSize: 10,
                      );
                      Widget text;
                      switch (value.toInt()) {
                        case 0:
                          text = const Text('CRÍTICO', style: style);
                          break;
                        case 1:
                          text = const Text('MEDIO', style: style);
                          break;
                        case 2:
                          text = const Text('BAJO', style: style);
                          break;
                        default:
                          text = const Text('', style: style);
                          break;
                      }
                      return SideTitleWidget(
                        meta: meta,
                        space: 4,
                        child: text,
                      );
                    },
                  ),
                ),
                leftTitles: const AxisTitles(
                  sideTitles: SideTitles(showTitles: false),
                ),
                topTitles: const AxisTitles(
                  sideTitles: SideTitles(showTitles: false),
                ),
                rightTitles: const AxisTitles(
                  sideTitles: SideTitles(showTitles: false),
                ),
              ),
              gridData: const FlGridData(show: false),
              borderData: FlBorderData(show: false),
              barGroups: [
                BarChartGroupData(
                  x: 0,
                  barRods: [
                    BarChartRodData(
                      toY: criticoCount.toDouble(),
                      color: const Color(0xFFD32F2F),
                      width: 20,
                      borderRadius: const BorderRadius.vertical(top: Radius.circular(6)),
                    )
                  ],
                ),
                BarChartGroupData(
                  x: 1,
                  barRods: [
                    BarChartRodData(
                      toY: medioCount.toDouble(),
                      color: const Color(0xFFFFA000),
                      width: 20,
                      borderRadius: const BorderRadius.vertical(top: Radius.circular(6)),
                    )
                  ],
                ),
                BarChartGroupData(
                  x: 2,
                  barRods: [
                    BarChartRodData(
                      toY: bajoCount.toDouble(),
                      color: const Color(0xFF388E3C),
                      width: 20,
                      borderRadius: const BorderRadius.vertical(top: Radius.circular(6)),
                    )
                  ],
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
