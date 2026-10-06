import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:eco_alerta/main.dart';
import 'package:eco_alerta/models/eco_alert.dart';
import 'package:eco_alerta/state/app_state.dart';
import 'package:eco_alerta/services/mock_alert_service.dart';

void main() {
  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  testWidgets('EcoAlerta dashboard rendering test', (WidgetTester tester) async {
    final alertService = MockAlertService();
    
    await tester.pumpWidget(
      ChangeNotifierProvider(
        create: (_) => AppState(alertService),
        child: const EcoAlertaApp(),
      ),
    );
    await tester.pump(const Duration(milliseconds: 400));
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.byType(EcoAlertaApp), findsOneWidget);
  });

  test('AppState enforces login before creating an alert', () async {
    final alertService = MockAlertService();
    final appState = AppState(alertService);

    expect(appState.isLoggedIn, isFalse);

    // Intentar crear alerta sin iniciar sesión debe retornar false
    final result = await appState.createAlert(
      title: 'Derrame Ilegal',
      description: 'Reporte sin autenticar',
      category: EcoCategory.agua,
      severity: EcoSeverity.critico,
      address: 'Cerro de Pasco',
    );

    expect(result, isFalse);
  });
}
