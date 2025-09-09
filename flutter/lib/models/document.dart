import 'package:flutter/foundation.dart';

// Main document structure
@immutable
class Document {
  final String name;
  final DocumentMeta meta;
  final List<DocElement> elements;

  const Document({
    required this.name,
    required this.meta, 
    required this.elements,
  });

  Map<String, dynamic> toJson() {
    return {
      'name': name,
      'meta': meta.toJson(),
      'elements': elements.map((e) => e.toJson()).toList(),
    };
  }

  factory Document.fromJson(Map<String, dynamic> json) {
    return Document(
      name: json['name'] as String? ?? '',
      meta: json['meta'] != null 
          ? DocumentMeta.fromJson(json['meta'] as Map<String, dynamic>)
          : const DocumentMeta(),
      elements: (json['elements'] as List?)
          ?.map((e) => DocElement.fromJson(e as Map<String, dynamic>))
          .toList() ?? [],
    );
  }
}

// Document metadata
@immutable
class DocumentMeta {
  final String font;
  final int fontSize;
  final List<double> defaultTextColor;
  final String? title;
  final String? author;
  final DateTime? created;
  final DateTime? modified;

  const DocumentMeta({
    this.font = "Helvetica",
    this.fontSize = 11,
    this.defaultTextColor = const [0, 0, 0, 1],
    this.title,
    this.author,
    this.created,
    this.modified,
  });

  Map<String, dynamic> toJson() {
    return {
      'font': font,
      'fontSize': fontSize,
      'defaultTextColor': defaultTextColor,
      if (title != null) 'title': title,
      if (author != null) 'author': author,
      if (created != null) 'created': created!.toIso8601String(),
      if (modified != null) 'modified': modified!.toIso8601String(),
    };
  }

  factory DocumentMeta.fromJson(Map<String, dynamic> json) {
    return DocumentMeta(
      font: json['font'] as String? ?? "Helvetica",
      fontSize: json['fontSize'] as int? ?? 11,
      defaultTextColor: (json['defaultTextColor'] as List?)
          ?.map((e) => (e as num).toDouble()).toList() ?? [0, 0, 0, 1],
      title: json['title'] as String?,
      author: json['author'] as String?,
      created: json['created'] != null ? DateTime.parse(json['created']) : null,
      modified: json['modified'] != null ? DateTime.parse(json['modified']) : null,
    );
  }
}

// Base element
@immutable
abstract class DocElement {
  const DocElement();

  Map<String, dynamic> toJson();
  
  factory DocElement.fromJson(Map<String, dynamic> json) {
    final type = json['type'] as String;
    
    switch (type) {
      case 'text':
        return DocTextElement.fromJson(json);
      case 'image':
        return DocImageElement.fromJson(json);
      case 'table':
        return DocTableElement.fromJson(json);
      default:
        throw ArgumentError('Unknown element type: $type');
    }
  }
}

// Text span
@immutable
class DocTextSpan {
  final String text;
  final bool bold;
  final bool italic;
  final DocUnderline? underline;
  final DocHighlight? highlight;

  const DocTextSpan({
    required this.text,
    this.bold = false,
    this.italic = false,
    this.underline,
    this.highlight,
  });

  Map<String, dynamic> toJson() {
    return {
      'text': text,
      'bold': bold,
      'italic': italic,
      if (underline != null) 'underline': underline!.toJson(),
      if (highlight != null) 'highlight': highlight!.toJson(),
    };
  }

  factory DocTextSpan.fromJson(Map<String, dynamic> json) {
    return DocTextSpan(
      text: json['text'] as String,
      bold: json['bold'] as bool? ?? false,
      italic: json['italic'] as bool? ?? false,
      underline: json['underline'] != null 
          ? DocUnderline.fromJson(json['underline'] as Map<String, dynamic>)
          : null,
      highlight: json['highlight'] != null
          ? DocHighlight.fromJson(json['highlight'] as Map<String, dynamic>)
          : null,
    );
  }
}

// Text element
@immutable
class DocTextElement extends DocElement {
  final List<DocTextSpan> spans;
  final String align;
  final String? font;
  final int? fontSize;
  final List<double>? color;
  final int level;

  const DocTextElement({
    required this.spans,
    this.align = "left",
    this.font,
    this.fontSize,
    this.color,
    this.level = 0,
  });

