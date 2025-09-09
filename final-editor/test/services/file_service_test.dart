import 'package:flutter_test/flutter_test.dart';
import '../../lib/features/final_editor/services/file_service.dart';

void main() {
  group('FileService Tests', () {
    late FileService fileService;
    
    setUp(() {
      fileService = FileService();
    });

    group('File Validation', () {
      test('should validate JSON content', () {
        const validJson = '{"name": "test", "elements": []}';
        const invalidJson = '{"name": "test", "elements": [';
        
        expect(fileService.validateFileContent(validJson, 'json'), isTrue);
        expect(fileService.validateFileContent(invalidJson, 'json'), isFalse);
      });

      test('should validate markdown content', () {
        const validMarkdown = '# Heading\n\nParagraph text';
        const emptyMarkdown = '';
        
        expect(fileService.validateFileContent(validMarkdown, 'markdown'), isTrue);
        expect(fileService.validateFileContent(emptyMarkdown, 'markdown'), isFalse);
      });

      test('should return false for unknown file types', () {
        const content = 'some content';
        
        expect(fileService.validateFileContent(content, 'unknown'), isFalse);
      });
    });

    group('MIME Types', () {
      test('should return correct MIME types', () {
        expect(fileService.getMimeType('json'), equals('application/json'));
        expect(fileService.getMimeType('markdown'), equals('text/markdown'));
        expect(fileService.getMimeType('md'), equals('text/markdown'));
        expect(fileService.getMimeType('unknown'), equals('text/plain'));
      });
    });

    group('Filename Suggestions', () {
      test('should suggest filename for JSON', () {
        expect(
          fileService.suggestFilename('My Document', 'json'),
          equals('my_document.json'),
        );
      });

      test('should suggest filename for Markdown', () {
        expect(
          fileService.suggestFilename('Test Article', 'markdown'),
          equals('test_article.md'),
        );
        
        expect(
          fileService.suggestFilename('Test Article', 'md'),
          equals('test_article.md'),
        );
      });

      test('should clean special characters from filename', () {
        expect(
          fileService.suggestFilename('My Doc@#$%^&*()', 'json'),
          equals('my_doc.json'),
        );
      });

      test('should handle empty document names', () {
        expect(
          fileService.suggestFilename('', 'json'),
          equals('document.json'),
        );
      });

      test('should replace spaces with underscores', () {
        expect(
          fileService.suggestFilename('My Long Document Name', 'markdown'),
          equals('my_long_document_name.md'),
        );
      });

      test('should handle unknown file types', () {
        expect(
          fileService.suggestFilename('Test', 'unknown'),
          equals('test.txt'),
        );
      });
    });

    group('Case Insensitive Type Handling', () {
      test('should handle uppercase file types', () {
        expect(fileService.getMimeType('JSON'), equals('application/json'));
        expect(fileService.getMimeType('MARKDOWN'), equals('text/markdown'));
        
        expect(fileService.validateFileContent('{"test": true}', 'JSON'), isTrue);
        expect(fileService.validateFileContent('# Title', 'MARKDOWN'), isTrue);
        
        expect(
          fileService.suggestFilename('Test', 'JSON'),
          equals('test.json'),
        );
      });

      test('should handle mixed case file types', () {
        expect(fileService.getMimeType('Json'), equals('application/json'));
        expect(fileService.getMimeType('MarkDown'), equals('text/markdown'));
      });
    });

    group('Edge Cases', () {
      test('should handle complex JSON validation', () {
        const complexJson = '''
        {
          "name": "Complex Document",
          "meta": {
            "created": "2024-01-01T00:00:00Z",
            "tags": ["test", "demo"]
          },
          "elements": [
            {
              "type": "paragraph",
              "content": [
                {"type": "text", "text": "Hello "},
                {"type": "text", "text": "world", "marks": ["bold"]}
              ]
            }
          ]
        }
        ''';
        
        expect(fileService.validateFileContent(complexJson, 'json'), isTrue);
      });

      test('should handle JSON with syntax errors', () {
        const jsonWithErrors = '{"name": "test",, "invalid": }';
        
        expect(fileService.validateFileContent(jsonWithErrors, 'json'), isFalse);
      });

      test('should handle very long document names', () {
        final longName = 'A' * 100 + ' Document';
        final filename = fileService.suggestFilename(longName, 'json');
        
        expect(filename, contains('.json'));
        expect(filename.length, lessThan(200)); // Should be reasonable length
      });

      test('should handle document names with only special characters', () {
        expect(
          fileService.suggestFilename('@#$%^&*()', 'json'),
          equals('document.json'),
        );
      });
    });
  });
}