import 'package:latlong2/latlong.dart';

enum WasteType {
  general,
  organic,
  recyclable,
  municipalDump,
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
  final String? imageUrl;

  const WastePoint({
    required this.id,
    required this.name,
    required this.description,
    required this.location,
    required this.type,
    required this.fillLevel,
    required this.truckSchedule,
    this.imageUrl,
  });

  factory WastePoint.fromJson(Map<String, dynamic> json) {
    WasteType type;
    switch (json['type']) {
      case 'recyclable':
        type = WasteType.recyclable;
        break;
      case 'organic':
        type = WasteType.organic;
        break;
      case 'municipalDump':
        type = WasteType.municipalDump;
        break;
      default:
        type = WasteType.general;
    }

    FillLevel fillLevel;
    switch (json['fill_level']) {
      case 'low':
        fillLevel = FillLevel.low;
        break;
      case 'full':
        fillLevel = FillLevel.full;
        break;
      default:
        fillLevel = FillLevel.medium;
    }

    return WastePoint(
      id: json['id'].toString(),
      name: json['name'] ?? '',
      description: json['description'] ?? '',
      location: LatLng(
        (json['latitude'] as num).toDouble(),
        (json['longitude'] as num).toDouble(),
      ),
      type: type,
      fillLevel: fillLevel,
      truckSchedule: json['schedule'] ?? '',
      imageUrl: json['image'],
    );
  }
}
