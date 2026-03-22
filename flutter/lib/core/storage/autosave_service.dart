import 'dart:async';
import 'dart:convert';
import 'dart:io' as io;
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:c_editor_flutter/models/models.dart';
import 'storage_service.dart';

/// Autosave service with versioning and crash recovery
class AutosaveService {
  final StorageService _storageService;
  Timer? _autosaveTimer;
  Timer? _versionTimer;
  
  static const Duration autosaveInterval = Duration(milliseconds: 700);
  static const Duration versionInterval = Duration(minutes: 5);
  static const int maxVersions = 25;
  
  String? _currentPath;
  Document? _currentDocument;
  String? _lastSavedContent;
  bool _hasUnsavedChanges = false;
  
  // Status tracking
  DateTime? _lastAutosave;
  DateTime? _lastVersion;
  int _autosaveCount = 0;
  int _versionCount = 0;

  AutosaveService(this._storageService);

  /// Start autosave for a document
  void startAutosave(String documentPath, Document document) {
    _currentPath = documentPath;
    _currentDocument = document;
    _lastSavedContent = jsonEncode(document.toJson());
    _hasUnsavedChanges = false;
    
    _scheduleAutosave();
    _scheduleVersionSave();
  }

  /// Update document content (triggers autosave)
  void updateDocument(Document document) {
    _currentDocument = document;
    final newContent = jsonEncode(document.toJson());
    
    if (newContent != _lastSavedContent) {
      _hasUnsavedChanges = true;
      _scheduleAutosave();
    }
  }

  /// Force immediate save
  Future<void> forceSave() async {
    _autosaveTimer?.cancel();
    await _performAutosave();
  }

  /// Force version save (Cmd/Ctrl+S)
  Future<void> forceVersionSave() async {
    if (_currentDocument != null && _currentPath != null) {
      await _saveVersion();
      await _performAutosave();
    }
  }

  /// Stop autosave
  void stopAutosave() {
    _autosaveTimer?.cancel();
    _versionTimer?.cancel();
    _currentPath = null;
    _currentDocument = null;
    _lastSavedContent = null;
    _hasUnsavedChanges = false;
  }

  /// Check for crash recovery files
  Future<CrashRecoveryInfo?> checkCrashRecovery(String documentPath) async {
    final tempPath = '$documentPath.tmp';
    
    try {
      if (kIsWeb) {
        // Web implementation using storage service
        final exists = await _storageService.pickNoteFile(); // Simplified for web
        return null; // Web doesn't support crash recovery yet
      } else {
        // Native implementation
        final tempFile = io.File(tempPath);
        if (await tempFile.exists()) {
          final content = await tempFile.readAsString();
          final document = Document.fromJson(jsonDecode(content));
          final lastModified = await tempFile.lastModified();
          
          return CrashRecoveryInfo(
            tempPath: tempPath,
            document: document,
            lastModified: lastModified,
          );
        }
      }
    } catch (e) {
      debugPrint('Error checking crash recovery: $e');
    }
    
    return null;
  }

  /// Accept crash recovery (delete temp file)
  Future<void> acceptCrashRecovery(String tempPath) async {
    try {
      if (!kIsWeb) {
        final tempFile = io.File(tempPath);
        if (await tempFile.exists()) {
          await tempFile.delete();
        }
      }
    } catch (e) {
      debugPrint('Error accepting crash recovery: $e');
    }
  }

  /// Reject crash recovery (keep temp file as backup)
  Future<void> rejectCrashRecovery(String tempPath) async {
    try {
      if (!kIsWeb) {
        final tempFile = io.File(tempPath);
        if (await tempFile.exists()) {
          // Rename to .backup instead of deleting
          final backupPath = '$tempPath.backup.${DateTime.now().millisecondsSinceEpoch}';
          await tempFile.rename(backupPath);
        }
      }
    } catch (e) {
      debugPrint('Error rejecting crash recovery: $e');
    }
  }

  /// Get available versions for a document
  Future<List<VersionInfo>> getVersions(String documentPath) async {
    final versions = <VersionInfo>[];
    
    try {
      if (kIsWeb) {
        // Web: versions stored in localStorage with keys
        // This would need integration with storage service
        return versions;
      } else {
        // Native: versions stored in .versions directory
        final versionsDir = io.Directory('${_getDocumentDir(documentPath)}/.versions');
        if (await versionsDir.exists()) {
          await for (final entity in versionsDir.list()) {
            if (entity is io.File && entity.path.endsWith('.json')) {
              try {
                final content = await entity.readAsString();
                final document = Document.fromJson(jsonDecode(content));
                final lastModified = await entity.lastModified();
                final filename = entity.path.split('/').last;
                
                versions.add(VersionInfo(
                  path: entity.path,
                  document: document,
                  timestamp: lastModified,
                  filename: filename,
                ));
              } catch (e) {
                debugPrint('Error reading version file ${entity.path}: $e');
              }
            }
          }
        }
      }
    } catch (e) {
      debugPrint('Error getting versions: $e');
    }
    
    // Sort by timestamp, newest first
    versions.sort((a, b) => b.timestamp.compareTo(a.timestamp));
    return versions;
  }

