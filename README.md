# EcoAlerta 🌱 - Dashboard de Monitoreo Ambiental (Cerro de Pasco)

EcoAlerta es una aplicación web de ciencia ciudadana diseñada para reportar, visualizar y monitorear incidentes ecológicos (tales como contaminación de agua, aire, residuos sólidos e impacto de minería) en la ciudad de **Cerro de Pasco, Perú**.

La plataforma presenta una interfaz de control moderna en modo oscuro con efecto glassmorphism, filtros dinámicos, gráficos reactivos y un mapa interactivo completamente configurable.

---

## 🚀 Requisitos e Instalación

Para ejecutar la aplicación web en tu máquina local, asegúrate de tener instalado el SDK de Flutter.

### 1. Clonar o acceder al repositorio
Asegúrate de que estás situado en el directorio raíz del proyecto.

### 2. Descargar las dependencias
Ejecuta el siguiente comando para restaurar los paquetes de Flutter y Dart:
```bash
flutter pub get
```

### 3. Ejecutar la aplicación en Chrome
Inicia el servidor de desarrollo web y abre la aplicación en Google Chrome:
```bash
flutter run -d chrome
```

---

## 🎨 Características Implementadas

1. **Dashboard Lateral con Estética Premium**:
   - **Filtros por Categoría**: Filtra los reportes activos por tipo: Minería, Basura, Agua y Aire. Los botones adoptan el color distintivo y se reflejan en tiempo real.
   - **Filtros por Nivel**: Tres severidades de impacto (Crítico 💀, Medio ⚠️, Bajo 🍃).
   - **Resumen Mensual**: Gráfico de dona (Donut Chart) dinámico que resume el porcentaje de alertas por categoría.
   - **Resumen por Gravedad**: Gráfico de barras (Bar Chart) dinámico que compara las cantidades por severidad.
   - *Nota:* Tanto el mapa como los gráficos se actualizan reactivamente al interactuar con los filtros laterales.

2. **Mapa Interactivo (OpenStreetMap)**:
   - Centrado estrictamente en Cerro de Pasco, Perú (`Lat: -10.6675, Lon: -76.2567`, Zoom 14).
   - Marcadores dinámicos que cambian de color (según severidad) y muestran el icono de su categoría.
   - Cuenta con un botón selector de estilo de mapa (Alterna entre **Modo Claro** estándar y **Modo Oscuro** utilizando tiles oscuros de CartoDB).
   - Detalle flotante superpuesto en la esquina inferior izquierda al presionar cualquier marcador.

3. **Flujo de Reporte Rápido y Preciso**:
   - Al pulsar `+ Reportar Alerta`, la app entra en **Modo Reporte**, mostrando una mira central flotante.
   - El usuario arrastra el mapa para posicionar el incidente con precisión.
   - Al hacer clic en `Confirmar Ubicación`, se abre un formulario flotante con las coordenadas capturadas automáticamente.
   - Permite ingresar Título, Descripción, Categoría, Nivel de Gravedad y simular la subida de una foto.
   - Al enviar, la alerta se agrega en memoria y actualiza instantáneamente los gráficos y el mapa.

---

## 🗄️ Integración con Supabase (Opcional para Producción)

Para que el proyecto sea completamente desplegable en producción, hemos diseñado un esquema SQL robusto con soporte geográfico para PostgreSQL.

### 1. Configurar la base de datos
Importa el script SQL provisto en la carpeta del proyecto a tu consola de Supabase (SQL Editor):
- Archivo: [supabase/schema.sql](file:///Users/miguel/Documents/Proyectos/EcoAlerta/supabase/schema.sql)

Este script:
- Habilita la extensión `postgis` en tu base de datos.
- Crea la tabla `eco_alerts` con soporte de columnas geométricas (`Point`, SRID 4326).
- Habilita RLS (Row Level Security) y define políticas públicas de lectura e inserción.
- Llena la base de datos con las alertas iniciales geolocalizadas.

### 2. Conexión en Flutter
Para conectar tu app Flutter con Supabase:
1. Agrega la dependencia en `pubspec.yaml`:
   ```yaml
   dependencies:
     supabase_flutter: ^2.8.0
   ```
2. Reemplaza el servicio mock por uno conectado a Supabase. Puedes crear la siguiente clase en `lib/services/supabase_service.dart`:

```dart
import 'package:supabase_flutter/supabase_flutter.dart';
import '../models/eco_alert.dart';
import 'eco_alert_service.dart';

class SupabaseAlertService implements EcoAlertService {
  final _client = Supabase.instance.client;

  @override
  Stream<List<EcoAlert>> watchAlerts() {
    // Escucha en tiempo real la tabla de alertas
    return _client
        .from('eco_alerts')
        .stream(primaryKey: ['id'])
        .map((data) => data.map((json) => EcoAlert.fromJson(json)).toList());
  }

  @override
  Future<void> addAlert(EcoAlert alert) async {
    // Inserta una alerta con soporte PostGIS para el punto geométrico
    await _client.from('eco_alerts').insert({
      'title': alert.title,
      'description': alert.description,
      'category': alert.category.toString().split('.').last,
      'severity': alert.severity.toString().split('.').last,
      'location': 'SRID=4326;POINT(${alert.longitude} ${alert.latitude})',
      'image_url': alert.imageUrl,
    });
  }
}
```

3. Modifica tu `main.dart` para inicializar Supabase y pasar este nuevo servicio en el `ChangeNotifierProvider`.

---

## 🐳 Despliegue en Producción, Local y Coolify

Para desplegar la aplicación completa con **Docker Compose** (PostgreSQL, Django Backend REST API y Flutter Web Nginx):

```bash
# 1. Copiar configuración de entorno
cp .env.example .env

# 2. Levantar la pila completa en segundo plano
docker compose up -d --build

# 3. Comprobar salud del sistema
curl http://localhost:8000/api/health/
```

- **Guía Completa de Despliegue y Manual de Administración**: Consulta la guía detallada [COOLIFY_DEPLOY_GUIDE.md](file:///Users/miguel/Documents/GitHub/ecoalertapasco/COOLIFY_DEPLOY_GUIDE.md) para aprender a desplegar en **Coolify**, realizar pruebas en **Multipass VPS (Mac Intel)**, consultar logs y realizar copias de seguridad de la base de datos.

