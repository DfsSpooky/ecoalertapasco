enum EcoCategory {
  mineria,
  basura,
  agua,
  aire,
}

enum EcoSeverity {
  critico,
  medio,
  bajo,
}

enum EcoDistrict {
  chaupimarca,
  yanacancha,
  simonBolivar,
}

class EcoAlert {
  final String id;
  final String title;
  final String description;
  final EcoCategory category;
  final EcoSeverity severity;
  final EcoDistrict district;
  final String address;
  final double latitude;
  final double longitude;
  final DateTime createdAt;
  final String? imageUrl;
  
  // Campos de moderación y resolución por autoridades
  final String status; // 'pending', 'verified', 'solved'
  final String? resolutionComment;
  final String? resolutionImageUrl;
  final DateTime? resolvedAt;
  final String? historyLog;

  EcoAlert({
    required this.id,
    required this.title,
    required this.description,
    required this.category,
    required this.severity,
    required this.district,
    required this.address,
    required this.latitude,
    required this.longitude,
    required this.createdAt,
    this.imageUrl,
    this.status = 'pending',
    this.resolutionComment,
    this.resolutionImageUrl,
    this.resolvedAt,
    this.historyLog,
  });

  EcoAlert copyWith({
    String? id,
    String? title,
    String? description,
    EcoCategory? category,
    EcoSeverity? severity,
    EcoDistrict? district,
    String? address,
    double? latitude,
    double? longitude,
    DateTime? createdAt,
    String? imageUrl,
    String? status,
    String? resolutionComment,
    String? resolutionImageUrl,
    DateTime? resolvedAt,
    String? historyLog,
  }) {
    return EcoAlert(
      id: id ?? this.id,
      title: title ?? this.title,
      description: description ?? this.description,
      category: category ?? this.category,
      severity: severity ?? this.severity,
      district: district ?? this.district,
      address: address ?? this.address,
      latitude: latitude ?? this.latitude,
      longitude: longitude ?? this.longitude,
      createdAt: createdAt ?? this.createdAt,
      imageUrl: imageUrl ?? this.imageUrl,
      status: status ?? this.status,
      resolutionComment: resolutionComment ?? this.resolutionComment,
      resolutionImageUrl: resolutionImageUrl ?? this.resolutionImageUrl,
      resolvedAt: resolvedAt ?? this.resolvedAt,
      historyLog: historyLog ?? this.historyLog,
    );
  }

  factory EcoAlert.fromJson(Map<String, dynamic> json) {
    double lat = 0.0;
    double lng = 0.0;
    
    if (json['latitude'] != null && json['longitude'] != null) {
      lat = (json['latitude'] as num).toDouble();
      lng = (json['longitude'] as num).toDouble();
    } else if (json['location'] != null && json['location'] is Map) {
      final loc = json['location'] as Map;
      if (loc['coordinates'] != null && loc['coordinates'] is List) {
        final coords = loc['coordinates'] as List;
        lng = (coords[0] as num).toDouble();
        lat = (coords[1] as num).toDouble();
      }
    }

    return EcoAlert(
      id: json['id']?.toString() ?? '',
      title: json['title'] ?? '',
      description: json['description'] ?? '',
      category: EcoCategory.values.firstWhere(
        (e) => e.toString().split('.').last == json['category'],
        orElse: () => EcoCategory.mineria,
      ),
      severity: EcoSeverity.values.firstWhere(
        (e) => e.toString().split('.').last == json['severity'],
        orElse: () => EcoSeverity.bajo,
      ),
      district: EcoDistrict.values.firstWhere(
        (e) => e.toString().split('.').last == json['district'],
        orElse: () => EcoDistrict.chaupimarca,
      ),
      address: json['address'] ?? 'Calle de Cerro de Pasco, Perú',
      latitude: lat,
      longitude: lng,
      createdAt: json['created_at'] != null 
          ? DateTime.parse(json['created_at']) 
          : DateTime.now(),
      imageUrl: json['image_url'],
      status: json['status'] ?? 'pending',
      resolutionComment: json['resolution_comment'],
      resolutionImageUrl: json['resolution_image_url'],
      resolvedAt: json['resolved_at'] != null ? DateTime.parse(json['resolved_at']) : null,
      historyLog: json['history_log'],
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'title': title,
      'description': description,
      'category': category.toString().split('.').last,
      'severity': severity.toString().split('.').last,
      'district': district.toString().split('.').last,
      'address': address,
      'latitude': latitude,
      'longitude': longitude,
      'created_at': createdAt.toIso8601String(),
      'image_url': imageUrl,
      'status': status,
      'resolution_comment': resolutionComment,
      'resolution_image_url': resolutionImageUrl,
      'resolved_at': resolvedAt?.toIso8601String(),
      'history_log': historyLog,
    };
  }

  List<Map<String, String>> get parsedHistory {
    if (historyLog == null || historyLog!.trim().isEmpty) return [];
    return historyLog!.split('\n').where((line) => line.contains('|')).map((line) {
      final parts = line.split('|');
      final date = parts[0];
      final action = parts.sublist(1).join('|');
      return {'date': date, 'action': action};
    }).toList();
  }
}
