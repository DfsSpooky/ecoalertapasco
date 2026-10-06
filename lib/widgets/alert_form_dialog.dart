import 'dart:async';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:provider/provider.dart';
import '../models/eco_alert.dart';
import '../state/app_state.dart';
import 'package:file_picker/file_picker.dart';

class AlertFormDialog extends StatefulWidget {
  const AlertFormDialog({super.key});

  @override
  State<AlertFormDialog> createState() => _AlertFormDialogState();
}

class _AlertFormDialogState extends State<AlertFormDialog> {
  final _formKey = GlobalKey<FormState>();
  final _titleController = TextEditingController();
  final _descController = TextEditingController();
  final _addressController = TextEditingController();
  
  EcoCategory _selectedCategory = EcoCategory.mineria;
  EcoSeverity _selectedSeverity = EcoSeverity.medio;
  
  final List<Map<String, dynamic>> _photos = [];
  bool _isUploadingPhoto = false;
  bool _isSubmitting = false;
  Timer? _uiTicker;
  bool _isAnalyzingAI = false;
  Map<String, dynamic>? _aiAnalysisResult;

  @override
  void initState() {
    super.initState();
    final appState = Provider.of<AppState>(context, listen: false);
    if (appState.tempLatitude != null && appState.tempLongitude != null) {
      _addressController.text = appState.currentGeocodedAddress == 'Alineando mira...'
          ? appState.reverseGeocode(appState.tempLatitude!, appState.tempLongitude!)
          : appState.currentGeocodedAddress;
    }

    // Si hay un cooldown activo, refrescar el timer por segundo
    if (appState.cooldownSecondsRemaining > 0) {
      _uiTicker = Timer.periodic(const Duration(seconds: 1), (timer) {
        if (!mounted) {
          timer.cancel();
          return;
        }
        if (appState.cooldownSecondsRemaining <= 0) {
          timer.cancel();
        }
        setState(() {});
      });
    }
  }

  @override
  void dispose() {
    _uiTicker?.cancel();
    _titleController.dispose();
    _descController.dispose();
    _addressController.dispose();
    super.dispose();
  }

