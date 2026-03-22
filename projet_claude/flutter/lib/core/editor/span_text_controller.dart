import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:c_editor_flutter/models/models.dart';
import '../../widgets/adapters.dart';

/// Custom TextEditingController that renders text based on document spans
/// Provides real-time syntax highlighting and formatting preview
class SpanTextEditingController extends TextEditingController {
  Document? _document;
  bool _enableSyntaxHighlighting = true;
  final Map<String, TextStyle> _syntaxStyles = {};
  
  SpanTextEditingController({
    super.text,
    bool enableSyntaxHighlighting = true,
  }) : _enableSyntaxHighlighting = enableSyntaxHighlighting {
    _initializeSyntaxStyles();
  }
  
  /// Enable/disable syntax highlighting
  bool get enableSyntaxHighlighting => _enableSyntaxHighlighting;
  set enableSyntaxHighlighting(bool value) {
    if (_enableSyntaxHighlighting != value) {
      _enableSyntaxHighlighting = value;
      notifyListeners();
    }
  }
  
  /// Update document for span-based rendering
  void updateDocument(Document? document) {
    if (_document != document) {
      _document = document;
      notifyListeners();
    }
  }
  
  /// Initialize syntax highlighting styles
  void _initializeSyntaxStyles() {
    _syntaxStyles.addAll({
      'header': const TextStyle(
        fontWeight: FontWeight.bold,
        color: Color(0xFF4FC3F7), // Light blue for headers
      ),
      'bold': const TextStyle(
        fontWeight: FontWeight.bold,
      ),
      'italic': const TextStyle(
        fontStyle: FontStyle.italic,
      ),
      'code': const TextStyle(
        fontFamily: 'monospace',
        backgroundColor: Color(0xFF2D2D2D),
        color: Color(0xFFE0E0E0),
      ),
      'link': const TextStyle(
        color: Color(0xFF81C784), // Light green for links
        decoration: TextDecoration.underline,
      ),
      'quote': const TextStyle(
        fontStyle: FontStyle.italic,
        color: Color(0xFF9E9E9E),
      ),
    });
  }
  
  /// Update syntax style for a specific element type
  void updateSyntaxStyle(String type, TextStyle style) {
    _syntaxStyles[type] = style;
    notifyListeners();
  }
  
  @override
  TextSpan buildTextSpan({
    required BuildContext context,
    TextStyle? style,
    required bool withComposing,
  }) {
    // If syntax highlighting is disabled or no document, use default rendering
    if (!_enableSyntaxHighlighting || _document == null) {
      return _buildPlainTextSpan(context, style, withComposing);
    }
    
    try {
      return _buildDocumentSpans(context, style, withComposing);
    } catch (e) {
      // Fallback to plain text on error
      return _buildPlainTextSpan(context, style, withComposing);
    }
  }
  
  /// Build plain text span (fallback)
  TextSpan _buildPlainTextSpan(BuildContext context, TextStyle? style, bool withComposing) {
    final text = value.text;
    final composingRange = value.composing;
    
    if (!withComposing || !composingRange.isValid || composingRange.isCollapsed) {
      return TextSpan(text: text, style: style);
    }
    
    final children = <TextSpan>[];
    
    // Text before composing
    if (composingRange.start > 0) {
      children.add(TextSpan(
        text: text.substring(0, composingRange.start),
        style: style,
      ));
    }
    
    // Composing text
    children.add(TextSpan(
      text: text.substring(composingRange.start, composingRange.end),
      style: style?.merge(const TextStyle(decoration: TextDecoration.underline)) ??
             const TextStyle(decoration: TextDecoration.underline),
    ));
    
    // Text after composing
    if (composingRange.end < text.length) {
      children.add(TextSpan(
        text: text.substring(composingRange.end),
        style: style,
      ));
    }
    
    return TextSpan(children: children);
  }
  
  /// Build spans from document elements
  TextSpan _buildDocumentSpans(BuildContext context, TextStyle? style, bool withComposing) {
    final document = _document!;
    final children = <TextSpan>[];
    
    // Track current position in text
    int currentOffset = 0;
    final text = value.text;
    
    for (final element in document.elements) {
      switch (element) {
        case DocTextElement():
          final elementSpans = _buildTextElementSpans(
            element, 
            style, 
            currentOffset, 
            text,
            withComposing,
          );
          children.addAll(elementSpans);
          
          // Calculate element length for offset tracking
          final elementText = element.spans.map((s) => s.text).join('');
          currentOffset += elementText.length;
          
          // Add newline if not last element
          if (element != document.elements.last) {
            children.add(const TextSpan(text: '\n'));
            currentOffset += 1;
          }
          break;
          
        case DocImageElement():
          // Represent images as placeholder text
          final placeholder = '![${element.alt}](${element.src})';
          children.add(TextSpan(
            text: placeholder,
            style: _syntaxStyles['link']?.merge(style) ?? style,
          ));
          currentOffset += placeholder.length;
          break;
          
        case DocTableElement():
          // Represent tables as placeholder
          children.add(TextSpan(
            text: '<!-- Table -->',
            style: _syntaxStyles['code']?.merge(style) ?? style,
          ));
          currentOffset += '<!-- Table -->'.length;
          break;
      }
    }
    
    return TextSpan(children: children.isEmpty ? [TextSpan(text: text, style: style)] : children);
  }
  
