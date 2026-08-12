import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:provider/provider.dart';
import '../state/app_state.dart';
import 'eco_notification.dart';

class AuthDialog extends StatefulWidget {
  const AuthDialog({super.key});

  @override
  State<AuthDialog> createState() => _AuthDialogState();
}

class _AuthDialogState extends State<AuthDialog> with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final _loginFormKey = GlobalKey<FormState>();
  final _registerFormKey = GlobalKey<FormState>();

  // Login Controllers
  final _loginUserCtrl = TextEditingController();
  final _loginPassCtrl = TextEditingController();

  // Register Controllers
  final _regUserCtrl = TextEditingController();
  final _regEmailCtrl = TextEditingController();
  final _regPassCtrl = TextEditingController();
  final _regPassConfirmCtrl = TextEditingController();

  bool _isSubmitting = false;
  bool _obscureLoginPass = true;
  bool _obscureRegPass = true;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    _loginUserCtrl.dispose();
    _loginPassCtrl.dispose();
    _regUserCtrl.dispose();
    _regEmailCtrl.dispose();
    _regPassCtrl.dispose();
    _regPassConfirmCtrl.dispose();
    super.dispose();
  }

  InputDecoration _buildInputDecoration(String label, IconData icon, {Widget? suffixIcon}) {
    return InputDecoration(
      labelText: label,
      labelStyle: GoogleFonts.outfit(color: const Color(0xFF64748B), fontSize: 13),
      floatingLabelStyle: GoogleFonts.outfit(color: const Color(0xFF0288D1), fontSize: 13, fontWeight: FontWeight.bold),
      prefixIcon: Icon(icon, size: 18, color: const Color(0xFF94A3B8)),
      suffixIcon: suffixIcon,
      filled: true,
      fillColor: const Color(0xFFF8FAFC),
      contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: Color(0xFFE2E8F0), width: 1.5),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: Color(0xFF0288D1), width: 2.0),
      ),
      errorBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: Colors.redAccent, width: 1.5),
      ),
      focusedErrorBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: Colors.redAccent, width: 2.0),
      ),
    );
  }

  void _handleLogin(AppState appState) async {
    if (!_loginFormKey.currentState!.validate()) return;

    setState(() {
      _isSubmitting = true;
    });

    final username = _loginUserCtrl.text.trim();
    final password = _loginPassCtrl.text.trim();

    final success = await appState.login(username, password);

    setState(() {
      _isSubmitting = false;
    });

    if (mounted) {
      if (success) {
        EcoNotification.show(
          context,
          title: 'Sesión Iniciada',
          message: '¡Bienvenido de nuevo, $username!',
          type: EcoNotificationType.success,
        );
        Navigator.pop(context);
      } else {
        EcoNotification.show(
          context,
          title: 'Error de Acceso',
          message: 'Usuario o contraseña incorrectos.',
          type: EcoNotificationType.error,
        );
      }
    }
  }

  void _handleRegister(AppState appState) async {
    if (!_registerFormKey.currentState!.validate()) return;

    setState(() {
      _isSubmitting = true;
    });

    final username = _regUserCtrl.text.trim();
    final email = _regEmailCtrl.text.trim();
    final password = _regPassCtrl.text.trim();

    final success = await appState.register(username, email, password);

    setState(() {
      _isSubmitting = false;
    });

    if (mounted) {
      if (success) {
        EcoNotification.show(
          context,
          title: 'Registro Completo',
          message: 'Cuenta creada con éxito. ¡Bienvenido, $username!',
          type: EcoNotificationType.success,
        );
        Navigator.pop(context);
      } else {
        EcoNotification.show(
          context,
          title: 'Error de Registro',
          message: 'Error al registrarse. El usuario o correo ya podría estar en uso.',
          type: EcoNotificationType.error,
        );
      }
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
            width: 420,
            decoration: BoxDecoration(
              color: Colors.white,
              borderRadius: BorderRadius.circular(24),
              border: Border.all(color: const Color(0xFFE2E8F0), width: 1.5),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withValues(alpha: 0.12),
                  blurRadius: 24,
                  offset: const Offset(0, 8),
                )
              ],
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                // Header del Dialog
                Padding(
                  padding: const EdgeInsets.only(top: 24, left: 24, right: 24, bottom: 8),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            'EcoAlerta',
                            style: GoogleFonts.outfit(
                              color: const Color(0xFF0F172A),
                              fontSize: 22,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          Text(
                            'Plataforma de Vigilancia Ambiental',
                            style: GoogleFonts.outfit(
                              color: const Color(0xFF64748B),
                              fontSize: 11,
                              fontWeight: FontWeight.w500,
                            ),
                          ),
                        ],
                      ),
                      IconButton(
                        icon: const Icon(Icons.close_rounded, color: Color(0xFF94A3B8)),
                        onPressed: () => Navigator.pop(context),
                      ),
                    ],
                  ),
                ),
                const Divider(color: Color(0xFFE2E8F0), height: 1),

                // Control de Pestañas
                TabBar(
                  controller: _tabController,
                  labelColor: const Color(0xFF0288D1),
                  unselectedLabelColor: const Color(0xFF64748B),
                  indicatorColor: const Color(0xFF0288D1),
                  indicatorSize: TabBarIndicatorSize.tab,
                  labelStyle: GoogleFonts.outfit(fontWeight: FontWeight.bold, fontSize: 13),
                  unselectedLabelStyle: GoogleFonts.outfit(fontWeight: FontWeight.w500, fontSize: 13),
                  tabs: const [
                    Tab(text: 'Iniciar Sesión'),
                    Tab(text: 'Registrarse'),
                  ],
                ),

                // Contenido de las pestañas
                Container(
                  constraints: const BoxConstraints(maxHeight: 380),
                  child: TabBarView(
                    controller: _tabController,
                    children: [
                      // Pestaña 1: Login
                      Padding(
                        padding: const EdgeInsets.all(24.0),
                        child: Form(
                          key: _loginFormKey,
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: [
                              TextFormField(
                                controller: _loginUserCtrl,
                                style: GoogleFonts.outfit(fontSize: 14),
                                decoration: _buildInputDecoration('Usuario', Icons.person_outline_rounded),
                                validator: (value) => value == null || value.trim().isEmpty ? 'Ingresa tu usuario' : null,
                              ),
                              const SizedBox(height: 16),
                              TextFormField(
                                controller: _loginPassCtrl,
                                obscureText: _obscureLoginPass,
                                style: GoogleFonts.outfit(fontSize: 14),
                                decoration: _buildInputDecoration(
                                  'Contraseña',
                                  Icons.lock_outline_rounded,
                                  suffixIcon: IconButton(
                                    icon: Icon(
                                      _obscureLoginPass ? Icons.visibility_off_rounded : Icons.visibility_rounded,
                                      size: 18,
                                      color: const Color(0xFF64748B),
                                    ),
                                    onPressed: () {
                                      setState(() {
                                        _obscureLoginPass = !_obscureLoginPass;
                                      });
                                    },
                                  ),
                                ),
                                validator: (value) => value == null || value.trim().isEmpty ? 'Ingresa tu contraseña' : null,
                              ),
                              const Spacer(),
                              ElevatedButton(
                                onPressed: _isSubmitting ? null : () => _handleLogin(appState),
                                style: ElevatedButton.styleFrom(
                                  backgroundColor: const Color(0xFF0288D1),
                                  foregroundColor: Colors.white,
                                  padding: const EdgeInsets.symmetric(vertical: 14),
                                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                                  elevation: 0,
                                ),
                                child: _isSubmitting
                                    ? const SizedBox(
                                        width: 18,
                                        height: 18,
                                        child: CircularProgressIndicator(strokeWidth: 2.0, color: Colors.white),
                                      )
                                    : Text(
                                        'Ingresar',
                                        style: GoogleFonts.outfit(fontWeight: FontWeight.bold, fontSize: 14),
                                      ),
                              ),
                            ],
                          ),
                        ),
                      ),

                      // Pestaña 2: Registro
                      SingleChildScrollView(
                        padding: const EdgeInsets.all(24.0),
                        child: Form(
                          key: _registerFormKey,
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: [
                              TextFormField(
                                controller: _regUserCtrl,
                                style: GoogleFonts.outfit(fontSize: 14),
                                decoration: _buildInputDecoration('Usuario', Icons.person_outline_rounded),
                                validator: (value) => value == null || value.trim().isEmpty ? 'Elige un usuario' : null,
                              ),
                              const SizedBox(height: 12),
                              TextFormField(
                                controller: _regEmailCtrl,
                                keyboardType: TextInputType.emailAddress,
                                style: GoogleFonts.outfit(fontSize: 14),
                                decoration: _buildInputDecoration('Correo Electrónico', Icons.email_outlined),
                                validator: (value) {
                                  if (value == null || value.trim().isEmpty) return 'Ingresa tu correo';
                                  if (!RegExp(r'^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$').hasMatch(value)) {
                                    return 'Ingresa un correo válido';
                                  }
                                  return null;
                                },
                              ),
                              const SizedBox(height: 12),
                              TextFormField(
                                controller: _regPassCtrl,
                                obscureText: _obscureRegPass,
                                style: GoogleFonts.outfit(fontSize: 14),
                                decoration: _buildInputDecoration(
                                  'Contraseña',
                                  Icons.lock_outline_rounded,
                                  suffixIcon: IconButton(
                                    icon: Icon(
                                      _obscureRegPass ? Icons.visibility_off_rounded : Icons.visibility_rounded,
                                      size: 18,
                                      color: const Color(0xFF64748B),
                                    ),
                                    onPressed: () {
                                      setState(() {
                                        _obscureRegPass = !_obscureRegPass;
                                      });
                                    },
                                  ),
                                ),
                                validator: (value) => value == null || value.length < 6 ? 'Mínimo 6 caracteres' : null,
                              ),
                              const SizedBox(height: 12),
                              TextFormField(
                                controller: _regPassConfirmCtrl,
                                obscureText: true,
                                style: GoogleFonts.outfit(fontSize: 14),
                                decoration: _buildInputDecoration('Confirmar Contraseña', Icons.lock_outline_rounded),
                                validator: (value) {
                                  if (value != _regPassCtrl.text) {
                                    return 'Las contraseñas no coinciden';
                                  }
                                  return null;
                                },
                              ),
                              const SizedBox(height: 20),
                              ElevatedButton(
                                onPressed: _isSubmitting ? null : () => _handleRegister(appState),
                                style: ElevatedButton.styleFrom(
                                  backgroundColor: const Color(0xFF0288D1),
                                  foregroundColor: Colors.white,
                                  padding: const EdgeInsets.symmetric(vertical: 14),
                                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                                  elevation: 0,
                                ),
                                child: _isSubmitting
                                    ? const SizedBox(
                                        width: 18,
                                        height: 18,
                                        child: CircularProgressIndicator(strokeWidth: 2.0, color: Colors.white),
                                      )
                                    : Text(
                                        'Crear Cuenta',
                                        style: GoogleFonts.outfit(fontWeight: FontWeight.bold, fontSize: 14),
                                      ),
                              ),
                            ],
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
    );
  }
}
