import 'dart:ui';

/// Types of document elements
enum ElementType { text, image, table }

/// Base class for all document elements
abstract class Element {
  ElementType get type;
  
  /// Convert element to JSON representation
  Map<String, dynamic> toJson();
  
  /// Create element from JSON
  static Element fromJson(Map<String, dynamic> json) {
    final type = json['type'] as String;
    switch (type) {
      case 'text':
        return TextElement.fromJson(json);
      case 'image':
        return ImageElement.fromJson(json);
      case 'table':
        return TableElement.fromJson(json);
      default:
        throw ArgumentError('Unknown element type: $type');
    }
  }
}

/// Text element with inline formatting spans
class TextElement extends Element {
  @override
  ElementType get type => ElementType.text;
  
  final List<TextSpan> spans;
  final int level; // Header level: 0=normal, 1-6=headers
  
  const TextElement(this.spans, {this.level = 0});
  
  @override
  Map<String, dynamic> toJson() {
    return {
      'type': 'text',
      'level': level,
      'spans': spans.map((s) => s.toJson()).toList(),
    };
  }
  
  static TextElement fromJson(Map<String, dynamic> json) {
    final spansList = json['spans'] as List? ?? [];
    final spans = spansList.map((s) => TextSpan.fromJson(s)).toList();
    final level = json['level'] as int? ?? 0;
    return TextElement(spans, level: level);
  }
}

/// Image element with attributes
class ImageElement extends Element {
  @override
  ElementType get type => ElementType.image;
  
  final String src;
  final String alt;
  final int? width;
  final int? height;
  final double alpha;
  final TextAlign align;
  
  const ImageElement({
    required this.src,
    required this.alt,
    this.width,
    this.height,
    this.alpha = 1.0,
    this.align = TextAlign.left,
  });
  
  @override
  Map<String, dynamic> toJson() {
    return {
      'type': 'image',
      'src': src,
      'alt': alt,
      if (width != null) 'width': width,
      if (height != null) 'height': height,
      'alpha': alpha,
      'align': _alignToString(align),
    };
  }
  
  static ImageElement fromJson(Map<String, dynamic> json) {
    return ImageElement(
      src: json['src'] as String? ?? '',
      alt: json['alt'] as String? ?? '',
      width: json['width'] as int?,
      height: json['height'] as int?,
      alpha: (json['alpha'] as double?) ?? 1.0,
      align: _alignFromString(json['align'] as String? ?? 'left'),
    );
  }
  
  static String _alignToString(TextAlign align) {
    switch (align) {
      case TextAlign.left: return 'left';
      case TextAlign.center: return 'center';
      case TextAlign.right: return 'right';
      case TextAlign.justify: return 'justify';
      default: return 'left';
    }
  }
  
  static TextAlign _alignFromString(String align) {
    switch (align) {
      case 'left': return TextAlign.left;
      case 'center': return TextAlign.center;
      case 'right': return TextAlign.right;
      case 'justify': return TextAlign.justify;
      default: return TextAlign.left;
    }
  }
}

/// Table element
class TableElement extends Element {
  @override
  ElementType get type => ElementType.table;
  
  final List<List<TextElement>> rows;
  
  const TableElement(this.rows);
  
  @override
  Map<String, dynamic> toJson() {
    return {
      'type': 'table',
      'rows': rows.map((row) => 
        row.map((cell) => cell.toJson()).toList()
      ).toList(),
    };
  }
  
  static TableElement fromJson(Map<String, dynamic> json) {
    final rowsList = json['rows'] as List? ?? [];
    final rows = rowsList.map((row) {
      final cellsList = row as List;
      return cellsList.map((cell) => TextElement.fromJson(cell)).toList();
    }).toList();
    return TableElement(rows);
  }
}

/// Individual text span with formatting
class TextSpan {
  final String text;
  final bool bold;
  final bool italic;
  final bool highlight;
  final bool underline;
  final Color? highlightColor;
  final Color? underlineColor;
  final int? underlineGap;
  
  const TextSpan({
    required this.text,
    this.bold = false,
    this.italic = false,
    this.highlight = false,
    this.underline = false,
    this.highlightColor,
    this.underlineColor,
    this.underlineGap,
  });
  
  Map<String, dynamic> toJson() {
    return {
      'text': text,
      'bold': bold,
      'italic': italic,
      'highlight': highlight,
      'underline': underline,
      if (highlightColor != null) 'highlight_color': _colorToRgba(highlightColor!),
      if (underlineColor != null) 'underline_color': _colorToRgba(underlineColor!),
      if (underlineGap != null) 'underline_gap': underlineGap,
    };
  }
  
  static TextSpan fromJson(Map<String, dynamic> json) {
    return TextSpan(
      text: json['text'] as String? ?? '',
      bold: json['bold'] as bool? ?? false,
      italic: json['italic'] as bool? ?? false,
      highlight: json['highlight'] as bool? ?? false,
      underline: json['underline'] as bool? ?? false,
      highlightColor: json['highlight_color'] != null 
          ? _colorFromRgba(json['highlight_color']) 
          : null,
      underlineColor: json['underline_color'] != null
          ? _colorFromRgba(json['underline_color'])
          : null,
      underlineGap: json['underline_gap'] as int?,
    );
  }
  
  static List<double> _colorToRgba(Color color) {
    return [
      color.red / 255.0,
      color.green / 255.0,
      color.blue / 255.0,
      color.alpha / 255.0,
    ];
  }
  
  static Color _colorFromRgba(dynamic rgba) {
    if (rgba is List && rgba.length >= 4) {
      final r = (rgba[0] as double).clamp(0.0, 1.0);
      final g = (rgba[1] as double).clamp(0.0, 1.0);
      final b = (rgba[2] as double).clamp(0.0, 1.0);
      final a = (rgba[3] as double).clamp(0.0, 1.0);
      return Color.from(alpha: a, red: r, green: g, blue: b);
    }
    return const Color(0xFF000000);
  }
}

/// Complete document containing elements
class Document {
  final List<Element> elements;
  
  const Document(this.elements);
  
  Map<String, dynamic> toJson() {
    return {
      'elements': elements.map((e) => e.toJson()).toList(),
    };
  }
  
  static Document fromJson(Map<String, dynamic> json) {
    final elementsList = json['elements'] as List? ?? [];
    final elements = elementsList.map((e) => Element.fromJson(e)).toList();
    return Document(elements);
  }
}