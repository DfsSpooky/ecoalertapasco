import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:provider/provider.dart';
import '../models/eco_alert.dart';
import '../state/app_state.dart';
import 'package:file_picker/file_picker.dart';
import 'eco_notification.dart';

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
  
  bool _isUploadingImage = false;
  String? _mockImageName;
  bool _isSubmitting = false;
  List<int>? _selectedFileBytes;
  String? _uploadedImageUrl;

  @override
  void initState() {
    super.initState();
    final appState = Provider.of<AppState>(context, listen: false);
    if (appState.tempLatitude != null && appState.tempLongitude != null) {
      _addressController.text = appState.currentGeocodedAddress == 'Alineando mira...'
          ? appState.reverseGeocode(appState.tempLatitude!, appState.tempLongitude!)
          : appState.currentGeocodedAddress;
    }
  }

  @override
  void dispose() {
    _titleController.dispose();
    _descController.dispose();
    _addressController.dispose();
    super.dispose();
  }

  // Carga real de fotos utilizando FilePicker y Django REST api
  void _pickAndUploadImage() async {
    final appState = Provider.of<AppState>(context, listen: false);
    final result = await FilePicker.platform.pickFiles(type: FileType.image);
    if (!mounted) return;
    if (result != null && result.files.first.bytes != null) {
      setState(() {
        _isUploadingImage = true;
        _mockImageName = result.files.first.name;
        _selectedFileBytes = result.files.first.bytes;
      });
      
      final url = await appState.uploadImage(_selectedFileBytes!, _mockImageName!);
      
      if (!mounted) return;
      setState(() {
        _isUploadingImage = false;
        if (url != null) {
          _uploadedImageUrl = url;
        } else {
          _mockImageName = null;
          _selectedFileBytes = null;
          EcoNotification.show(
            context,
            title: 'Error de Imagen',
            message: 'Error al subir la foto al servidor.',
            type: EcoNotificationType.error,
          );
        }
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context, listen: false);

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
                                  EcoNotification.show(
                                    context,
                                    title: 'Ubicación GPS',
                                    message: 'Ubicación GPS fijada correctamente.',
                                    type: EcoNotificationType.success,
                                  );
                                }
                              } else {
                                if (context.mounted) {
                                  EcoNotification.show(
                                    context,
                                    title: 'Ubicación GPS',
                                    message: 'No se pudo obtener la ubicación GPS. Verifique permisos.',
                                    type: EcoNotificationType.warning,
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
                        const SizedBox(height: 20),

                        // Carga de Foto Simulada
                        Text(
                          'FOTO DE EVIDENCIA (OPCIONAL)',
                          style: GoogleFonts.outfit(
                            fontSize: 10,
                            fontWeight: FontWeight.bold,
                            color: const Color(0xFF64748B),
                            letterSpacing: 1.0,
                          ),
                        ),
                        const SizedBox(height: 8),
                        _buildImageUploader(),
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
                          onPressed: _isSubmitting ? null : _submitReport,
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
                                  'Enviar Alerta',
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

  Widget _buildImageUploader() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InkWell(
          onTap: _isUploadingImage ? null : _pickAndUploadImage,
          borderRadius: BorderRadius.circular(12),
          child: Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: const Color(0xFFF1F5F9),
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: const Color(0xFFE2E8F0), width: 1.5),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                if (_isUploadingImage)
                  const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      valueColor: AlwaysStoppedAnimation<Color>(Color(0xFF0288D1)),
                    ),
                  )
                else if (_mockImageName != null)
                  const Icon(Icons.check_circle_rounded, color: Color(0xFF10B981), size: 20)
                else
                  const Icon(Icons.add_photo_alternate_outlined, color: Color(0xFF64748B), size: 20),
                const SizedBox(width: 8),
                Text(
                  _isUploadingImage
                      ? 'Subiendo evidencia...'
                      : (_mockImageName ?? 'Haga clic para subir una foto...'),
                  style: GoogleFonts.outfit(
                    color: _mockImageName != null ? const Color(0xFF0F172A) : const Color(0xFF64748B),
                    fontSize: 13,
                    fontWeight: _mockImageName != null ? FontWeight.bold : FontWeight.normal,
                  ),
                ),
              ],
            ),
          ),
        ),
        if (_mockImageName != null) ...[
          const SizedBox(height: 10),
          Container(
            height: 120,
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: const Color(0xFFE2E8F0)),
              gradient: LinearGradient(
                colors: [
                  const Color(0xFF0288D1).withValues(alpha: 0.2),
                  const Color(0xFF00897B).withValues(alpha: 0.1),
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
                      const Icon(Icons.image_rounded, color: Color(0xFF0288D1), size: 36),
                      const SizedBox(height: 6),
                      Text(
                        _mockImageName!,
                        style: GoogleFonts.outfit(
                          color: const Color(0xFF334155),
                          fontSize: 12,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      Text(
                        'Carga completada con éxito',
                        style: GoogleFonts.outfit(
                          color: const Color(0xFF64748B),
                          fontSize: 10,
                        ),
                      ),
                    ],
                  ),
                ),
                Positioned(
                  top: 4,
                  right: 4,
                  child: IconButton(
                    icon: const Icon(Icons.cancel, color: Color(0xFF94A3B8), size: 20),
                    onPressed: () {
                      setState(() {
                        _mockImageName = null;
                      });
                    },
                  ),
                )
              ],
            ),
          ),
        ]
      ],
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
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: Color(0xFFE2E8F0), width: 1.5),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: Color(0xFF0288D1), width: 1.5),
      ),
      errorBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: Colors.redAccent, width: 1.5),
      ),
      focusedErrorBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: Colors.redAccent, width: 2),
      ),
    );
  }

  void _submitReport() async {
    if (!_formKey.currentState!.validate()) return;
    
    setState(() {
      _isSubmitting = true;
    });

    final appState = Provider.of<AppState>(context, listen: false);
    final success = await appState.createAlert(
      title: _titleController.text.trim(),
      description: _descController.text.trim(),
      category: _selectedCategory,
      severity: _selectedSeverity,
      address: _addressController.text.trim(),
      imageUrl: _uploadedImageUrl,
    );

    if (mounted) {
      setState(() {
        _isSubmitting = false;
      });

      if (success) {
        // Mostrar aviso de éxito
        EcoNotification.show(
          context,
          title: 'Alerta Registrada',
          message: '¡Alerta ambiental registrada correctamente!',
          type: EcoNotificationType.success,
        );
        Navigator.of(context).pop();
      } else {
        EcoNotification.show(
          context,
          title: 'Error de Envío',
          message: 'Ocurrió un error al registrar la alerta. Inténtelo nuevamente.',
          type: EcoNotificationType.error,
        );
      }
    }
  }
}
