import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_localizations/flutter_localizations.dart';

import 'shared/theme/app_theme.dart';
import 'core/routing/app_router.dart';
import 'shared/i18n/app_localizations.dart';

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
      
      // Internationalization
      localizationsDelegates: const [
        AppLocalizations.delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      supportedLocales: const [
        Locale('fr', ''),
        Locale('en', ''),
      ],
    );
  }
}