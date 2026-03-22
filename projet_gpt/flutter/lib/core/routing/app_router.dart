import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../features/home/screens/home_screen.dart';
import '../../features/viewer/screens/viewer_screen.dart';
import '../../features/settings/screens/settings_screen.dart';
import '../../features/editor/screens/editor_screen.dart';
// Playground example moved to examples/ directory
import '../state/app_state.dart';

// Routes
class AppRoutes {
  static const String home = '/';
  static const String viewer = '/viewer';
  static const String editor = '/editor';
  static const String settings = '/settings';
}

final appRouterProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    initialLocation: AppRoutes.home,
    debugLogDiagnostics: false,
    routes: [
      GoRoute(
        path: AppRoutes.home,
        name: 'home',
        builder: (context, state) => const HomeScreen(),
      ),
      GoRoute(
        path: AppRoutes.viewer,
        name: 'viewer',
        builder: (context, state) {
          // Extract note path from query parameters
          final notePath = state.uri.queryParameters['path'];
          return ViewerScreen(notePath: notePath);
        },
      ),
      GoRoute(
        path: AppRoutes.editor,
        name: 'editor',
        builder: (context, state) {
          // Extract document path from query parameters
          final documentPath = state.uri.queryParameters['path'];
          return EditorScreen(documentPath: documentPath);
        },
      ),
      GoRoute(
        path: AppRoutes.settings,
        name: 'settings',
        builder: (context, state) => const SettingsScreen(),
      ),
    ],
    errorBuilder: (context, state) => const ErrorScreen(),
  );
});

class ErrorScreen extends StatelessWidget {
  const ErrorScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF202124),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(
              Icons.error_outline,
              color: Colors.red,
              size: 64,
            ),
            const SizedBox(height: 16),
            Text(
              'Page non trouvée',
              style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                color: Colors.white,
              ),
            ),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: () => context.go(AppRoutes.home),
              child: const Text('Retour à l\'accueil'),
            ),
          ],
        ),
      ),
    );
  }
}

// Navigation helpers
class AppNavigation {
  static void toHome(BuildContext context) {
    context.go(AppRoutes.home);
  }

  static void toViewer(BuildContext context, {String? notePath}) {
    final uri = Uri(
      path: AppRoutes.viewer,
      queryParameters: notePath != null ? {'path': notePath} : null,
    );
    context.go(uri.toString());
  }

  static void toEditor(BuildContext context, {String? documentPath}) {
    final uri = Uri(
      path: AppRoutes.editor,
      queryParameters: documentPath != null ? {'path': documentPath} : null,
    );
    context.go(uri.toString());
  }

  static void toSettings(BuildContext context) {
    context.go(AppRoutes.settings);
  }
}