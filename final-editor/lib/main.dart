import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'features/final_editor/screens/final_editor_screen.dart';
import 'core/wasm_bridge.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Initialize WASM bridge
  await WasmBridge.initialize();
  
  runApp(const ProviderScope(child: FinalEditorApp()));
}

class FinalEditorApp extends StatelessWidget {
  const FinalEditorApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      title: 'Final Editor',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF1976D2),
          brightness: Brightness.light,
        ),
        useMaterial3: true,
        fontFamily: 'RobotoMono',
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF1976D2),
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
        fontFamily: 'RobotoMono',
      ),
      routerConfig: _router,
    );
  }
}

final _router = GoRouter(
  initialLocation: '/final',
  routes: [
    GoRoute(
      path: '/final',
      name: 'final-editor',
      builder: (context, state) => const FinalEditorScreen(),
    ),
    GoRoute(
      path: '/',
      redirect: (context, state) => '/final',
    ),
  ],
);