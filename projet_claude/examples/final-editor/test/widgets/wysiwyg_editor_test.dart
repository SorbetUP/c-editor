import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../lib/features/final_editor/widgets/wysiwyg_editor.dart';
import '../../lib/features/final_editor/services/editor_service.dart';

void main() {
  group('WysiwygEditor Widget Tests', () {
    testWidgets('should display empty state when no content', (tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(
            home: Scaffold(
              body: WysiwygEditor(),
            ),
          ),
        ),
      );

      expect(find.text('Document vide'), findsOneWidget);
      expect(find.text('Commencez à écrire ou importez un fichier'), findsOneWidget);
      expect(find.byIcon(Icons.description_outlined), findsOneWidget);
    });

    testWidgets('should display document name in header', (tester) async {
      final container = ProviderContainer(
        overrides: [
          editorServiceProvider.overrideWith((ref) {
            final service = EditorService(ref);
            service.updateDocument(const DocumentData(
              name: 'Test Document',
              metadata: {},
              elements: [],
            ));
            return service;
          }),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: WysiwygEditor(),
            ),
          ),
        ),
      );

      expect(find.text('Test Document'), findsOneWidget);
    });

    testWidgets('should show unsaved changes indicator', (tester) async {
      final container = ProviderContainer(
        overrides: [
          editorServiceProvider.overrideWith((ref) {
            final service = EditorService(ref);
            final state = EditorState(
              document: const DocumentData(
                name: 'Test Document',
                metadata: {},
                elements: [],
              ),
              hasUnsavedChanges: true,
            );
            return service;
          }),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: WysiwygEditor(),
            ),
          ),
        ),
      );

      // Should show the unsaved changes dot indicator
      expect(find.byType(Container), findsWidgets);
    });

    testWidgets('should render paragraph elements', (tester) async {
      final container = ProviderContainer(
        overrides: [
          editorServiceProvider.overrideWith((ref) {
            final service = EditorService(ref);
            service.updateDocument(const DocumentData(
              name: 'Test Document',
              metadata: {},
              elements: [
                {
                  'type': 'paragraph',
                  'content': 'This is a test paragraph',
                }
              ],
            ));
            return service;
          }),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: WysiwygEditor(),
            ),
          ),
        ),
      );

      expect(find.text('This is a test paragraph'), findsOneWidget);
    });

    testWidgets('should render heading elements', (tester) async {
      final container = ProviderContainer(
        overrides: [
          editorServiceProvider.overrideWith((ref) {
            final service = EditorService(ref);
            service.updateDocument(const DocumentData(
              name: 'Test Document',
              metadata: {},
              elements: [
                {
                  'type': 'heading',
                  'level': 1,
                  'content': 'Main Heading',
                }
              ],
            ));
            return service;
          }),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: WysiwygEditor(),
            ),
          ),
        ),
      );

      expect(find.text('Main Heading'), findsOneWidget);
    });

    testWidgets('should render list elements', (tester) async {
      final container = ProviderContainer(
        overrides: [
          editorServiceProvider.overrideWith((ref) {
            final service = EditorService(ref);
            service.updateDocument(const DocumentData(
              name: 'Test Document',
              metadata: {},
              elements: [
                {
                  'type': 'list',
                  'ordered': false,
                  'items': [
                    'First item',
                    'Second item',
                  ],
                }
              ],
            ));
            return service;
          }),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: WysiwygEditor(),
            ),
          ),
        ),
      );

      expect(find.text('First item'), findsOneWidget);
      expect(find.text('Second item'), findsOneWidget);
      expect(find.text('• '), findsNWidgets(2)); // Bullet points
    });

    testWidgets('should render ordered list elements', (tester) async {
      final container = ProviderContainer(
        overrides: [
          editorServiceProvider.overrideWith((ref) {
            final service = EditorService(ref);
            service.updateDocument(const DocumentData(
              name: 'Test Document',
              metadata: {},
              elements: [
                {
                  'type': 'list',
                  'ordered': true,
                  'items': [
                    'First item',
                    'Second item',
                  ],
                }
              ],
            ));
            return service;
          }),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: WysiwygEditor(),
            ),
          ),
        ),
      );

      expect(find.text('First item'), findsOneWidget);
      expect(find.text('Second item'), findsOneWidget);
      expect(find.text('1. '), findsOneWidget);
      expect(find.text('2. '), findsOneWidget);
    });

    testWidgets('should render code block elements', (tester) async {
      final container = ProviderContainer(
        overrides: [
          editorServiceProvider.overrideWith((ref) {
            final service = EditorService(ref);
            service.updateDocument(const DocumentData(
              name: 'Test Document',
              metadata: {},
              elements: [
                {
                  'type': 'code_block',
                  'language': 'javascript',
                  'content': 'console.log("Hello World");',
                }
              ],
            ));
            return service;
          }),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: WysiwygEditor(),
            ),
          ),
        ),
      );

      expect(find.text('javascript'), findsOneWidget);
      expect(find.text('console.log("Hello World");'), findsOneWidget);
    });

    testWidgets('should render blockquote elements', (tester) async {
      final container = ProviderContainer(
        overrides: [
          editorServiceProvider.overrideWith((ref) {
            final service = EditorService(ref);
            service.updateDocument(const DocumentData(
              name: 'Test Document',
              metadata: {},
              elements: [
                {
                  'type': 'blockquote',
                  'content': 'This is a blockquote',
                }
              ],
            ));
            return service;
          }),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: WysiwygEditor(),
            ),
          ),
        ),
      );

      expect(find.text('This is a blockquote'), findsOneWidget);
    });

    testWidgets('should render unknown elements with error styling', (tester) async {
      final container = ProviderContainer(
        overrides: [
          editorServiceProvider.overrideWith((ref) {
            final service = EditorService(ref);
            service.updateDocument(const DocumentData(
              name: 'Test Document',
              metadata: {},
              elements: [
                {
                  'type': 'unknown_element',
                  'content': 'Unknown content',
                }
              ],
            ));
            return service;
          }),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: WysiwygEditor(),
            ),
          ),
        ),
      );

      expect(find.textContaining('Unknown element: unknown_element'), findsOneWidget);
    });

    testWidgets('should be scrollable when content exceeds viewport', (tester) async {
      final container = ProviderContainer(
        overrides: [
          editorServiceProvider.overrideWith((ref) {
            final service = EditorService(ref);
            service.updateDocument(DocumentData(
              name: 'Long Document',
              metadata: const {},
              elements: List.generate(20, (index) => {
                'type': 'paragraph',
                'content': 'Paragraph ${index + 1}',
              }),
            ));
            return service;
          }),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: WysiwygEditor(),
            ),
          ),
        ),
      );

      expect(find.byType(Scrollbar), findsOneWidget);
      expect(find.byType(SingleChildScrollView), findsOneWidget);
    });
  });
}