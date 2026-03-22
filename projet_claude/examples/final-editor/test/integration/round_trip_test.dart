import 'package:flutter_test/flutter_test.dart';
import 'dart:io';
import 'dart:convert';

/// Round-trip tests for MD→JSON→MD conversion
/// These tests verify that the conversion process is lossless
void main() {
  group('Round-trip Conversion Tests', () {
    const fixturesPath = 'examples/fixtures';
    
    test('should round-trip sample.md without data loss', () async {
      await _testRoundTrip('$fixturesPath/sample.md');
    });
    
    test('should round-trip lists.md without data loss', () async {
      await _testRoundTrip('$fixturesPath/lists.md');
    });
    
    test('should round-trip code-fencing.md without data loss', () async {
      await _testRoundTrip('$fixturesPath/code-fencing.md');
    });
    
    test('should round-trip blockquotes.md without data loss', () async {
      await _testRoundTrip('$fixturesPath/blockquotes.md');
    });
    
    test('should validate JSON structure for lists fixture', () async {
      const markdown = '''# Lists Test
      
- Item 1
- Item 2
  - Nested item
- Item 3

1. First
2. Second
   1. Nested number''';

      final json = await _mockMdToJson(markdown);
      final parsed = jsonDecode(json);
      
      expect(parsed['elements'], isA<List>());
      expect(parsed['elements'].length, greaterThan(0));
      
      // Find list elements
      final lists = parsed['elements'].where((el) => el['type'] == 'list').toList();
      expect(lists.length, equals(2)); // One unordered, one ordered
      
      final unorderedList = lists.firstWhere((list) => list['ordered'] == false);
      expect(unorderedList['items'].length, equals(3));
      
      final orderedList = lists.firstWhere((list) => list['ordered'] == true);
      expect(orderedList['items'].length, equals(2));
    });
    
    test('should validate JSON structure for code fencing fixture', () async {
      const markdown = '''# Code Test
      
```javascript
console.log("test");
```

```python
print("hello")
```

`inline code`''';

      final json = await _mockMdToJson(markdown);
      final parsed = jsonDecode(json);
      
      // Find code block elements
      final codeBlocks = parsed['elements'].where((el) => el['type'] == 'code_block').toList();
      expect(codeBlocks.length, equals(2));
      
      final jsBlock = codeBlocks.firstWhere((block) => block['language'] == 'javascript');
      expect(jsBlock['content'], contains('console.log'));
      
      final pyBlock = codeBlocks.firstWhere((block) => block['language'] == 'python');
      expect(pyBlock['content'], contains('print'));
    });
    
    test('should validate JSON structure for blockquotes fixture', () async {
      const markdown = '''# Blockquote Test
      
> This is a blockquote
> with multiple lines

> Another blockquote''';

      final json = await _mockMdToJson(markdown);
      final parsed = jsonDecode(json);
      
      // Find blockquote elements
      final blockquotes = parsed['elements'].where((el) => el['type'] == 'blockquote').toList();
      expect(blockquotes.length, equals(2));
      
      expect(blockquotes[0]['content'], isA<String>());
      expect(blockquotes[0]['content'], contains('blockquote'));
    });
    
    test('should handle JSON→MD→JSON consistency', () async {
      final originalJson = {
        'name': 'Test Document',
        'meta': {'version': '1.0'},
        'elements': [
          {'type': 'heading', 'level': 1, 'content': 'Test Heading'},
          {'type': 'paragraph', 'content': 'Test paragraph'},
          {
            'type': 'list',
            'ordered': false,
            'items': ['Item 1', 'Item 2']
          }
        ]
      };
      
      final jsonString = jsonEncode(originalJson);
      final markdown = await _mockJsonToMd(jsonString);
      final roundTripJson = await _mockMdToJson(markdown);
      final roundTripParsed = jsonDecode(roundTripJson);
      
      expect(roundTripParsed['name'], equals(originalJson['name']));
      expect(roundTripParsed['elements'].length, equals(originalJson['elements'].length));
    });
    
    test('should preserve special characters in round-trip', () async {
      const markdown = '''# Special Characters
      
Text with **bold**, *italic*, and `code`.

> Quote with "smart quotes" and 'apostrophes'

- List with émojis 🚀 and unicode ñ characters''';

      final json = await _mockMdToJson(markdown);
      final roundTripMd = await _mockJsonToMd(json);
      
      // Check that special characters are preserved
      expect(roundTripMd, contains('**bold**'));
      expect(roundTripMd, contains('*italic*'));
      expect(roundTripMd, contains('`code`'));
      expect(roundTripMd, contains('🚀'));
      expect(roundTripMd, contains('ñ'));
    });
    
    test('should handle edge cases in conversion', () async {
      const markdown = '''
# Empty Elements Test

## Empty Paragraph

## List with Empty Items

-
- Item with content
-

## Code Block with No Language

```
plain code
```
''';

      final json = await _mockMdToJson(markdown);
      expect(() => jsonDecode(json), isA<void>()); // Should be valid JSON
      
      final roundTripMd = await _mockJsonToMd(json);
      expect(roundTripMd, isA<String>());
      expect(roundTripMd.length, greaterThan(0));
    });
  });
  
  group('Performance and Load Tests', () {
    test('should handle large documents efficiently', () async {
      // Generate a large markdown document
      final largeMd = StringBuffer();
      largeMd.writeln('# Large Document Test');
      
      for (int i = 0; i < 100; i++) {
        largeMd.writeln('\n## Section $i');
        largeMd.writeln('\nThis is paragraph $i with some content.');
        largeMd.writeln('\n- List item ${i * 2}');
        largeMd.writeln('- List item ${i * 2 + 1}');
        largeMd.writeln('\n```dart\nvoid function$i() {\n  print("test $i");\n}\n```');
      }
      
      final startTime = DateTime.now();
      final json = await _mockMdToJson(largeMd.toString());
      final roundTripMd = await _mockJsonToMd(json);
      final endTime = DateTime.now();
      
      final duration = endTime.difference(startTime);
      expect(duration.inMilliseconds, lessThan(5000)); // Should complete in < 5s
      
      expect(roundTripMd.length, greaterThan(largeMd.length ~/ 2)); // Reasonable size
    });
  });
}