  /// Restore from version
  Future<Document?> restoreVersion(VersionInfo version) async {
    try {
      return version.document;
    } catch (e) {
      debugPrint('Error restoring version: $e');
      return null;
    }
  }

  /// Get autosave status
  AutosaveStatus get status => AutosaveStatus(
    isActive: _autosaveTimer?.isActive ?? false,
    hasUnsavedChanges: _hasUnsavedChanges,
    lastAutosave: _lastAutosave,
    lastVersion: _lastVersion,
    autosaveCount: _autosaveCount,
    versionCount: _versionCount,
  );

  void _scheduleAutosave() {
    _autosaveTimer?.cancel();
    _autosaveTimer = Timer(autosaveInterval, _performAutosave);
  }

  void _scheduleVersionSave() {
    _versionTimer?.cancel();
    _versionTimer = Timer(versionInterval, _saveVersion);
  }

  Future<void> _performAutosave() async {
    if (_currentDocument == null || _currentPath == null || !_hasUnsavedChanges) {
      return;
    }

    try {
      final content = jsonEncode(_currentDocument!.toJson());
      
      if (kIsWeb) {
        // Web: direct save (no temp file)
        await _storageService.saveNote(_currentPath!, _currentDocument!);
      } else {
        // Native: atomic save using temp file
        final tempPath = '$_currentPath.tmp';
        final tempFile = io.File(tempPath);
        
        // Write to temp file
        await tempFile.writeAsString(content);
        
        // Atomic rename
        await tempFile.rename(_currentPath!);
      }
      
      _lastSavedContent = content;
      _hasUnsavedChanges = false;
      _lastAutosave = DateTime.now();
      _autosaveCount++;
      
      debugPrint('Autosaved to $_currentPath');
    } catch (e) {
      debugPrint('Autosave failed: $e');
    }
  }

  Future<void> _saveVersion() async {
    if (_currentDocument == null || _currentPath == null) {
      return;
    }

    try {
      final timestamp = DateTime.now();
      final content = jsonEncode(_currentDocument!.toJson());
      
      if (kIsWeb) {
        // Web: store versions in localStorage with timestamp keys
        // This would need integration with web storage
        debugPrint('Version save (web): ${timestamp.millisecondsSinceEpoch}');
      } else {
        // Native: save to .versions directory
        final versionsDir = io.Directory('${_getDocumentDir(_currentPath!)}/.versions');
        if (!await versionsDir.exists()) {
          await versionsDir.create(recursive: true);
        }
        
        final versionPath = '${versionsDir.path}/${timestamp.millisecondsSinceEpoch}.json';
        final versionFile = io.File(versionPath);
        await versionFile.writeAsString(content);
        
        // Cleanup old versions
        await _cleanupOldVersions(versionsDir);
      }
      
      _lastVersion = timestamp;
      _versionCount++;
      
      debugPrint('Version saved: ${timestamp.toIso8601String()}');
      
      // Schedule next version save
      _scheduleVersionSave();
    } catch (e) {
      debugPrint('Version save failed: $e');
    }
  }

  Future<void> _cleanupOldVersions(io.Directory versionsDir) async {
    try {
      final versions = <io.File>[];
      await for (final entity in versionsDir.list()) {
        if (entity is io.File && entity.path.endsWith('.json')) {
          versions.add(entity);
        }
      }
      
      // Sort by name (timestamp), newest first
      versions.sort((a, b) => b.path.compareTo(a.path));
      
      // Remove excess versions
      if (versions.length > maxVersions) {
        for (int i = maxVersions; i < versions.length; i++) {
          await versions[i].delete();
          debugPrint('Deleted old version: ${versions[i].path}');
        }
      }
    } catch (e) {
      debugPrint('Error cleaning up versions: $e');
    }
  }

  String _getDocumentDir(String documentPath) {
    final parts = documentPath.split('/');
    parts.removeLast(); // Remove filename
    return parts.join('/');
  }

  void dispose() {
    stopAutosave();
  }
}

/// Crash recovery information
class CrashRecoveryInfo {
  final String tempPath;
  final Document document;
  final DateTime lastModified;

  const CrashRecoveryInfo({
    required this.tempPath,
    required this.document,
    required this.lastModified,
  });
}

/// Version information
class VersionInfo {
  final String path;
  final Document document;
  final DateTime timestamp;
  final String filename;

  const VersionInfo({
    required this.path,
    required this.document,
    required this.timestamp,
    required this.filename,
  });
}

/// Autosave status
class AutosaveStatus {
  final bool isActive;
  final bool hasUnsavedChanges;
  final DateTime? lastAutosave;
  final DateTime? lastVersion;
  final int autosaveCount;
  final int versionCount;

  const AutosaveStatus({
    required this.isActive,
    required this.hasUnsavedChanges,
    this.lastAutosave,
    this.lastVersion,
    required this.autosaveCount,
    required this.versionCount,
  });
}

/// Provider for autosave service
final autosaveServiceProvider = Provider<AutosaveService>((ref) {
  final storageService = ref.watch(storageServiceProvider);
  final autosaveService = AutosaveService(storageService);
  
  ref.onDispose(() {
    autosaveService.dispose();
  });
  
  return autosaveService;
});