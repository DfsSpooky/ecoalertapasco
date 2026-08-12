import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

class MaintenanceScreen extends StatelessWidget {
  final String message;

  const MaintenanceScreen({
    super.key,
    required this.message,
  });

  @override
  Widget build(BuildContext context) {
    final cleanMessage = message.isNotEmpty
        ? message
        : 'El sistema se encuentra en mantenimiento temporal para mejorar los servicios. Disculpe las molestias.';

    return Scaffold(
      body: Container(
        decoration: const BoxDecoration(
          gradient: LinearGradient(
            colors: [
              Color(0xFF042F1A), // Deep Forest Green
              Color(0xFF022C22), // Dark Emerald
              Color(0xFF064E3B), // Rich Green
            ],
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
          ),
        ),
        child: Stack(
          children: [
            // Orbe difuso esmeralda superior derecho con desenfoque de luz real (Sunbeam Glow)
            Positioned(
              top: -80,
              right: -80,
              child: ImageFiltered(
                imageFilter: ImageFilter.blur(sigmaX: 120, sigmaY: 120),
                child: Container(
                  width: 320,
                  height: 320,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: const Color(0xFF10B981).withValues(alpha: 0.18), // Emerald glow
                  ),
                ),
              ),
            ),
            
            // Orbe difuso cálido inferior izquierdo (Golden glow)
            Positioned(
              bottom: -100,
              left: -100,
              child: ImageFiltered(
                imageFilter: ImageFilter.blur(sigmaX: 100, sigmaY: 100),
                child: Container(
                  width: 380,
                  height: 380,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: const Color(0xFFFBBF24).withValues(alpha: 0.08), // Amber glow
                  ),
                ),
              ),
            ),
            
            // Pequeñas luciérnagas flotantes (pequeños puntos de luz brillantes esparcidos)
            Positioned(
              top: MediaQuery.of(context).size.height * 0.15,
              left: MediaQuery.of(context).size.width * 0.2,
              child: _buildFirefly(Color(0xFFA7F3D0).withValues(alpha: 0.6), 6),
            ),
            Positioned(
              top: MediaQuery.of(context).size.height * 0.35,
              right: MediaQuery.of(context).size.width * 0.15,
              child: _buildFirefly(Color(0xFFFDE047).withValues(alpha: 0.5), 8),
            ),
            Positioned(
              bottom: MediaQuery.of(context).size.height * 0.25,
              right: MediaQuery.of(context).size.width * 0.25,
              child: _buildFirefly(Color(0xFF34D399).withValues(alpha: 0.7), 5),
            ),
            Positioned(
              bottom: MediaQuery.of(context).size.height * 0.15,
              left: MediaQuery.of(context).size.width * 0.3,
              child: _buildFirefly(Color(0xFFA7F3D0).withValues(alpha: 0.4), 7),
            ),
            
            // Contenido Central (Tarjeta de Vidrio Esmerilado)
            Center(
              child: SingleChildScrollView(
                padding: const EdgeInsets.symmetric(horizontal: 24),
                child: Container(
                  constraints: const BoxConstraints(maxWidth: 460),
                  padding: const EdgeInsets.symmetric(horizontal: 40, vertical: 50),
                  decoration: BoxDecoration(
                    color: Colors.white.withValues(alpha: 0.02),
                    borderRadius: BorderRadius.circular(36),
                    border: Border.all(
                      color: Colors.white.withValues(alpha: 0.08),
                      width: 1.5,
                    ),
                    boxShadow: [
                      BoxShadow(
                        color: Colors.black.withValues(alpha: 0.4),
                        blurRadius: 40,
                        offset: const Offset(0, 20),
                      ),
                    ],
                  ),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      // Icono ecológico de hoja (Eco)
                      Container(
                        padding: const EdgeInsets.all(24),
                        decoration: BoxDecoration(
                          color: const Color(0xFF10B981).withValues(alpha: 0.12),
                          shape: BoxShape.circle,
                          border: Border.all(
                            color: const Color(0xFF10B981).withValues(alpha: 0.25),
                            width: 1.5,
                          ),
                        ),
                        child: const Icon(
                          Icons.eco_rounded,
                          size: 56,
                          color: Color(0xFF34D399),
                        ),
                      ),
                      const SizedBox(height: 32),
                      
                      // Título principal
                      Text(
                        'Pausa Ecológica',
                        style: GoogleFonts.outfit(
                          fontSize: 32,
                          fontWeight: FontWeight.bold,
                          color: Colors.white,
                          letterSpacing: 0.5,
                        ),
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: 12),
                      
                      // Subtítulo
                      Text(
                        'CUIDADO Y MANTENIMIENTO DEL ENTORNO',
                        style: GoogleFonts.outfit(
                          fontSize: 10,
                          fontWeight: FontWeight.bold,
                          color: const Color(0xFF34D399),
                          letterSpacing: 1.8,
                        ),
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: 24),
                      
                      // Línea divisora verde
                      Container(
                        width: 60,
                        height: 3,
                        decoration: BoxDecoration(
                          color: const Color(0xFF10B981),
                          borderRadius: BorderRadius.circular(2),
                        ),
                      ),
                      const SizedBox(height: 28),
                      
                      // Mensaje personalizado del administrador
                      Text(
                        cleanMessage,
                        style: GoogleFonts.outfit(
                          fontSize: 16,
                          color: const Color(0xFFE2E8F0),
                          height: 1.7,
                        ),
                        textAlign: TextAlign.center,
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  // Helper para construir luciérnagas con brillo
  Widget _buildFirefly(Color color, double size) {
    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: color,
        boxShadow: [
          BoxShadow(
            color: color.withValues(alpha: 0.8),
            blurRadius: size * 1.5,
            spreadRadius: size * 0.5,
          ),
        ],
      ),
    );
  }
}
