import 'package:flutter/material.dart';

class PulsingCriticalDot extends StatefulWidget {
  final bool isRadarRing;

  const PulsingCriticalDot({
    super.key,
    this.isRadarRing = false,
  });

  @override
  State<PulsingCriticalDot> createState() => _PulsingCriticalDotState();
}

class _PulsingCriticalDotState extends State<PulsingCriticalDot>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1200),
    )..repeat();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (widget.isRadarRing) {
      return AnimatedBuilder(
        animation: _controller,
        builder: (context, child) {
          // El anillo de radar se expande simétricamente desde el centro del pin
          final size = 32.0 + (24.0 * _controller.value);
          final opacity = 0.8 * (1.0 - _controller.value);
          return Container(
            width: size,
            height: size,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: const Color(0xFFEF4444).withOpacity(opacity * 0.12),
              border: Border.all(
                color: const Color(0xFFEF4444).withOpacity(opacity),
                width: 1.5,
              ),
              boxShadow: [
                BoxShadow(
                  color: const Color(0xFFEF4444).withOpacity(opacity * 0.25),
                  blurRadius: 8 * _controller.value,
                  spreadRadius: 2 * _controller.value,
                ),
              ],
            ),
          );
        },
      );
    } else {
      return AnimatedBuilder(
        animation: _controller,
        builder: (context, child) {
          // Pequeño indicador palpitante simétrico para texto/celdas
          final pulseValue = _controller.value;
          return Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(
              color: const Color(0xFFEF4444),
              shape: BoxShape.circle,
              boxShadow: [
                BoxShadow(
                  color: const Color(0xFFEF4444).withOpacity(1.0 - pulseValue),
                  blurRadius: 6 * pulseValue,
                  spreadRadius: 2 * pulseValue,
                ),
              ],
            ),
          );
        },
      );
    }
  }
}
