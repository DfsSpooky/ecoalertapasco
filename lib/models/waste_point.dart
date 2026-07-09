import 'package:latlong2/latlong.dart';

enum WasteType {
  general,
  organic,
  recyclable,
}

enum FillLevel {
  low,
  medium,
  full,
}

class WastePoint {
  final String id;
  final String name;
  final String description;
  final LatLng location;
  final WasteType type;
  final FillLevel fillLevel;
  final String truckSchedule;

  const WastePoint({
    required this.id,
    required this.name,
    required this.description,
    required this.location,
    required this.type,
    required this.fillLevel,
    required this.truckSchedule,
  });
}
