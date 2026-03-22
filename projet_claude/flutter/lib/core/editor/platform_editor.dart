import 'dart:io';

import '../../editor_api.dart';

/// Platform-specific editor factory
class PlatformEditor {
  static EditorApi? _instance;
  
  static EditorApi get instance {
    return _instance ??= EditorApiFactory.instance();
  }
  
  static void reset() {
    _instance?.dispose();
    _instance = null;
  }
}