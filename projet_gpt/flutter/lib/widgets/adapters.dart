import 'package:flutter/material.dart';
import 'package:c_editor_flutter/models/models.dart';

/// Adapter to convert DocTextSpan to Flutter TextSpan
TextSpan mapDocSpan(DocTextSpan s) {
  final style = TextStyle(
    fontWeight: s.bold ? FontWeight.bold : null,
    fontStyle: s.italic ? FontStyle.italic : null,
    backgroundColor: s.highlight != null
        ? Color.fromRGBO(
            (s.highlight!.color[0] * 255).round(),
            (s.highlight!.color[1] * 255).round(),
            (s.highlight!.color[2] * 255).round(),
            s.highlight!.color[3],
          )
        : null,
    decoration: s.underline != null ? TextDecoration.underline : null,
    decorationColor: s.underline != null
        ? Color.fromRGBO(
            (s.underline!.color[0] * 255).round(),
            (s.underline!.color[1] * 255).round(),
            (s.underline!.color[2] * 255).round(),
            s.underline!.color[3],
          )
        : null,
  );
  return TextSpan(text: s.text, style: style);
}

/// Convert Doc align string to TextAlign enum
TextAlign mapDocAlign(String align) {
  switch (align) {
    case 'center':
      return TextAlign.center;
    case 'right':
      return TextAlign.right;
    case 'justify':
      return TextAlign.justify;
    case 'left':
    default:
      return TextAlign.left;
  }
}