import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:eco_alerta/main.dart';
import 'package:eco_alerta/state/app_state.dart';
import 'package:eco_alerta/services/mock_alert_service.dart';

void main() {
  testWidgets('EcoAlerta dashboard rendering test', (WidgetTester tester) async {
    final alertService = MockAlertService();
    
    await tester.pumpWidget(
      ChangeNotifierProvider(
        create: (_) => AppState(alertService),
        child: const EcoAlertaApp(),
      ),
    );

    expect(find.byType(EcoAlertaApp), findsOneWidget);
  });
}
