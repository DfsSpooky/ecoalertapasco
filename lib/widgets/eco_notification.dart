import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

enum EcoNotificationType { success, error, info, warning }

class EcoNotification {
  static void show(
    BuildContext context, {
    required String title,
    required String message,
    EcoNotificationType type = EcoNotificationType.info,
    Duration duration = const Duration(seconds: 4),
  }) {
    final overlayState = Overlay.of(context);
    late OverlayEntry overlayEntry;

    // Colores y diseño premium
    Color accentColor;
    IconData icon;
    Color bgColor = const Color(0xFF0F172A).withOpacity(0.92); // Slate 900 con transparencia
    Color textColor = Colors.white;

    switch (type) {
      case EcoNotificationType.success:
        accentColor = const Color(0xFF10B981); // Esmeralda 500
        icon = Icons.check_circle_outline_rounded;
        break;
      case EcoNotificationType.error:
        accentColor = const Color(0xFFEF4444); // Rojo 500
        icon = Icons.error_outline_rounded;
        break;
      case EcoNotificationType.warning:
        accentColor = const Color(0xFFFF9800); // Naranja 500
        icon = Icons.warning_amber_rounded;
        break;
      case EcoNotificationType.info:
        accentColor = const Color(0xFF0288D1); // Celeste 500
        icon = Icons.info_outline_rounded;
        break;
    }

    overlayEntry = OverlayEntry(
      builder: (context) {
        return _EcoNotificationWidget(
          title: title,
          message: message,
          accentColor: accentColor,
          bgColor: bgColor,
          textColor: textColor,
          icon: icon,
          onDismiss: () {
            if (overlayEntry.mounted) {
              overlayEntry.remove();
            }
          },
        );
      },
    );

    overlayState.insert(overlayEntry);

    // Auto-remover después de la duración indicada
    Future.delayed(duration, () {
      if (overlayEntry.mounted) {
        overlayEntry.remove();
      }
    });
  }
}

class _EcoNotificationWidget extends StatefulWidget {
  final String title;
  final String message;
  final Color accentColor;
  final Color bgColor;
  final Color textColor;
  final IconData icon;
  final VoidCallback onDismiss;

  const _EcoNotificationWidget({
    required this.title,
    required this.message,
    required this.accentColor,
    required this.bgColor,
    required this.textColor,
    required this.icon,
    required this.onDismiss,
  });

  @override
  State<_EcoNotificationWidget> createState() => _EcoNotificationWidgetState();
}

class _EcoNotificationWidgetState extends State<_EcoNotificationWidget>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _slideAnimation;
  late Animation<double> _opacityAnimation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 350),
    );

    _slideAnimation = Tween<double>(begin: -60.0, end: 0.0).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeOutBack),
    );

    _opacityAnimation = Tween<double>(begin: 0.0, end: 1.0).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeIn),
    );

    _controller.forward();

    // Animación de salida programada
    Future.delayed(const Duration(seconds: 3, milliseconds: 600), () {
      if (mounted) {
        _controller.reverse();
      }
    });
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Positioned(
      top: 24,
      right: 24,
      child: Material(
        color: Colors.transparent,
        child: AnimatedBuilder(
          animation: _controller,
          builder: (context, child) {
            return Transform.translate(
              offset: Offset(0, _slideAnimation.value),
              child: Opacity(
                opacity: _opacityAnimation.value,
                child: child,
              ),
            );
          },
          child: Container(
            width: 320,
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
            decoration: BoxDecoration(
              color: widget.bgColor,
              borderRadius: BorderRadius.circular(16),
              border: Border.all(
                color: widget.accentColor.withOpacity(0.35),
                width: 1.5,
              ),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.24),
                  blurRadius: 18,
                  offset: const Offset(0, 8),
                )
              ],
            ),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(
                  padding: const EdgeInsets.all(6),
                  decoration: BoxDecoration(
                    color: widget.accentColor.withOpacity(0.12),
                    shape: BoxShape.circle,
                  ),
                  child: Icon(
                    widget.icon,
                    color: widget.accentColor,
                    size: 18,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        widget.title,
                        style: GoogleFonts.outfit(
                          fontSize: 13,
                          fontWeight: FontWeight.bold,
                          color: widget.textColor,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        widget.message,
                        style: GoogleFonts.outfit(
                          fontSize: 11,
                          color: widget.textColor.withOpacity(0.85),
                          height: 1.3,
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 8),
                GestureDetector(
                  onTap: widget.onDismiss,
                  child: Icon(
                    Icons.close_rounded,
                    color: widget.textColor.withOpacity(0.5),
                    size: 16,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
