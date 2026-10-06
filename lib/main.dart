import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:provider/provider.dart';
import 'services/django_alert_service.dart';
import 'state/app_state.dart';
import 'widgets/sidebar.dart';
import 'widgets/map_view.dart';
import 'widgets/authority_dashboard.dart';
import 'widgets/transparency_portal.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Inicializamos el servicio real de Django
  final alertService = DjangoAlertService();

  runApp(
    ChangeNotifierProvider(
      create: (_) => AppState(alertService),
      child: const EcoAlertaApp(),
    ),
  );
}

class EcoAlertaApp extends StatelessWidget {
  const EcoAlertaApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'EcoAlerta - Cerro de Pasco',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        useMaterial3: true,
        brightness: Brightness.light,
        scaffoldBackgroundColor: const Color(0xFFF8FAFC), // Slate 50
        colorScheme: ColorScheme.fromSeed(
          brightness: Brightness.light,
          seedColor: const Color(0xFF0288D1), // Blue
          surface: Colors.white,
        ),
        textTheme: GoogleFonts.outfitTextTheme(
          ThemeData.light().textTheme.apply(
            bodyColor: const Color(0xFF0F172A), // Slate 900
            displayColor: const Color(0xFF0F172A),
          ),
        ),
      ),
      home: const EcoAlertaDashboard(),
    );
  }
}

class EcoAlertaDashboard extends StatelessWidget {
  const EcoAlertaDashboard({super.key});

  @override
  Widget build(BuildContext context) {
    final appState = Provider.of<AppState>(context);

    if (appState.isLoading) {
      return const Scaffold(
        body: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              CircularProgressIndicator(
                valueColor: AlwaysStoppedAnimation<Color>(Color(0xFF0288D1)),
              ),
              SizedBox(height: 16),
              Text(
                'Cargando Dashboard Ambiental...',
                style: TextStyle(
                  color: Color(0xFF64748B), // Slate 500
                  fontSize: 14,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ],
          ),
        ),
      );
    }

    final screenWidth = MediaQuery.of(context).size.width;
    final isMobile = screenWidth < 850;

    final showDashboard = appState.isLoggedInAuthority && appState.showAuthorityDashboard;

    return Scaffold(
      drawer: isMobile
          ? const Drawer(
              elevation: 16,
              child: LeftSidebar(),
            )
          : null,
      body: Row(
        children: [
          if (!isMobile) const LeftSidebar(),
          Expanded(
            child: appState.showTransparencyPortal
                ? const TransparencyPortalView()
                : showDashboard
                    ? const AuthorityDashboardView()
                    : const EcoMapView(),
          ),
        ],
      ),
    );
  }
}
