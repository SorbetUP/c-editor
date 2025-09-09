import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/state/app_state.dart';
import 'package:c_editor_flutter/models/models.dart';

class LanguageSettings extends ConsumerWidget {
  final AppConfig config;

  const LanguageSettings({
    super.key,
    required this.config,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Column(
      children: [
        ListTile(
          title: const Text('Langue de l\'interface'),
          trailing: DropdownButton<String>(
            value: config.locale.languageCode,
            onChanged: (value) {
              if (value != null) {
                _updateLanguage(ref, value);
              }
            },
            items: const [
              DropdownMenuItem(
                value: 'fr',
                child: Text('Français'),
              ),
              DropdownMenuItem(
                value: 'en',
                child: Text('English'),
              ),
            ],
          ),
        ),
        
        ListTile(
          title: const Text('Format de date'),
          subtitle: Text(_getDateFormat(config.locale.languageCode)),
          trailing: const Icon(Icons.chevron_right),
          onTap: () {
            // TODO: Implement date format selection
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(
                content: Text('Sélection de format bientôt disponible'),
              ),
            );
          },
        ),

        ListTile(
          title: const Text('Région'),
          subtitle: Text(_getRegion(config.locale.languageCode)),
          trailing: const Icon(Icons.chevron_right),
          onTap: () {
            // TODO: Implement region selection
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(
                content: Text('Sélection de région bientôt disponible'),
              ),
            );
          },
        ),
      ],
    );
  }

  void _updateLanguage(WidgetRef ref, String languageCode) {
    final newLocale = Locale(languageCode, '');
    final newConfig = config.copyWith(locale: newLocale);
    ref.read(appStateProvider.notifier).updateConfig(newConfig);
  }

  String _getDateFormat(String languageCode) {
    switch (languageCode) {
      case 'fr':
        return 'JJ/MM/AAAA';
      case 'en':
        return 'MM/DD/YYYY';
      default:
        return 'JJ/MM/AAAA';
    }
  }

  String _getRegion(String languageCode) {
    switch (languageCode) {
      case 'fr':
        return 'France';
      case 'en':
        return 'États-Unis';
      default:
        return 'France';
    }
  }
}