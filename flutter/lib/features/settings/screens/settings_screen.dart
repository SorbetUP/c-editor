import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:c_editor_flutter/models/models.dart';

import '../../../core/routing/app_router.dart';
import '../../../core/state/app_state.dart';
import '../../../shared/widgets/loading_widget.dart';
import '../widgets/settings_section.dart';
import '../widgets/font_settings.dart';
import '../widgets/theme_settings.dart';
import '../widgets/language_settings.dart';
import '../widgets/storage_settings.dart';
import '../widgets/about_section.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final appState = ref.watch(appStateProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Paramètres'),
        leading: IconButton(
          onPressed: () => AppNavigation.toHome(context),
          icon: const Icon(Icons.arrow_back),
        ),
      ),
      body: appState.when(
        loading: () => const LoadingWidget(),
        error: (error, _) => _buildErrorView(context, error.toString()),
        data: (config) => _buildSettingsView(context, config),
      ),
    );
  }

  Widget _buildErrorView(BuildContext context, String error) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(
            Icons.error_outline,
            size: 64,
            color: Colors.red,
          ),
          const SizedBox(height: 16),
          Text(
            'Erreur de configuration',
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          const SizedBox(height: 8),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 32),
            child: Text(
              error,
              style: Theme.of(context).textTheme.bodyMedium,
              textAlign: TextAlign.center,
            ),
          ),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: () => AppNavigation.toHome(context),
            child: const Text('Retour'),
          ),
        ],
      ),
    );
  }

  Widget _buildSettingsView(BuildContext context, AppConfig config) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Font & Typography
          SettingsSection(
            title: 'Police et typographie',
            icon: Icons.text_fields,
            children: [
              FontSettings(config: config),
            ],
          ),

          const SizedBox(height: 24),

          // Theme
          SettingsSection(
            title: 'Thème',
            icon: Icons.palette,
            children: [
              ThemeSettings(config: config),
            ],
          ),

          const SizedBox(height: 24),

          // Language
          SettingsSection(
            title: 'Langue',
            icon: Icons.language,
            children: [
              LanguageSettings(config: config),
            ],
          ),

          const SizedBox(height: 24),

          // Storage
          SettingsSection(
            title: 'Stockage',
            icon: Icons.storage,
            children: [
              StorageSettings(config: config),
            ],
          ),

          const SizedBox(height: 24),

          // Performance (Web specific)
          SettingsSection(
            title: 'Performance',
            icon: Icons.speed,
            children: [
              SwitchListTile(
                title: const Text('CanvasKit Web'),
                subtitle: const Text(
                  'Améliore les performances sur le web (nécessite un redémarrage)',
                ),
                value: config.enableCanvasKitWeb,
                onChanged: (value) => _updateConfig(
                  context,
                  config.copyWith(enableCanvasKitWeb: value),
                ),
              ),
            ],
          ),

          const SizedBox(height: 24),

          // About
          const AboutSection(),

          const SizedBox(height: 32),
        ],
      ),
    );
  }

  void _updateConfig(BuildContext context, AppConfig newConfig) async {
    final container = ProviderScope.containerOf(context);
    await container.read(appStateProvider.notifier).updateConfig(newConfig);
  }
}