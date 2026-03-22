import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/state/app_state.dart';
import 'package:c_editor_flutter/models/models.dart';

class FontSettings extends ConsumerWidget {
  final AppConfig config;

  const FontSettings({
    super.key,
    required this.config,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Column(
      children: [
        ListTile(
          title: const Text('Taille de police de base'),
          subtitle: Text('${config.fontSize.toInt()}px'),
          trailing: SizedBox(
            width: 200,
            child: Slider(
              value: config.fontSize,
              min: 12.0,
              max: 24.0,
              divisions: 12,
              label: '${config.fontSize.toInt()}px',
              onChanged: (value) => _updateFontSize(ref, value),
            ),
          ),
        ),
        
        ExpansionTile(
          title: const Text('Tailles des titres'),
          children: [
            for (int level = 1; level <= 6; level++)
              ListTile(
                title: Text('Titre H$level'),
                subtitle: Text('${config.headerSizes[level]?.toInt() ?? 16}px'),
                trailing: SizedBox(
                  width: 200,
                  child: Slider(
                    value: config.headerSizes[level] ?? 16.0,
                    min: 14.0,
                    max: 36.0,
                    divisions: 22,
                    label: '${config.headerSizes[level]?.toInt() ?? 16}px',
                    onChanged: (value) => _updateHeaderSize(ref, level, value),
                  ),
                ),
              ),
          ],
        ),

        ListTile(
          title: const Text('Alignement par défaut'),
          trailing: DropdownButton<TextAlign>(
            value: config.defaultAlignment,
            onChanged: (value) {
              if (value != null) {
                _updateAlignment(ref, value);
              }
            },
            items: const [
              DropdownMenuItem(
                value: TextAlign.left,
                child: Text('Gauche'),
              ),
              DropdownMenuItem(
                value: TextAlign.center,
                child: Text('Centre'),
              ),
              DropdownMenuItem(
                value: TextAlign.right,
                child: Text('Droite'),
              ),
              DropdownMenuItem(
                value: TextAlign.justify,
                child: Text('Justifié'),
              ),
            ],
          ),
        ),
      ],
    );
  }

  void _updateFontSize(WidgetRef ref, double fontSize) {
    final newConfig = config.copyWith(fontSize: fontSize);
    ref.read(appStateProvider.notifier).updateConfig(newConfig);
  }

  void _updateHeaderSize(WidgetRef ref, int level, double size) {
    final newSizes = Map<int, double>.from(config.headerSizes);
    newSizes[level] = size;
    final newConfig = config.copyWith(headerSizes: newSizes);
    ref.read(appStateProvider.notifier).updateConfig(newConfig);
  }

  void _updateAlignment(WidgetRef ref, TextAlign alignment) {
    final newConfig = config.copyWith(defaultAlignment: alignment);
    ref.read(appStateProvider.notifier).updateConfig(newConfig);
  }
}