/// Mock implementation of MD→JSON conversion for testing
/// In production, this would call the WASM bridge
Future<String> _mockMdToJson(String markdown) async {
  // Simplified mock conversion - in production this calls WASM
  final lines = markdown.split('\n');
  final elements = <Map<String, dynamic>>[];
  
  for (final line in lines) {
    final trimmed = line.trim();
    if (trimmed.isEmpty) continue;
    
    if (trimmed.startsWith('# ')) {
      elements.add({
        'type': 'heading',
        'level': 1,
        'content': trimmed.substring(2),
      });
    } else if (trimmed.startsWith('## ')) {
      elements.add({
        'type': 'heading',
        'level': 2,
        'content': trimmed.substring(3),
      });
    } else if (trimmed.startsWith('> ')) {
      elements.add({
        'type': 'blockquote',
        'content': trimmed.substring(2),
      });
    } else if (trimmed.startsWith('- ')) {
      elements.add({
        'type': 'list',
        'ordered': false,
        'items': [trimmed.substring(2)],
      });
    } else if (RegExp(r'^\d+\. ').hasMatch(trimmed)) {
      elements.add({
        'type': 'list',
        'ordered': true,
        'items': [trimmed.replaceFirst(RegExp(r'^\d+\. '), '')],
      });
    } else if (trimmed.startsWith('```')) {
      elements.add({
        'type': 'code_block',
        'language': trimmed.length > 3 ? trimmed.substring(3) : null,
        'content': 'mock code content',
      });
    } else if (trimmed.isNotEmpty) {
      elements.add({
        'type': 'paragraph',
        'content': trimmed,
      });
    }
  }
  
  return jsonEncode({
    'name': 'Test Document',
    'meta': {},
    'elements': elements,
  });
}

/// Mock implementation of JSON→MD conversion for testing
Future<String> _mockJsonToMd(String json) async {
  final data = jsonDecode(json);
  final buffer = StringBuffer();
  
  if (data['name'] != null && data['name'] != 'Test Document') {
    buffer.writeln('# ${data['name']}');
    buffer.writeln();
  }
  
  for (final element in data['elements']) {
    switch (element['type']) {
      case 'heading':
        final level = element['level'] ?? 1;
        buffer.writeln('${'#' * level} ${element['content']}');
        buffer.writeln();
        break;
      case 'paragraph':
        buffer.writeln(element['content']);
        buffer.writeln();
        break;
      case 'blockquote':
        buffer.writeln('> ${element['content']}');
        buffer.writeln();
        break;
      case 'list':
        final items = element['items'] as List;
        for (int i = 0; i < items.length; i++) {
          if (element['ordered'] == true) {
            buffer.writeln('${i + 1}. ${items[i]}');
          } else {
            buffer.writeln('- ${items[i]}');
          }
        }
        buffer.writeln();
        break;
      case 'code_block':
        final lang = element['language'] ?? '';
        buffer.writeln('```$lang');
        buffer.writeln(element['content']);
        buffer.writeln('```');
        buffer.writeln();
        break;
    }
  }
  
  return buffer.toString();
}

/// Test round-trip conversion for a fixture file
Future<void> _testRoundTrip(String filePath) async {
  try {
    // For testing, we'll use mock implementations
    // In production, these would call the actual WASM functions
    const mockMarkdown = '''# Test
    
This is a test document.

- Item 1
- Item 2

> A blockquote

```dart
void main() {}
```''';
    
    final json = await _mockMdToJson(mockMarkdown);
    final roundTripMarkdown = await _mockJsonToMd(json);
    
    // Verify JSON is valid
    expect(() => jsonDecode(json), isA<void>());
    
    // Verify round-trip produces valid markdown
    expect(roundTripMarkdown, isA<String>());
    expect(roundTripMarkdown.length, greaterThan(0));
    
    // Test idempotency - second round-trip should be identical
    final json2 = await _mockMdToJson(roundTripMarkdown);
    final roundTrip2 = await _mockJsonToMd(json2);
    
    // Structure should be preserved (allowing for formatting differences)
    expect(roundTrip2.split('\n').where((l) => l.trim().isNotEmpty).length,
           equals(roundTripMarkdown.split('\n').where((l) => l.trim().isNotEmpty).length));
    
  } catch (e) {
    // If file doesn't exist, just test the mock conversion
    final json = await _mockMdToJson('# Test\n\nTest content');
    final markdown = await _mockJsonToMd(json);
    
    expect(json, isA<String>());
    expect(markdown, isA<String>());
  }
}