  @override
  Map<String, dynamic> toJson() {
    return {
      'type': 'text',
      'spans': spans.map((s) => s.toJson()).toList(),
      'align': align,
      if (font != null) 'font': font,
      if (fontSize != null) 'fontSize': fontSize,
      if (color != null) 'color': color,
      'level': level,
    };
  }

  factory DocTextElement.fromJson(Map<String, dynamic> json) {
    return DocTextElement(
      spans: (json['spans'] as List?)
          ?.map((s) => DocTextSpan.fromJson(s as Map<String, dynamic>))
          .toList() ?? [DocTextSpan(text: json['text'] as String? ?? '')],
      align: json['align'] as String? ?? "left",
      font: json['font'] as String?,
      fontSize: json['fontSize'] as int?,
      color: (json['color'] as List?)
          ?.map((e) => (e as num).toDouble()).toList(),
      level: json['level'] as int? ?? 0,
    );
  }
}

// Image element  
@immutable
class DocImageElement extends DocElement {
  final String src;
  final String alt;
  final String align;
  final int? width;
  final int? height;
  final double alpha;

  const DocImageElement({
    required this.src,
    this.alt = "",
    this.align = "left",
    this.width,
    this.height,
    this.alpha = 1.0,
  });

  @override
  Map<String, dynamic> toJson() {
    return {
      'type': 'image',
      'src': src,
      'alt': alt,
      'align': align,
      if (width != null) 'width': width,
      if (height != null) 'height': height,
      'alpha': alpha,
    };
  }

  factory DocImageElement.fromJson(Map<String, dynamic> json) {
    return DocImageElement(
      src: json['src'] as String,
      alt: json['alt'] as String? ?? "",
      align: json['align'] as String? ?? "left",
      width: json['width'] as int?,
      height: json['height'] as int?,
      alpha: (json['alpha'] as num?)?.toDouble() ?? 1.0,
    );
  }
}

// Table element
@immutable  
class DocTableElement extends DocElement {
  final List<List<List<DocTextSpan>>> rows;
  final List<double> gridColor;
  final int gridSize;
  final List<double> backgroundColor;

  const DocTableElement({
    required this.rows,
    this.gridColor = const [0, 0, 0, 0],
    this.gridSize = 1,
    this.backgroundColor = const [1, 1, 1, 1],
  });

  @override
  Map<String, dynamic> toJson() {
    return {
      'type': 'table',
      'rows': rows.map((row) => 
          row.map((cell) => 
              cell.map((span) => span.toJson()).toList()
          ).toList()
      ).toList(),
      'gridColor': gridColor,
      'gridSize': gridSize,
      'backgroundColor': backgroundColor,
    };
  }

  factory DocTableElement.fromJson(Map<String, dynamic> json) {
    return DocTableElement(
      rows: (json['rows'] as List).map((row) =>
          (row as List).map((cell) =>
              (cell as List).map((span) =>
                  DocTextSpan.fromJson(span as Map<String, dynamic>)
              ).toList()
          ).toList()
      ).toList(),
      gridColor: (json['gridColor'] as List?)
          ?.map((e) => (e as num).toDouble()).toList() ?? [0, 0, 0, 0],
      gridSize: json['gridSize'] as int? ?? 1,
      backgroundColor: (json['backgroundColor'] as List?)
          ?.map((e) => (e as num).toDouble()).toList() ?? [1, 1, 1, 1],
    );
  }
}

// Underline style
@immutable
class DocUnderline {
  final List<double> color;
  final int gap;

  const DocUnderline({
    this.color = const [0, 0, 0, 0.4],
    this.gap = 7,
  });

  Map<String, dynamic> toJson() {
    return {
      'color': color,
      'gap': gap,
    };
  }

  factory DocUnderline.fromJson(Map<String, dynamic> json) {
    return DocUnderline(
      color: (json['color'] as List?)
          ?.map((e) => (e as num).toDouble()).toList() ?? [0, 0, 0, 0.4],
      gap: json['gap'] as int? ?? 7,
    );
  }
}

// Highlight style
@immutable
class DocHighlight {
  final List<double> color;

  const DocHighlight({
    this.color = const [1, 1, 0, 0.3],
  });

  Map<String, dynamic> toJson() {
    return {
      'color': color,
    };
  }

  factory DocHighlight.fromJson(Map<String, dynamic> json) {
    return DocHighlight(
      color: (json['color'] as List?)
          ?.map((e) => (e as num).toDouble()).toList() ?? [1, 1, 0, 0.3],
    );
  }
}