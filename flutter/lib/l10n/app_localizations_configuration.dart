import 'package:flutter/widgets.dart';

const List<Locale> supportedLocales = [
  Locale('fr', ''),
  Locale('en', ''),
];

const Locale defaultLocale = Locale('fr', '');

Locale localeFromString(String localeString) {
  final parts = localeString.split('_');
  if (parts.length == 2) {
    return Locale(parts[0], parts[1]);
  }
  return Locale(parts[0]);
}

String localeToString(Locale locale) {
  return '${locale.languageCode}_${locale.countryCode ?? ''}';
}