  /// Build spans for text elements with syntax highlighting
  List<TextSpan> _buildTextElementSpans(
    DocTextElement element,
    TextStyle? baseStyle,
    int elementOffset,
    String fullText,
    bool withComposing,
  ) {
    final spans = <TextSpan>[];
    final composingRange = value.composing;
    
    // Apply header styling
    TextStyle? elementStyle = baseStyle;
    if (element.level > 0) {
      elementStyle = _syntaxStyles['header']?.merge(baseStyle) ?? baseStyle;
      elementStyle = elementStyle?.copyWith(
        fontSize: (18.0 - element.level * 2.0).clamp(12.0, 18.0),
      );
    }
    
    int spanOffset = elementOffset;
    
    for (final span in element.spans) {
      TextStyle spanStyle = elementStyle ?? const TextStyle();
      
      // Apply span formatting
      if (span.bold) {
        spanStyle = spanStyle.merge(_syntaxStyles['bold'] ?? const TextStyle(fontWeight: FontWeight.bold));
      }
      if (span.italic) {
        spanStyle = spanStyle.merge(_syntaxStyles['italic'] ?? const TextStyle(fontStyle: FontStyle.italic));
      }
      if (span.underline != null) {
        spanStyle = spanStyle.copyWith(decoration: TextDecoration.underline);
        final color = span.underline!.color;
        if (color.isNotEmpty && color.length >= 4) {
          spanStyle = spanStyle.copyWith(decorationColor: Color.fromRGBO(
            (color[0] * 255).round(),
            (color[1] * 255).round(),
            (color[2] * 255).round(),
            color[3],
          ));
        }
      }
      if (span.highlight != null) {
        final color = span.highlight!.color;
        if (color.isNotEmpty && color.length >= 4) {
          spanStyle = spanStyle.copyWith(backgroundColor: Color.fromRGBO(
            (color[0] * 255).round(),
            (color[1] * 255).round(),
            (color[2] * 255).round(),
            color[3],
          ));
        }
      }
      
      // Handle composing text highlighting
      if (withComposing && composingRange.isValid && !composingRange.isCollapsed) {
        final spanStart = spanOffset;
        final spanEnd = spanOffset + span.text.length;
        
        // Check if span overlaps with composing range
        if (spanStart < composingRange.end && spanEnd > composingRange.start) {
          spans.addAll(_buildComposingTextSpan(
            span.text,
            spanStyle,
            spanStart,
            composingRange,
          ));
        } else {
          spans.add(TextSpan(text: span.text, style: spanStyle));
        }
      } else {
        spans.add(TextSpan(text: span.text, style: spanStyle));
      }
      
      spanOffset += span.text.length;
    }
    
    return spans;
  }
  
  /// Build spans with composing text highlighting
  List<TextSpan> _buildComposingTextSpan(
    String text,
    TextStyle style,
    int textOffset,
    TextRange composingRange,
  ) {
    final spans = <TextSpan>[];
    final textStart = textOffset;
    final textEnd = textOffset + text.length;
    
    // Text before composing
    if (composingRange.start > textStart) {
      final beforeEnd = (composingRange.start - textStart).clamp(0, text.length);
      spans.add(TextSpan(
        text: text.substring(0, beforeEnd),
        style: style,
      ));
    }
    
    // Composing text
    final composingStart = (composingRange.start - textStart).clamp(0, text.length);
    final composingEnd = (composingRange.end - textStart).clamp(0, text.length);
    
    if (composingStart < composingEnd) {
      spans.add(TextSpan(
        text: text.substring(composingStart, composingEnd),
        style: style.merge(const TextStyle(decoration: TextDecoration.underline)),
      ));
    }
    
    // Text after composing
    if (composingRange.end < textEnd) {
      final afterStart = (composingRange.end - textStart).clamp(0, text.length);
      spans.add(TextSpan(
        text: text.substring(afterStart),
        style: style,
      ));
    }
    
    return spans.isEmpty ? [TextSpan(text: text, style: style)] : spans;
  }
  
