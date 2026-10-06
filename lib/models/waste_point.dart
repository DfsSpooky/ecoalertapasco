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
  final String address;
  final LatLng location;
  final WasteType type;
  final FillLevel fillLevel;
  final String truckSchedule;
  final String imageUrl;
  final String recommendations;

  const WastePoint({
    required this.id,
    required this.name,
    required this.description,
    this.address = '',
    required this.location,
    required this.type,
    this.fillLevel = FillLevel.low,
    required this.truckSchedule,
    this.imageUrl = '',
    this.recommendations = '',
  });

  factory WastePoint.fromJson(Map<String, dynamic> json) {
    return WastePoint(
      id: json['id']?.toString() ?? '',
      name: json['name'] ?? '',
      description: json['description'] ?? '',
      address: json['address'] ?? '',
      location: LatLng(
        (json['latitude'] as num).toDouble(),
        (json['longitude'] as num).toDouble(),
      ),
      type: WasteType.values.firstWhere(
        (e) => e.toString().split('.').last == json['type'],
        orElse: () => WasteType.general,
      ),
      fillLevel: FillLevel.values.firstWhere(
        (e) => e.toString().split('.').last == json['fill_level'],
        orElse: () => FillLevel.low,
      ),
      truckSchedule: json['truck_schedule'] ?? '',
      imageUrl: json['image_url'] ?? '',
      recommendations: json['recommendations'] ?? '',
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'description': description,
      'address': address,
      'latitude': location.latitude,
      'longitude': location.longitude,
      'type': type.toString().split('.').last,
      'fill_level': fillLevel.toString().split('.').last,
      'truck_schedule': truckSchedule,
      'image_url': imageUrl,
      'recommendations': recommendations,
    };
  }
}
