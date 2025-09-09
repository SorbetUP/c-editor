import 'package:flutter/material.dart';

class AppConfig {
  final String defaultNotesPath;
  final Locale locale;
  final double fontSize;
  final Map<int, double> headerSizes;
  final TextAlign defaultAlignment;
  final List<String> recentNotes;
  final bool enableCanvasKitWeb;

  const AppConfig({
    required this.defaultNotesPath,
    this.locale = const Locale('fr', ''),
    this.fontSize = 16.0,
    this.headerSizes = const {
      1: 32.0,
      2: 28.0,
      3: 24.0,
      4: 20.0,
      5: 18.0,
      6: 16.0,
    },
    this.defaultAlignment = TextAlign.left,
    this.recentNotes = const [],
    this.enableCanvasKitWeb = true,
  });

  AppConfig copyWith({
    String? defaultNotesPath,
    Locale? locale,
    double? fontSize,
    Map<int, double>? headerSizes,
    TextAlign? defaultAlignment,
    List<String>? recentNotes,
    bool? enableCanvasKitWeb,
  }) {
    return AppConfig(
      defaultNotesPath: defaultNotesPath ?? this.defaultNotesPath,
      locale: locale ?? this.locale,
      fontSize: fontSize ?? this.fontSize,
      headerSizes: headerSizes ?? this.headerSizes,
      defaultAlignment: defaultAlignment ?? this.defaultAlignment,
      recentNotes: recentNotes ?? this.recentNotes,
      enableCanvasKitWeb: enableCanvasKitWeb ?? this.enableCanvasKitWeb,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'defaultNotesPath': defaultNotesPath,
      'locale': '${locale.languageCode}_${locale.countryCode}',
      'fontSize': fontSize,
      'headerSizes': headerSizes.map((k, v) => MapEntry(k.toString(), v)),
      'defaultAlignment': defaultAlignment.index,
      'recentNotes': recentNotes,
      'enableCanvasKitWeb': enableCanvasKitWeb,
    };
  }

  factory AppConfig.fromJson(Map<String, dynamic> json) {
    final localeStr = json['locale'] as String? ?? 'fr_';
    final localeParts = localeStr.split('_');
    
    return AppConfig(
      defaultNotesPath: json['defaultNotesPath'] as String? ?? '',
      locale: Locale(
        localeParts.isNotEmpty ? localeParts[0] : 'fr',
        localeParts.length > 1 ? localeParts[1] : '',
      ),
      fontSize: (json['fontSize'] as num?)?.toDouble() ?? 16.0,
      headerSizes: (json['headerSizes'] as Map<String, dynamic>?)?.map(
        (k, v) => MapEntry(int.parse(k), (v as num).toDouble()),
      ) ?? const {
        1: 32.0, 2: 28.0, 3: 24.0, 4: 20.0, 5: 18.0, 6: 16.0,
      },
      defaultAlignment: TextAlign.values[json['defaultAlignment'] as int? ?? 0],
      recentNotes: (json['recentNotes'] as List?)?.cast<String>() ?? [],
      enableCanvasKitWeb: json['enableCanvasKitWeb'] as bool? ?? true,
    );
  }

  static AppConfig get defaultConfig => AppConfig(
    defaultNotesPath: '',
    locale: const Locale('fr', ''),
    fontSize: 16.0,
    headerSizes: const {
      1: 32.0,
      2: 28.0,
      3: 24.0,
      4: 20.0,
      5: 18.0,
      6: 16.0,
    },
    defaultAlignment: TextAlign.left,
    recentNotes: const [],
    enableCanvasKitWeb: true,
  );
}