import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../shared/widgets/settings_section.dart';

class AboutSection extends ConsumerWidget {
  const AboutSection({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return SettingsSection(
      title: 'À propos',
      icon: Icons.info,
      children: [
        ListTile(
          title: const Text('Version de l\'application'),
          subtitle: const Text('v1.1.0-app-shell'),
          leading: const Icon(Icons.app_settings_alt),
        ),
        
        ListTile(
          title: const Text('Version du moteur C'),
          subtitle: const Text('v1.0.1-tests'), // TODO: Get from C core
          leading: const Icon(Icons.code),
        ),

        ListTile(
          title: const Text('Plateforme'),
          subtitle: Text(_getPlatformName()),
          leading: const Icon(Icons.devices),
        ),

        ListTile(
          title: const Text('Fonctionnalités actives'),
          subtitle: const Text('0x1f'), // TODO: Get from C core
          leading: const Icon(Icons.featured_play_list),
        ),

        const Divider(),

        ListTile(
          title: const Text('Licences'),
          leading: const Icon(Icons.article),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => _showLicenses(context),
        ),

        ListTile(
          title: const Text('Code source'),
          subtitle: const Text('Voir le projet sur GitHub'),
          leading: const Icon(Icons.code),
          trailing: const Icon(Icons.open_in_new),
          onTap: () => _openGitHub(context),
        ),

        ListTile(
          title: const Text('Signaler un problème'),
          leading: const Icon(Icons.bug_report),
          trailing: const Icon(Icons.open_in_new),
          onTap: () => _reportIssue(context),
        ),
      ],
    );
  }

  String _getPlatformName() {
    // TODO: Get actual platform info
    return 'Flutter Desktop (macOS)';
  }

  void _showLicenses(BuildContext context) {
    showLicensePage(
      context: context,
      applicationName: 'Note Editor',
      applicationVersion: 'v1.1.0-app-shell',
      applicationLegalese: '© 2024 Note Editor. Développé avec Claude Code.',
    );
  }

  void _openGitHub(BuildContext context) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Ouverture du lien GitHub...'),
      ),
    );
    // TODO: Launch URL
  }

  void _reportIssue(BuildContext context) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Ouverture de la page de signalement...'),
      ),
    );
    // TODO: Launch URL
  }
}