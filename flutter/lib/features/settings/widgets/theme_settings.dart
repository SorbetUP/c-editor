import 'package:flutter/material.dart';

import 'package:c_editor_flutter/models/models.dart';

class ThemeSettings extends StatelessWidget {
  final AppConfig config;

  const ThemeSettings({
    super.key,
    required this.config,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        ListTile(
          title: const Text('Mode sombre'),
          subtitle: const Text('Thème sombre avec fond #202124'),
          leading: const Icon(Icons.dark_mode),
          trailing: Switch(
            value: true, // Always dark for now
            onChanged: null, // Disabled for now
          ),
        ),
        
        ListTile(
          title: const Text('Couleurs d\'accentuation'),
          subtitle: const Text('Bleu (#8AB4F8)'),
          leading: Container(
            width: 24,
            height: 24,
            decoration: const BoxDecoration(
              color: Color(0xFF8AB4F8),
              shape: BoxShape.circle,
            ),
          ),
          trailing: const Icon(Icons.chevron_right),
          onTap: () {
            // TODO: Implement color picker
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(
                content: Text('Sélection de couleurs bientôt disponible'),
              ),
            );
          },
        ),

        const ListTile(
          title: Text('Couleurs personnalisées'),
          subtitle: Text('Personnaliser les couleurs de surlignage et soulignement'),
          leading: Icon(Icons.color_lens),
          trailing: Icon(Icons.chevron_right),
        ),
      ],
    );
  }
}