  /// Apply markdown formatting to selection
  void applyFormatting(MarkdownFormat format) {
    final selection = this.selection;
    if (!selection.isValid) return;
    
    final selectedText = selection.textInside(text);
    String prefix = '';
    String suffix = '';
    
    switch (format) {
      case MarkdownFormat.bold:
        prefix = suffix = '**';
        break;
      case MarkdownFormat.italic:
        prefix = suffix = '*';
        break;
      case MarkdownFormat.boldItalic:
        prefix = suffix = '***';
        break;
      case MarkdownFormat.underline:
        prefix = suffix = '++';
        break;
      case MarkdownFormat.highlight:
        prefix = suffix = '==';
        break;
      case MarkdownFormat.code:
        prefix = suffix = '`';
        break;
      case MarkdownFormat.header1:
        prefix = '# ';
        break;
      case MarkdownFormat.header2:
        prefix = '## ';
        break;
      case MarkdownFormat.header3:
        prefix = '### ';
        break;
      case MarkdownFormat.quote:
        prefix = '> ';
        break;
    }
    
    final newText = text.replaceRange(
      selection.start,
      selection.end,
      prefix + selectedText + suffix,
    );
    
    // Update text and selection
    value = value.copyWith(
      text: newText,
      selection: TextSelection(
        baseOffset: selection.start + prefix.length,
        extentOffset: selection.start + prefix.length + selectedText.length,
      ),
    );
  }
  
  /// Toggle format at cursor position
  void toggleFormat(MarkdownFormat format) {
    final selection = this.selection;
    final currentWord = _getCurrentWord();
    
    if (currentWord.isEmpty) {
      applyFormatting(format);
      return;
    }
    
    // Check if format is already applied
    final patterns = _getFormatPatterns(format);
    bool isFormatted = false;
    
    for (final pattern in patterns) {
      if (currentWord.startsWith(pattern.prefix) && currentWord.endsWith(pattern.suffix)) {
        // Remove formatting
        final newText = currentWord.substring(
          pattern.prefix.length,
          currentWord.length - pattern.suffix.length,
        );
        
        final wordStart = _getWordStart();
        final wordEnd = _getWordEnd();
        
        value = value.copyWith(
          text: text.replaceRange(wordStart, wordEnd, newText),
          selection: TextSelection.collapsed(offset: wordStart + newText.length),
        );
        
        isFormatted = true;
        break;
      }
    }
    
    if (!isFormatted) {
      applyFormatting(format);
    }
  }
  
  /// Get current word under cursor
  String _getCurrentWord() {
    final offset = selection.start;
    final wordStart = _getWordStart();
    final wordEnd = _getWordEnd();
    return text.substring(wordStart, wordEnd);
  }
  
  /// Get start of current word
  int _getWordStart() {
    final offset = selection.start;
    int start = offset;
    while (start > 0 && text[start - 1] != ' ' && text[start - 1] != '\n') {
      start--;
    }
    return start;
  }
  
  /// Get end of current word
  int _getWordEnd() {
    final offset = selection.start;
    int end = offset;
    while (end < text.length && text[end] != ' ' && text[end] != '\n') {
      end++;
    }
    return end;
  }
  
  /// Get format patterns for detection
  List<FormatPattern> _getFormatPatterns(MarkdownFormat format) {
    switch (format) {
      case MarkdownFormat.bold:
        return [const FormatPattern('**', '**')];
      case MarkdownFormat.italic:
        return [const FormatPattern('*', '*')];
      case MarkdownFormat.boldItalic:
        return [const FormatPattern('***', '***')];
      case MarkdownFormat.underline:
        return [const FormatPattern('++', '++')];
      case MarkdownFormat.highlight:
        return [const FormatPattern('==', '==')];
      case MarkdownFormat.code:
        return [const FormatPattern('`', '`')];
      case MarkdownFormat.header1:
        return [const FormatPattern('# ', '')];
      case MarkdownFormat.header2:
        return [const FormatPattern('## ', '')];
      case MarkdownFormat.header3:
        return [const FormatPattern('### ', '')];
      case MarkdownFormat.quote:
        return [const FormatPattern('> ', '')];
    }
  }
}

/// Markdown formatting types
enum MarkdownFormat {
  bold,
  italic,
  boldItalic,
  underline,
  highlight,
  code,
  header1,
  header2,
  header3,
  quote,
}

/// Format pattern for detection
class FormatPattern {
  final String prefix;
  final String suffix;
  
  const FormatPattern(this.prefix, this.suffix);
}