import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:golden_toolkit/golden_toolkit.dart';

import '../../lib/features/final_editor/screens/final_editor_screen.dart';
import '../../lib/features/final_editor/widgets/wysiwyg_editor.dart';
import '../../lib/features/final_editor/widgets/editor_toolbar.dart';
import '../../lib/features/final_editor/widgets/markdown_panel.dart';
import '../../lib/features/final_editor/widgets/json_panel.dart';

void main() {
  group('Final Editor Golden Tests', () {
    setUpAll(() async {
      await loadAppFonts();
    });

    testGoldens('Final Editor Screen - Initial State', (tester) async {
      await tester.pumpWidgetBuilder(
        const ProviderScope(
          child: FinalEditorScreen(),
        ),
        surfaceSize: const Size(1200, 800),
        wrapper: materialAppWrapper(
          theme: ThemeData(
            colorScheme: ColorScheme.fromSeed(
              seedColor: const Color(0xFF1976D2),
              brightness: Brightness.light,
            ),
            useMaterial3: true,
            fontFamily: 'Roboto', // Use system font for golden tests
          ),
        ),
      );

      await screenMatchesGolden(tester, 'final_editor_initial');
    });

    testGoldens('Final Editor Screen - With Panels Open', (tester) async {
      await tester.pumpWidgetBuilder(
        const ProviderScope(
          child: FinalEditorScreen(),
        ),
        surfaceSize: const Size(1400, 900),
        wrapper: materialAppWrapper(
          theme: ThemeData(
            colorScheme: ColorScheme.fromSeed(
              seedColor: const Color(0xFF1976D2),
              brightness: Brightness.light,
            ),
            useMaterial3: true,
            fontFamily: 'Roboto',
          ),
        ),
      );

      // Open markdown and JSON panels
      await tester.tap(find.byTooltip('Toggle Markdown Panel'));
      await tester.pump();
      
      await tester.tap(find.byTooltip('Toggle JSON Panel'));
      await tester.pump();

      await screenMatchesGolden(tester, 'final_editor_with_panels');
    });

    testGoldens('Editor Toolbar - All States', (tester) async {
      await tester.pumpWidgetBuilder(
        EditorToolbar(
          onImportMarkdown: () {},
          onImportJson: () {},
          onExportMarkdown: () {},
          onExportJson: () {},
          onToggleMarkdown: () {},
          onToggleJson: () {},
          showMarkdownPanel: true,
          showJsonPanel: true,
        ),
        surfaceSize: const Size(1000, 100),
        wrapper: materialAppWrapper(
          theme: ThemeData(
            colorScheme: ColorScheme.fromSeed(
              seedColor: const Color(0xFF1976D2),
              brightness: Brightness.light,
            ),
            useMaterial3: true,
            fontFamily: 'Roboto',
          ),
        ),
      );

      await screenMatchesGolden(tester, 'editor_toolbar_active');
    });

    testGoldens('WYSIWYG Editor - Empty State', (tester) async {
      await tester.pumpWidgetBuilder(
        const ProviderScope(
          child: WysiwygEditor(),
        ),
        surfaceSize: const Size(600, 400),
        wrapper: materialAppWrapper(
          theme: ThemeData(
            colorScheme: ColorScheme.fromSeed(
              seedColor: const Color(0xFF1976D2),
              brightness: Brightness.light,
            ),
            useMaterial3: true,
            fontFamily: 'Roboto',
          ),
        ),
      );

      await screenMatchesGolden(tester, 'wysiwyg_editor_empty');
    });

    testGoldens('Markdown Panel', (tester) async {
      await tester.pumpWidgetBuilder(
        ProviderScope(
          child: MarkdownPanel(
            onClose: () {},
          ),
        ),
        surfaceSize: const Size(400, 600),
        wrapper: materialAppWrapper(
          theme: ThemeData(
            colorScheme: ColorScheme.fromSeed(
              seedColor: const Color(0xFF1976D2),
              brightness: Brightness.light,
            ),
            useMaterial3: true,
            fontFamily: 'Roboto',
          ),
        ),
      );

      await screenMatchesGolden(tester, 'markdown_panel');
    });

    testGoldens('JSON Panel', (tester) async {
      await tester.pumpWidgetBuilder(
        ProviderScope(
          child: JsonPanel(
            onClose: () {},
          ),
        ),
        surfaceSize: const Size(400, 600),
        wrapper: materialAppWrapper(
          theme: ThemeData(
            colorScheme: ColorScheme.fromSeed(
              seedColor: const Color(0xFF1976D2),
              brightness: Brightness.light,
            ),
            useMaterial3: true,
            fontFamily: 'Roboto',
          ),
        ),
      );

      await screenMatchesGolden(tester, 'json_panel');
    });

    testGoldens('JSON Panel - Error State', (tester) async {
      await tester.pumpWidgetBuilder(
        ProviderScope(
          child: JsonPanel(
            onClose: () {},
          ),
        ),
        surfaceSize: const Size(400, 600),
        wrapper: materialAppWrapper(
          theme: ThemeData(
            colorScheme: ColorScheme.fromSeed(
              seedColor: const Color(0xFF1976D2),
              brightness: Brightness.light,
            ),
            useMaterial3: true,
            fontFamily: 'Roboto',
          ),
        ),
      );

      // Enter invalid JSON to trigger error state
      final textField = find.byType(TextField);
      await tester.enterText(textField, '{ invalid json }');
      await tester.pump();

      await screenMatchesGolden(tester, 'json_panel_error');
    });

    testGoldens('Dark Theme - Final Editor Screen', (tester) async {
      await tester.pumpWidgetBuilder(
        const ProviderScope(
          child: FinalEditorScreen(),
        ),
        surfaceSize: const Size(1200, 800),
        wrapper: materialAppWrapper(
          theme: ThemeData(
            colorScheme: ColorScheme.fromSeed(
              seedColor: const Color(0xFF1976D2),
              brightness: Brightness.dark,
            ),
            useMaterial3: true,
            fontFamily: 'Roboto',
          ),
        ),
      );

      await screenMatchesGolden(tester, 'final_editor_dark_theme');
    });
  });

  group('Responsive Golden Tests', () {
    setUpAll(() async {
      await loadAppFonts();
    });

    testGoldens('Final Editor - Mobile Size', (tester) async {
      await tester.pumpWidgetBuilder(
        const ProviderScope(
          child: FinalEditorScreen(),
        ),
        surfaceSize: const Size(375, 667), // iPhone size
        wrapper: materialAppWrapper(
          theme: ThemeData(
            colorScheme: ColorScheme.fromSeed(
              seedColor: const Color(0xFF1976D2),
              brightness: Brightness.light,
            ),
            useMaterial3: true,
            fontFamily: 'Roboto',
          ),
        ),
      );

      await screenMatchesGolden(tester, 'final_editor_mobile');
    });

    testGoldens('Final Editor - Tablet Size', (tester) async {
      await tester.pumpWidgetBuilder(
        const ProviderScope(
          child: FinalEditorScreen(),
        ),
        surfaceSize: const Size(768, 1024), // iPad size
        wrapper: materialAppWrapper(
          theme: ThemeData(
            colorScheme: ColorScheme.fromSeed(
              seedColor: const Color(0xFF1976D2),
              brightness: Brightness.light,
            ),
            useMaterial3: true,
            fontFamily: 'Roboto',
          ),
        ),
      );

      await screenMatchesGolden(tester, 'final_editor_tablet');
    });

    testGoldens('Final Editor - Desktop Size', (tester) async {
      await tester.pumpWidgetBuilder(
        const ProviderScope(
          child: FinalEditorScreen(),
        ),
        surfaceSize: const Size(1920, 1080), // Desktop size
        wrapper: materialAppWrapper(
          theme: ThemeData(
            colorScheme: ColorScheme.fromSeed(
              seedColor: const Color(0xFF1976D2),
              brightness: Brightness.light,
            ),
            useMaterial3: true,
            fontFamily: 'Roboto',
          ),
        ),
      );

      await screenMatchesGolden(tester, 'final_editor_desktop');
    });
  });
}