  // Carga de fotografías (máximo 2 para evitar saturación del servidor)
  void _pickAndUploadPhoto() async {
    if (_photos.length >= 2) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            'Límite alcanzado: máximo 2 fotografías por reporte ambiental.',
            style: GoogleFonts.outfit(),
          ),
          backgroundColor: const Color(0xFFF59E0B),
        ),
      );
      return;
    }

    final result = await FilePicker.platform.pickFiles(type: FileType.image);
    if (!mounted) return;
    if (result != null && result.files.first.bytes != null) {
      final fileName = result.files.first.name;
      final fileBytes = result.files.first.bytes!;

      setState(() {
        _isUploadingPhoto = true;
      });

      final appState = Provider.of<AppState>(context, listen: false);
      final url = await appState.uploadImage(fileBytes, fileName);

      if (!mounted) return;

      if (url == null) {
        setState(() {
          _isUploadingPhoto = false;
        });
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Error al subir la fotografía al servidor.', style: GoogleFonts.outfit()),
            backgroundColor: const Color(0xFFEF4444),
          ),
        );
        return;
      }

      setState(() {
        _isUploadingPhoto = false;
        _photos.add({
          'name': fileName,
          'bytes': fileBytes,
          'url': url,
        });
      });

      // Flujo Inteligente: Si es la primera foto y Gemini está activado en las configuraciones,
      // se procesa automáticamente en el momento para rellenar los campos.
      if (_photos.length == 1 && appState.isGeminiEnabled) {
        _autoAnalyzeWithGemini(fileBytes, url, fileName);
      }
    }
  }

  Future<void> _autoAnalyzeWithGemini(List<int> bytes, String url, String fileName) async {
    final appState = Provider.of<AppState>(context, listen: false);
    setState(() {
      _isAnalyzingAI = true;
      _aiAnalysisResult = null;
    });

    try {
      final result = await appState.analyzeEvidenceWithGemini(
        imageUrl: url,
        fileName: fileName,
      );

      if (!mounted) return;

      setState(() {
        _isAnalyzingAI = false;
        _aiAnalysisResult = result;

        final bool isMinorLitter = result['is_minor_litter'] == true ||
            result['rejection_reason'] == 'minor_litter' ||
            result['waste_scale'] == 'basura_menor';

        if (isMinorLitter) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Row(
                children: [
                  const Icon(Icons.local_shipping_rounded, color: Colors.white, size: 20),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      '🚛 El camión recolector ya va a pasar por esa zona a recoger esta basura.',
                      style: GoogleFonts.outfit(fontWeight: FontWeight.bold),
                    ),
                  ),
                ],
              ),
              backgroundColor: const Color(0xFFD97706),
              duration: const Duration(seconds: 5),
            ),
          );
        } else if (result['is_environmental_issue'] == false) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text(
                'Filtro IA: La fotografía no muestra contaminación ambiental en Cerro de Pasco.',
                style: GoogleFonts.outfit(),
              ),
              backgroundColor: const Color(0xFFEF4444),
              duration: const Duration(seconds: 4),
            ),
          );
        } else {
          // Autocompletar título sugerido
          if (result['suggested_title'] != null && result['suggested_title'].toString().isNotEmpty) {
            _titleController.text = result['suggested_title'].toString();
          }
          // Autocompletar descripción sugerida
          if (result['suggested_description'] != null && result['suggested_description'].toString().isNotEmpty) {
            _descController.text = result['suggested_description'].toString();
          }

          // Autocompletar categoría detectada (con soporte para sinónimos técnicos)
          final cat = (result['category'] ?? '').toString().toLowerCase();
          if (cat.contains('miner') || cat.contains('relave') || cat.contains('tajo')) {
            _selectedCategory = EcoCategory.mineria;
          } else if (cat.contains('basur') || cat.contains('desmonte') || cat.contains('residuo') || cat.contains('plastic') || cat.contains('escomb')) {
            _selectedCategory = EcoCategory.basura;
          } else if (cat.contains('agua') || cat.contains('rio') || cat.contains('lagun') || cat.contains('desag')) {
            _selectedCategory = EcoCategory.agua;
          } else if (cat.contains('aire') || cat.contains('humo') || cat.contains('quem')) {
            _selectedCategory = EcoCategory.aire;
          }

          // Autocompletar nivel de severidad
          final sev = result['severity']?.toString().toLowerCase();
          if (sev == 'critico') {
            _selectedSeverity = EcoSeverity.critico;
          } else if (sev == 'medio') {
            _selectedSeverity = EcoSeverity.medio;
          } else if (sev == 'bajo') {
            _selectedSeverity = EcoSeverity.bajo;
          }

          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text(
                '✨ ¡Analizado con éxito por Gemini! Título, categoría y severidad autocompletados.',
                style: GoogleFonts.outfit(),
              ),
              backgroundColor: const Color(0xFF0288D1),
              duration: const Duration(seconds: 4),
            ),
          );
        }
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _isAnalyzingAI = false;
      });
      String errorMsg = e.toString();
      if (errorMsg.startsWith('Exception: ')) {
        errorMsg = errorMsg.substring(11);
      }
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            errorMsg.isNotEmpty && !errorMsg.contains('XMLHttpRequest')
                ? errorMsg
                : 'Aviso de IA: No se pudo auto-analizar la foto. Puedes completar los campos manualmente.',
            style: GoogleFonts.outfit(),
          ),
          backgroundColor: const Color(0xFFD97706),
          duration: const Duration(seconds: 4),
        ),
      );
    }
  }

  void _removePhoto(int index) {
    setState(() {
      _photos.removeAt(index);
      // Si eliminamos la primera foto (la analizada), limpiamos el resultado de IA
      if (_photos.isEmpty) {
        _aiAnalysisResult = null;
      }
    });
  }

  Future<void> _retryAnalysisWithGemini() async {
    if (_photos.isEmpty) return;
    final first = _photos.first;
    await _autoAnalyzeWithGemini(
      first['bytes'] as List<int>,
      first['url'] as String,
      first['name'] as String,
    );
  }

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);
    final cooldownRemaining = appState.cooldownSecondsRemaining;
    final nearbyDuplicate = (appState.tempLatitude != null && appState.tempLongitude != null)
        ? appState.findNearbyActiveAlert(appState.tempLatitude!, appState.tempLongitude!, _selectedCategory)
        : null;

    return Dialog(
      backgroundColor: Colors.transparent,
      insetPadding: const EdgeInsets.symmetric(horizontal: 24, vertical: 24),
      child: Center(
        child: SingleChildScrollView(
          child: Container(
            width: 480,
            decoration: BoxDecoration(
              color: Colors.white,
              borderRadius: BorderRadius.circular(24),
              border: Border.all(color: const Color(0xFFE2E8F0), width: 2),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withValues(alpha: 0.12),
                  blurRadius: 24,
                  offset: const Offset(0, 10),
                )
              ],
            ),
            child: Form(
              key: _formKey,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  // Header
                  Padding(
                    padding: const EdgeInsets.only(top: 24, left: 24, right: 24, bottom: 8),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Expanded(
                          child: Text(
                            '📢 Reportar Alerta',
                            overflow: TextOverflow.ellipsis,
                            maxLines: 1,
                            style: GoogleFonts.outfit(
                              color: const Color(0xFF0F172A),
                              fontSize: 18,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ),
                        IconButton(
                          icon: const Icon(Icons.close_rounded, color: Color(0xFF94A3B8)),
                          onPressed: () {
                            appState.cancelReportMode();
                            Navigator.of(context).pop();
                          },
                        ),
                      ],
                    ),
                  ),
                  const Divider(color: Color(0xFFE2E8F0), height: 1),

                  // Form body
                  Padding(
                    padding: const EdgeInsets.all(24.0),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        // Banner de Cooldown Anti-Spam si está activo
                        if (cooldownRemaining > 0) ...[
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
                            decoration: BoxDecoration(
                              color: const Color(0xFFFEF3C7), // Amber 100
                              borderRadius: BorderRadius.circular(12),
                              border: Border.all(color: const Color(0xFFF59E0B), width: 1.5), // Amber 500
                            ),
                            child: Row(
                              children: [
                                const Icon(Icons.timer_outlined, color: Color(0xFFB45309), size: 20),
                                const SizedBox(width: 10),
                                Expanded(
                                  child: Text(
                                    'Enfriamiento anti-spam: Espera ${cooldownRemaining}s para registrar otro reporte.',
                                    style: GoogleFonts.outfit(
                                      color: const Color(0xFF92400E),
                                      fontSize: 12,
                                      fontWeight: FontWeight.bold,
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          ),
                          const SizedBox(height: 16),
                        ],

                        // Banner de Alerta Duplicada Cercana
                        if (nearbyDuplicate != null) ...[
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
                            decoration: BoxDecoration(
                              color: const Color(0xFFFEE2E2), // Red 100
                              borderRadius: BorderRadius.circular(12),
                              border: Border.all(color: const Color(0xFFEF4444), width: 1.5), // Red 500
                            ),
                            child: Row(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                const Icon(Icons.warning_amber_rounded, color: Color(0xFFDC2626), size: 22),
                                const SizedBox(width: 10),
                                Expanded(
                                  child: Column(
                                    crossAxisAlignment: CrossAxisAlignment.start,
                                    children: [
                                      Text(
                                        'Alerta similar ya registrada cerca',
                                        style: GoogleFonts.outfit(
                                          color: const Color(0xFF991B1B),
                                          fontSize: 12.5,
                                          fontWeight: FontWeight.bold,
                                        ),
                                      ),
                                      const SizedBox(height: 2),
                                      Text(
                                        'Existe una alerta activa en esta zona ("${nearbyDuplicate.title}"). Evitemos duplicar incidentes.',
                                        style: GoogleFonts.outfit(
                                          color: const Color(0xFF7F1D1D),
                                          fontSize: 11.5,
                                        ),
                                      ),
                                    ],
                                  ),
                                ),
                              ],
                            ),
                          ),
                          const SizedBox(height: 16),
                        ],

                        // 1. Evidencia Fotográfica (Al inicio, procesada automáticamente si Gemini está activo)
                        _buildImageUploader(appState),
                        const SizedBox(height: 22),

                        // Coordenadas autocompletadas (Lectura)
                        Text(
                          'UBICACIÓN SELECCIONADA',
                          style: GoogleFonts.outfit(
                            fontSize: 10,
                            fontWeight: FontWeight.bold,
                            color: const Color(0xFF64748B),
                            letterSpacing: 1.0,
                          ),
                        ),
                        const SizedBox(height: 8),
                        Container(
                          padding: const EdgeInsets.all(12),
                          decoration: BoxDecoration(
                            color: const Color(0xFFF1F5F9),
                            borderRadius: BorderRadius.circular(12),
                            border: Border.all(color: const Color(0xFFE2E8F0)),
                          ),
                          child: Row(
                            children: [
                              const Icon(Icons.my_location_rounded, color: Color(0xFF0288D1), size: 18),
                              const SizedBox(width: 10),
                              Expanded(
                                child: Text(
                                  appState.currentGeocodedAddress == 'Alineando mira...'
                                      ? appState.reverseGeocode(appState.tempLatitude!, appState.tempLongitude!)
                                      : appState.currentGeocodedAddress,
                                  style: GoogleFonts.outfit(
                                    color: const Color(0xFF0F172A),
                                    fontSize: 13,
                                    fontWeight: FontWeight.bold,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                        const SizedBox(height: 8),
                        SizedBox(
                          width: double.infinity,
                          child: TextButton.icon(
                            onPressed: () async {
                              final latLng = await appState.getUserLatLng();
                              if (latLng != null) {
                                appState.confirmReportLocation(latLng.latitude, latLng.longitude);
                                
                                setState(() {
                                  _addressController.text = 'Obteniendo dirección GPS...';
                                });
                                
                                final address = await appState.fetchAddressString(latLng.latitude, latLng.longitude);
                                
                                setState(() {
                                  _addressController.text = address;
                                });
                                
                                if (context.mounted) {
                                  ScaffoldMessenger.of(context).showSnackBar(
                                    SnackBar(
                                      backgroundColor: const Color(0xFF10B981),
                                      content: Text(
                                        'Ubicación GPS fijada correctamente.',
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
                                      content: Text(
                                        'No se pudo obtener la ubicación GPS. Verifique permisos.',
                                        style: GoogleFonts.outfit(color: Colors.white),
                                      ),
                                    ),
                                  );
                                }
                              }
                            },
                            icon: const Icon(Icons.gps_fixed_rounded, size: 14, color: Color(0xFF0288D1)),
                            label: Text(
                              'Usar mi ubicación actual (GPS)',
                              style: GoogleFonts.outfit(
                                fontSize: 12,
                                fontWeight: FontWeight.bold,
                                color: const Color(0xFF0288D1),
                              ),
                            ),
                            style: TextButton.styleFrom(
                              padding: const EdgeInsets.symmetric(vertical: 8),
                              shape: RoundedRectangleBorder(
                                borderRadius: BorderRadius.circular(10),
                                side: const BorderSide(color: Color(0xFF0288D1), width: 1.2),
                              ),
                            ),
                          ),
                        ),
                        const SizedBox(height: 20),

                        // Dirección / Referencia
                        Text(
                          'DIRECCIÓN O REFERENCIA',
                          style: GoogleFonts.outfit(
                            fontSize: 10,
                            fontWeight: FontWeight.bold,
                            color: const Color(0xFF64748B),
                            letterSpacing: 1.0,
                          ),
                        ),
                        const SizedBox(height: 8),
                        TextFormField(
                          controller: _addressController,
                          style: GoogleFonts.outfit(color: const Color(0xFF0F172A), fontSize: 14),
                          decoration: _buildInputDecoration('Ej. Av. Los Incas 240, Yanacancha...'),
                          validator: (value) {
                            if (value == null || value.trim().isEmpty) {
                              return 'Por favor ingresa la dirección o referencia';
                            }
                            return null;
                          },
                        ),
                        const SizedBox(height: 20),

                        // Selección de Categoría (Iconos Cuadrados)
                        Text(
                          'CATEGORÍA DEL INCIDENTE',
                          style: GoogleFonts.outfit(
                            fontSize: 10,
                            fontWeight: FontWeight.bold,
                            color: const Color(0xFF64748B),
                            letterSpacing: 1.0,
                          ),
                        ),
                        const SizedBox(height: 8),
                        _buildCategorySelector(),
                        const SizedBox(height: 20),

                        // Selección de Severidad (3 botones)
                        Text(
                          'NIVEL DE GRAVEDAD',
                          style: GoogleFonts.outfit(
                            fontSize: 10,
                            fontWeight: FontWeight.bold,
                            color: const Color(0xFF64748B),
                            letterSpacing: 1.0,
                          ),
                        ),
                        const SizedBox(height: 8),
                        _buildSeveritySelector(),
                        const SizedBox(height: 20),

                        // Título del Incidente
                        Text(
                          'TÍTULO DEL REPORTE',
                          style: GoogleFonts.outfit(
                            fontSize: 10,
                            fontWeight: FontWeight.bold,
                            color: const Color(0xFF64748B),
                            letterSpacing: 1.0,
                          ),
                        ),
                        const SizedBox(height: 8),
                        TextFormField(
                          controller: _titleController,
                          style: GoogleFonts.outfit(color: const Color(0xFF0F172A), fontSize: 14),
                          decoration: _buildInputDecoration('Ej. Derrame de relaves mineros, Basural acumulado...'),
                          validator: (value) {
                            if (value == null || value.trim().isEmpty) {
                              return 'Por favor ingresa un título para el reporte';
                            }
                            return null;
                          },
                        ),
                        const SizedBox(height: 20),

                        // Descripción Detallada
                        Text(
                          'DESCRIPCIÓN DETALLADA',
                          style: GoogleFonts.outfit(
                            fontSize: 10,
                            fontWeight: FontWeight.bold,
                            color: const Color(0xFF64748B),
                            letterSpacing: 1.0,
                          ),
                        ),
                        const SizedBox(height: 8),
                        TextFormField(
                          controller: _descController,
                          maxLines: 3,
                          style: GoogleFonts.outfit(color: const Color(0xFF0F172A), fontSize: 14),
                          decoration: _buildInputDecoration('Describe lo que está sucediendo y el impacto visual observado...'),
                          validator: (value) {
                            if (value == null || value.trim().isEmpty) {
                              return 'Por favor detalla la descripción del incidente';
                            }
                            return null;
                          },
                        ),
                      ],
                    ),
                  ),

                  const Divider(color: Color(0xFFE2E8F0), height: 1),

                  // Actions
                  Padding(
                    padding: const EdgeInsets.all(24.0),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.end,
                      children: [
                        TextButton(
                          onPressed: _isSubmitting
                              ? null
                              : () {
                                  appState.cancelReportMode();
                                  Navigator.of(context).pop();
                                },
                          style: TextButton.styleFrom(
                            foregroundColor: const Color(0xFF64748B),
                            padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 16),
                          ),
                          child: Text('Cancelar', style: GoogleFonts.outfit(fontWeight: FontWeight.w600)),
                        ),
                        const SizedBox(width: 12),
                        ElevatedButton(
                          onPressed: (_isSubmitting || cooldownRemaining > 0 || nearbyDuplicate != null)
                              ? null
                              : _submitReport,
                          style: ElevatedButton.styleFrom(
                            backgroundColor: const Color(0xFF0288D1), // Blue
                            foregroundColor: Colors.white,
                            padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 16),
                            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                            elevation: 0,
                          ),
                          child: _isSubmitting
                              ? const SizedBox(
                                  width: 20,
                                  height: 20,
                                  child: CircularProgressIndicator(
                                    strokeWidth: 2.5,
                                    valueColor: AlwaysStoppedAnimation<Color>(Colors.white),
                                  ),
                                )
                              : Text(
                                  cooldownRemaining > 0
                                      ? 'Espera (${cooldownRemaining}s)'
                                      : nearbyDuplicate != null
                                          ? 'Alerta ya Existente'
                                          : 'Enviar Alerta',
                                  style: GoogleFonts.outfit(
                                    fontWeight: FontWeight.bold,
                                    fontSize: 14,
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
    );
  }

  Widget _buildCategorySelector() {
    final categories = [
      {'val': EcoCategory.mineria, 'label': 'Minería', 'icon': Icons.terrain_rounded, 'color': const Color(0xFFE65100)},
      {'val': EcoCategory.basura, 'label': 'Basura', 'icon': Icons.delete_outline_rounded, 'color': const Color(0xFF607D8B)},
      {'val': EcoCategory.agua, 'label': 'Agua', 'icon': Icons.water_drop_rounded, 'color': const Color(0xFF0288D1)},
      {'val': EcoCategory.aire, 'label': 'Aire', 'icon': Icons.air_rounded, 'color': const Color(0xFF00897B)},
    ];

    return Row(
      children: categories.map((cat) {
        final val = cat['val'] as EcoCategory;
        final label = cat['label'] as String;
        final icon = cat['icon'] as IconData;
        final color = cat['color'] as Color;
        final isSelected = _selectedCategory == val;

        return Expanded(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 4.0),
            child: GestureDetector(
              onTap: () {
                setState(() {
                  _selectedCategory = val;
                });
              },
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 150),
                padding: const EdgeInsets.symmetric(vertical: 12),
                decoration: BoxDecoration(
                  color: isSelected ? color.withValues(alpha: 0.12) : const Color(0xFFF1F5F9),
                  borderRadius: BorderRadius.circular(12),
                  border: Border.all(
                    color: isSelected ? color : const Color(0xFFE2E8F0),
                    width: 1.5,
                  ),
                ),
                child: Column(
                  children: [
                    Icon(icon, color: isSelected ? color : const Color(0xFF64748B), size: 20),
                    const SizedBox(height: 6),
                    Text(
                      label,
                      style: GoogleFonts.outfit(
                        color: isSelected ? const Color(0xFF0F172A) : const Color(0xFF64748B),
                        fontSize: 11,
                        fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                      ),
                    )
                  ],
                ),
              ),
            ),
          ),
        );
      }).toList(),
    );
  }

  Widget _buildSeveritySelector() {
    final severities = [
      {'val': EcoSeverity.critico, 'label': 'Crítico', 'emoji': '💀', 'color': const Color(0xFFD32F2F)},
      {'val': EcoSeverity.medio, 'label': 'Medio', 'emoji': '⚠️', 'color': const Color(0xFFFFA000)},
      {'val': EcoSeverity.bajo, 'label': 'Bajo', 'emoji': '🍃', 'color': const Color(0xFF388E3C)},
    ];

    return Row(
      children: severities.map((sev) {
        final val = sev['val'] as EcoSeverity;
        final label = sev['label'] as String;
        final emoji = sev['emoji'] as String;
        final color = sev['color'] as Color;
        final isSelected = _selectedSeverity == val;

        return Expanded(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 4.0),
            child: GestureDetector(
              onTap: () {
                setState(() {
                  _selectedSeverity = val;
                });
              },
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 150),
                padding: const EdgeInsets.symmetric(vertical: 10),
                decoration: BoxDecoration(
                  color: isSelected ? color.withValues(alpha: 0.12) : const Color(0xFFF1F5F9),
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(
                    color: isSelected ? color : const Color(0xFFE2E8F0),
                    width: 1.5,
                  ),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text(emoji, style: const TextStyle(fontSize: 14)),
                    const SizedBox(width: 6),
                    Text(
                      label,
                      style: GoogleFonts.outfit(
                        color: isSelected ? const Color(0xFF0F172A) : const Color(0xFF64748B),
                        fontSize: 12,
                        fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                      ),
                    )
                  ],
                ),
              ),
            ),
          ),
        );
      }).toList(),
    );
  }

  Widget _buildImageUploader(AppState appState) {
    final isGeminiOn = appState.isGeminiEnabled;

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: const Color(0xFFF8FAFC),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: const Color(0xFFE2E8F0), width: 1.5),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Encabezado de la Sección de Evidencias
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Row(
                children: [
                  const Icon(Icons.add_a_photo_outlined, color: Color(0xFF0288D1), size: 18),
                  const SizedBox(width: 8),
                  Text(
                    '1. EVIDENCIA FOTOGRÁFICA',
                    style: GoogleFonts.outfit(
                      fontSize: 11,
                      fontWeight: FontWeight.bold,
                      color: const Color(0xFF0F172A),
                      letterSpacing: 0.8,
                    ),
                  ),
                ],
              ),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                decoration: BoxDecoration(
                  color: _photos.length >= 2 ? const Color(0xFFFEF3C7) : const Color(0xFFEFF6FF),
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(
                    color: _photos.length >= 2 ? const Color(0xFFF59E0B) : const Color(0xFFBFDBFE),
                  ),
                ),
                child: Text(
                  '${_photos.length}/2 fotos (Máximo)',
                  style: GoogleFonts.outfit(
                    fontSize: 10.5,
                    fontWeight: FontWeight.bold,
                    color: _photos.length >= 2 ? const Color(0xFFB45309) : const Color(0xFF1E40AF),
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 6),
          Text(
            isGeminiOn
                ? '✨ Sube tu foto y Gemini AI validará el incidente en tiempo real, completando automáticamente la categoría, gravedad y descripción.'
                : 'Modo manual activo: Puedes adjuntar hasta 2 fotografías de respaldo y completar los campos manualmente.',
            style: GoogleFonts.outfit(
              fontSize: 11.5,
              color: isGeminiOn ? const Color(0xFF0369A1) : const Color(0xFF64748B),
              height: 1.35,
            ),
          ),
          const SizedBox(height: 12),

          // Lista de Fotos Adjuntadas
          if (_photos.isNotEmpty) ...[
            ...List.generate(_photos.length, (index) {
              final photo = _photos[index];
              final isMain = index == 0;
              return Container(
                margin: const EdgeInsets.only(bottom: 8),
                padding: const EdgeInsets.all(10),
                decoration: BoxDecoration(
                  color: Colors.white,
                  borderRadius: BorderRadius.circular(6),
                  border: Border.all(color: const Color(0xFFCBD5E1)),
                  boxShadow: const [
                    BoxShadow(color: Color(0x0A0F172A), blurRadius: 4, offset: Offset(0, 1)),
                  ],
                ),
                child: Row(
                  children: [
                    ClipRRect(
                      borderRadius: BorderRadius.circular(4),
                      child: photo['bytes'] != null
                          ? Image.memory(
                              photo['bytes'] as Uint8List,
                              width: 38,
                              height: 38,
                              fit: BoxFit.cover,
                              errorBuilder: (c, e, s) => Container(
                                width: 38,
                                height: 38,
                                color: const Color(0xFF0288D1).withValues(alpha: 0.1),
                                child: const Icon(Icons.image_rounded, color: Color(0xFF0288D1), size: 22),
                              ),
                            )
                          : Container(
                              width: 38,
                              height: 38,
                              decoration: BoxDecoration(
                                color: const Color(0xFF0288D1).withValues(alpha: 0.1),
                                borderRadius: BorderRadius.circular(4),
                              ),
                              child: const Icon(Icons.image_rounded, color: Color(0xFF0288D1), size: 22),
                            ),
                    ),
                    const SizedBox(width: 10),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              Container(
                                padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                                decoration: BoxDecoration(
                                  color: isMain ? const Color(0xFFDCFCE7) : const Color(0xFFF1F5F9),
                                  borderRadius: BorderRadius.circular(3),
                                ),
                                child: Text(
                                  isMain ? 'Foto 1 (Principal)' : 'Foto 2 (Respaldo)',
                                  style: GoogleFonts.outfit(
                                    fontSize: 9.5,
                                    fontWeight: FontWeight.bold,
                                    color: isMain ? const Color(0xFF15803D) : const Color(0xFF475569),
                                  ),
                                ),
                              ),
                              const SizedBox(width: 6),
                              Expanded(
                                child: Text(
                                  photo['name'] ?? 'evidencia.jpg',
                                  overflow: TextOverflow.ellipsis,
                                  style: GoogleFonts.outfit(
                                    fontSize: 12,
                                    fontWeight: FontWeight.bold,
                                    color: const Color(0xFF0F172A),
                                  ),
                                ),
                              ),
                            ],
                          ),
                          const SizedBox(height: 2),
                          Text(
                            isMain && isGeminiOn
                                ? (_isAnalyzingAI ? 'Analizando con Gemini AI...' : 'Foto examinada por la IA')
                                : 'Foto de respaldo adjuntada',
                            style: GoogleFonts.outfit(
                              fontSize: 10,
                              color: const Color(0xFF64748B),
                            ),
                          ),
                        ],
                      ),
                    ),
                    IconButton(
                      icon: const Icon(Icons.close_rounded, color: Color(0xFF94A3B8), size: 18),
                      padding: EdgeInsets.zero,
                      constraints: const BoxConstraints(),
                      tooltip: 'Eliminar foto',
                      onPressed: (_isUploadingPhoto || _isAnalyzingAI) ? null : () => _removePhoto(index),
                    ),
                  ],
                ),
              );
            }),
          ],

          // Indicador de Subida y Compresión WebP
          if (_isUploadingPhoto) ...[
            Container(
              padding: const EdgeInsets.symmetric(vertical: 12),
              alignment: Alignment.center,
              child: Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2, color: Color(0xFF0288D1)),
                  ),
                  const SizedBox(width: 10),
                  Text(
                    '⚡ Optimizando a WebP y subiendo al servidor...',
                    style: GoogleFonts.outfit(fontSize: 12, color: const Color(0xFF0288D1), fontWeight: FontWeight.w600),
                  ),
                ],
              ),
            ),
          ],

          // Botón para Adjuntar Foto (Hasta 2)
          if (_photos.length < 2 && !_isUploadingPhoto) ...[
            InkWell(
              onTap: _pickAndUploadPhoto,
              borderRadius: BorderRadius.circular(6),
              child: Container(
                padding: const EdgeInsets.symmetric(vertical: 11, horizontal: 14),
                decoration: BoxDecoration(
                  color: Colors.white,
                  borderRadius: BorderRadius.circular(6),
                  border: Border.all(color: const Color(0xFF0288D1), width: 1.2),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    const Icon(Icons.cloud_upload_outlined, color: Color(0xFF0288D1), size: 18),
                    const SizedBox(width: 8),
                    Text(
                      _photos.isEmpty
                          ? (isGeminiOn ? '＋ Subir Foto (Auto-análisis con Gemini)' : '＋ Subir Primera Foto')
                          : '＋ Subir Segunda Foto de Respaldo',
                      style: GoogleFonts.outfit(
                        color: const Color(0xFF0288D1),
                        fontSize: 12.5,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ],

          // Indicador de Análisis de IA en Curso
          if (_isAnalyzingAI) ...[
            const SizedBox(height: 10),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: const Color(0xFFEFF6FF),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: const Color(0xFF93C5FD)),
              ),
              child: Row(
                children: [
                  const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2, color: Color(0xFF0288D1)),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          '🤖 Gemini AI analizando la evidencia fotográfica...',
                          style: GoogleFonts.outfit(
                            fontSize: 12,
                            fontWeight: FontWeight.bold,
                            color: const Color(0xFF1E40AF),
                          ),
                        ),
                        Text(
                          'Validando infracción ambiental y autocompletando formulario...',
                          style: GoogleFonts.outfit(
                            fontSize: 10.5,
                            color: const Color(0xFF3B82F6),
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ],

          // Tarjeta de Resultado Pericial de Gemini AI
          if (_aiAnalysisResult != null) ...[
            const SizedBox(height: 10),
            Builder(
              builder: (context) {
                final bool isMinorLitter = _aiAnalysisResult!['is_minor_litter'] == true ||
                    _aiAnalysisResult!['rejection_reason'] == 'minor_litter' ||
                    _aiAnalysisResult!['waste_scale'] == 'basura_menor';
                final bool isIssue = !isMinorLitter && _aiAnalysisResult!['is_environmental_issue'] != false;

                Color cardBg;
                Color cardBorder;
                IconData cardIcon;
                Color iconColor;
                String cardTitle;
                Color titleColor;

                if (isMinorLitter) {
                  cardBg = const Color(0xFFFFFBEB); // Amber 50
                  cardBorder = const Color(0xFFFDE68A); // Amber 200
                  cardIcon = Icons.local_shipping_rounded;
                  iconColor = const Color(0xFFD97706); // Amber 600
                  cardTitle = '🚛 Atención de Rutina: Camión Recolector';
                  titleColor = const Color(0xFFB45309);
                } else if (isIssue) {
                  cardBg = const Color(0xFFF0FDF4); // Green 50
                  cardBorder = const Color(0xFF86EFAC); // Green 200
                  cardIcon = Icons.verified_rounded;
                  iconColor = const Color(0xFF16A34A);
                  cardTitle = '✓ Evidencia Aprobada por Gemini AI';
                  titleColor = const Color(0xFF15803D);
                } else {
                  cardBg = const Color(0xFFFEF2F2); // Red 50
                  cardBorder = const Color(0xFFFCA5A5); // Red 200
                  cardIcon = Icons.cancel_rounded;
                  iconColor = const Color(0xFFDC2626);
                  cardTitle = '⛔ Evidencia Rechazada por el Filtro de IA';
                  titleColor = const Color(0xFF991B1B);
                }

                return Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: cardBg,
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(color: cardBorder),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Icon(cardIcon, color: iconColor, size: 18),
                          const SizedBox(width: 8),
                          Expanded(
                            child: Text(
                              cardTitle,
                              style: GoogleFonts.outfit(
                                color: titleColor,
                                fontWeight: FontWeight.bold,
                                fontSize: 12.5,
                              ),
                            ),
                          ),
                        ],
                      ),
                      if (_aiAnalysisResult!['technical_analysis'] != null) ...[
                        const SizedBox(height: 5),
                        Text(
                          _aiAnalysisResult!['technical_analysis'].toString(),
                          style: GoogleFonts.outfit(
                            color: isMinorLitter ? const Color(0xFF92400E) : (isIssue ? const Color(0xFF166534) : const Color(0xFFB91C1C)),
                            fontSize: 11.5,
                            height: 1.35,
                          ),
                        ),
                      ],
                      if (isMinorLitter) ...[
                        const SizedBox(height: 6),
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                          decoration: BoxDecoration(
                            color: const Color(0xFFFEF3C7),
                            borderRadius: BorderRadius.circular(4),
                            border: Border.all(color: const Color(0xFFF59E0B)),
                          ),
                          child: Row(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              const Icon(Icons.info_outline_rounded, size: 14, color: Color(0xFFB45309)),
                              const SizedBox(width: 6),
                              Expanded(
                                child: Text(
                                  'El camión recolector ya va a pasar por esa zona a recoger esta basura según su cronograma habitual. EcoAlerta está reservado para botaderos clandestinos y focos críticos de contaminación grande.',
                                  style: GoogleFonts.outfit(
                                    color: const Color(0xFF92400E),
                                    fontSize: 10.5,
                                    fontWeight: FontWeight.w600,
                                    height: 1.3,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ] else if (isIssue && _aiAnalysisResult!['municipal_action'] != null) ...[
                        const SizedBox(height: 5),
                        Text(
                          'Acción municipal: ${_aiAnalysisResult!['municipal_action']}',
                          style: GoogleFonts.outfit(
                            color: const Color(0xFF14532D),
                            fontSize: 11,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ] else if (!isIssue && !isMinorLitter) ...[
                        const SizedBox(height: 6),
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                          decoration: BoxDecoration(
                            color: const Color(0xFFFEE2E2),
                            borderRadius: BorderRadius.circular(4),
                            border: Border.all(color: const Color(0xFFF87171)),
                          ),
                          child: Text(
                            '⚠️ No se admiten fotos no ambientales (selfies, memes, comida, documentos o fotos ajenas). Elimina esta foto y sube una evidencia ambiental para poder enviar.',
                            style: GoogleFonts.outfit(
                              color: const Color(0xFF991B1B),
                              fontSize: 10.5,
                              fontWeight: FontWeight.w600,
                            ),
                          ),
                        ),
                      ],
                    ],
                  ),
                );
              },
            ),
          ],

          // Botón de reintento si la IA está activa pero falló el peritaje inicial
          if (_photos.isNotEmpty && isGeminiOn && _aiAnalysisResult == null && !_isAnalyzingAI && !_isUploadingPhoto) ...[
            const SizedBox(height: 8),
            Align(
              alignment: Alignment.centerRight,
              child: TextButton.icon(
                onPressed: _retryAnalysisWithGemini,
                icon: const Icon(Icons.refresh_rounded, size: 14, color: Color(0xFF0288D1)),
                label: Text(
                  'Reintentar análisis con Gemini AI',
                  style: GoogleFonts.outfit(fontSize: 11, fontWeight: FontWeight.bold, color: const Color(0xFF0288D1)),
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }

  InputDecoration _buildInputDecoration(String hintText) {
    return InputDecoration(
      hintText: hintText,
      hintStyle: GoogleFonts.outfit(color: const Color(0xFF94A3B8), fontSize: 13),
      filled: true,
      fillColor: const Color(0xFFF1F5F9),
      contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      errorStyle: GoogleFonts.outfit(color: Colors.redAccent, fontSize: 12),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(6),
        borderSide: const BorderSide(color: Color(0xFFE2E8F0), width: 1.5),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(6),
        borderSide: const BorderSide(color: Color(0xFF0288D1), width: 1.5),
      ),
      errorBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(6),
        borderSide: const BorderSide(color: Colors.redAccent, width: 1.5),
      ),
      focusedErrorBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(6),
        borderSide: const BorderSide(color: Colors.redAccent, width: 2),
      ),
    );
  }

  void _submitReport() async {
    if (!_formKey.currentState!.validate()) return;

    // 1. Bloquear si la IA detectó residuo menor o rechazó la foto por no ser ambiental
    if (_aiAnalysisResult != null) {
      final bool isMinorLitter = _aiAnalysisResult!['is_minor_litter'] == true ||
          _aiAnalysisResult!['rejection_reason'] == 'minor_litter' ||
          _aiAnalysisResult!['waste_scale'] == 'basura_menor';

      if (isMinorLitter) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            backgroundColor: const Color(0xFFD97706),
            duration: const Duration(seconds: 5),
            content: Row(
              children: [
                const Icon(Icons.local_shipping_rounded, color: Colors.white),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    '🚛 El camión recolector ya va a pasar por esa zona a recoger esta basura. EcoAlerta está reservado para grandes focos o botaderos.',
                    style: GoogleFonts.outfit(color: Colors.white, fontWeight: FontWeight.bold),
                  ),
                ),
              ],
            ),
          ),
        );
        return;
      }

      if (_aiAnalysisResult!['is_environmental_issue'] == false) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            backgroundColor: const Color(0xFFDC2626),
            duration: const Duration(seconds: 4),
            content: Row(
              children: [
                const Icon(Icons.block_rounded, color: Colors.white),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    '⛔ Evidencia rechazada: La foto no corresponde a contaminación ambiental. Cambia la foto para poder registrar el reporte.',
                    style: GoogleFonts.outfit(color: Colors.white, fontWeight: FontWeight.bold),
                  ),
                ),
              ],
            ),
          ),
        );
        return;
      }
    }

    final appState = Provider.of<AppState>(context, listen: false);

    // 2. Si adjuntó foto con Gemini activado pero no fue analizada: analizar en tiempo real antes de enviar
    if (_photos.isNotEmpty && _aiAnalysisResult == null && appState.isGeminiEnabled) {
      setState(() {
        _isSubmitting = true;
        _isAnalyzingAI = true;
      });
      try {
        final first = _photos.first;
        final result = await appState.analyzeEvidenceWithGemini(
          imageUrl: first['url'] as String?,
          fileName: first['name'] as String?,
        );
        if (!mounted) return;
        setState(() {
          _aiAnalysisResult = result;
          _isAnalyzingAI = false;
        });

        final bool isMinorLitter = result['is_minor_litter'] == true ||
            result['rejection_reason'] == 'minor_litter' ||
            result['waste_scale'] == 'basura_menor';

        if (isMinorLitter) {
          setState(() {
            _isSubmitting = false;
          });
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              backgroundColor: const Color(0xFFD97706),
              duration: const Duration(seconds: 5),
              content: Row(
                children: [
                  const Icon(Icons.local_shipping_rounded, color: Colors.white),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      '🚛 El camión recolector ya va a pasar por esa zona a recoger esta basura. EcoAlerta está reservado para grandes focos o botaderos.',
                      style: GoogleFonts.outfit(color: Colors.white, fontWeight: FontWeight.bold),
                    ),
                  ),
                ],
              ),
            ),
          );
          return;
        }

        if (result['is_environmental_issue'] == false) {
          setState(() {
            _isSubmitting = false;
          });
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              backgroundColor: const Color(0xFFDC2626),
              duration: const Duration(seconds: 5),
              content: Row(
                children: [
                  const Icon(Icons.block_rounded, color: Colors.white),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      '⛔ Filtro Anti-Spam de IA: La foto no muestra contaminación ambiental real en Cerro de Pasco. Cambia la foto para continuar.',
                      style: GoogleFonts.outfit(color: Colors.white, fontWeight: FontWeight.bold),
                    ),
                  ),
                ],
              ),
            ),
          );
          return;
        }
      } catch (e) {
        // En caso de error de red o cuota agotada, permitir continuar con envío manual
        setState(() {
          _isAnalyzingAI = false;
        });
      }
    }

    setState(() {
      _isSubmitting = true;
    });

    final success = await appState.createAlert(
      title: _titleController.text.trim(),
      description: _descController.text.trim(),
      category: _selectedCategory,
      severity: _selectedSeverity,
      address: _addressController.text.trim(),
      imageUrl: _photos.isNotEmpty ? _photos[0]['url'] as String? : null,
      imageUrl2: _photos.length > 1 ? _photos[1]['url'] as String? : null,
    );

    if (mounted) {
      setState(() {
        _isSubmitting = false;
      });

      if (success) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            backgroundColor: const Color(0xFF10B981),
            content: Row(
              children: [
                const Icon(Icons.check_circle, color: Colors.white),
                const SizedBox(width: 8),
                Text(
                  '¡Alerta ambiental registrada correctamente!',
                  style: GoogleFonts.outfit(color: Colors.white, fontWeight: FontWeight.w600),
                ),
              ],
            ),
          ),
        );
        Navigator.of(context).pop();
      } else {
        final errorMsg = appState.lastAlertError ?? 'Ocurrió un error al registrar la alerta. Inténtelo nuevamente.';
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            backgroundColor: const Color(0xFFDC2626),
            duration: const Duration(seconds: 4),
            content: Row(
              children: [
                const Icon(Icons.error_outline, color: Colors.white),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    errorMsg,
                    style: GoogleFonts.outfit(color: Colors.white, fontWeight: FontWeight.w600),
                  ),
                ),
              ],
            ),
          ),
        );
      }
    }
  }
}
