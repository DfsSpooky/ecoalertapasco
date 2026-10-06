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
  final String? imageUrl2;
  
  // Campos de moderación y resolución por autoridades
  final String status; // 'pending', 'verified', 'solved'
  final String? resolutionComment;
  final String? resolutionImageUrl;
  final DateTime? resolvedAt;
  final String? historyLog;
  final String reporterName;
  final int confirmationsCount;
  final bool hasUserConfirmed;

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
    this.imageUrl2,
    this.status = 'pending',
    this.resolutionComment,
    this.resolutionImageUrl,
    this.resolvedAt,
    this.historyLog,
    this.reporterName = 'Ciudadano',
    this.confirmationsCount = 0,
    this.hasUserConfirmed = false,
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
    String? imageUrl2,
    String? status,
    String? resolutionComment,
    String? resolutionImageUrl,
    DateTime? resolvedAt,
    String? historyLog,
    String? reporterName,
    int? confirmationsCount,
    bool? hasUserConfirmed,
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
      imageUrl2: imageUrl2 ?? this.imageUrl2,
      status: status ?? this.status,
      resolutionComment: resolutionComment ?? this.resolutionComment,
      resolutionImageUrl: resolutionImageUrl ?? this.resolutionImageUrl,
      resolvedAt: resolvedAt ?? this.resolvedAt,
      historyLog: historyLog ?? this.historyLog,
      reporterName: reporterName ?? this.reporterName,
      confirmationsCount: confirmationsCount ?? this.confirmationsCount,
      hasUserConfirmed: hasUserConfirmed ?? this.hasUserConfirmed,
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
      imageUrl2: json['image_url_2'],
      status: json['status'] ?? 'pending',
      resolutionComment: json['resolution_comment'],
      resolutionImageUrl: json['resolution_image_url'],
      resolvedAt: json['resolved_at'] != null ? DateTime.parse(json['resolved_at']) : null,
      historyLog: json['history_log'],
      reporterName: json['reporter_name'] ?? 'Ciudadano',
      confirmationsCount: json['confirmations_count'] is int ? json['confirmations_count'] : 0,
      hasUserConfirmed: json['has_user_confirmed'] == true,
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
      'image_url_2': imageUrl2,
      'status': status,
      'resolution_comment': resolutionComment,
      'resolution_image_url': resolutionImageUrl,
      'resolved_at': resolvedAt?.toIso8601String(),
      'history_log': historyLog,
      'reporter_name': reporterName,
      'confirmations_count': confirmationsCount,
      'has_user_confirmed': hasUserConfirmed,
    };
  }

  List<String> get evidenceImages {
    final list = <String>[];
    if (imageUrl != null && imageUrl!.trim().isNotEmpty) list.add(imageUrl!.trim());
    if (imageUrl2 != null && imageUrl2!.trim().isNotEmpty) list.add(imageUrl2!.trim());
    return list;
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
