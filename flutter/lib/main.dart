import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
// import 'package:flutter_localizations/flutter_localizations.dart'; // Commented for CI compatibility

import 'shared/theme/app_theme.dart';
import 'core/routing/app_router.dart';
// import 'shared/i18n/app_localizations.dart'; // Commented for CI compatibility

void main() {
  runApp(const ProviderScope(child: NoteEditorApp()));
}

class NoteEditorApp extends ConsumerWidget {
  const NoteEditorApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(appRouterProvider);
    
    return MaterialApp.router(
      title: 'Note Editor',
      theme: AppTheme.darkTheme,
      debugShowCheckedModeBanner: false,
      
      // Routing
      routerConfig: router,
      
      // Internationalization (commented for CI compatibility)
      // localizationsDelegates: const [
      //   AppLocalizations.delegate,
      //   GlobalMaterialLocalizations.delegate,
      //   GlobalWidgetsLocalizations.delegate,
      //   GlobalCupertinoLocalizations.delegate,
      // ],
      // supportedLocales: const [
      //   Locale('fr', ''),
      //   Locale('en', ''),
      // ],
    );
  }
}