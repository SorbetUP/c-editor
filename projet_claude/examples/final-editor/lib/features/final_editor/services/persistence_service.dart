import 'dart:convert';
import 'package:idb_shim/idb_browser.dart';
import '../services/editor_service.dart';

/// Service for persisting editor data using IndexedDB
class PersistenceService {
  static const String _dbName = 'final_editor_db';
  static const String _documentsStore = 'documents';
  static const String _sessionsStore = 'sessions';
  static const String _crashRecoveryStore = 'crash_recovery';
  static const int _dbVersion = 1;
  
  Database? _database;
  bool _initialized = false;

  /// Initialize the persistence service
  Future<void> initialize() async {
    if (_initialized) return;
    
    try {
      final factory = getIdbFactory()!;
      
      _database = await factory.open(_dbName, version: _dbVersion, onUpgradeNeeded: (VersionChangeEvent event) {
        final db = event.database;
        
        // Documents store
        if (!db.objectStoreNames.contains(_documentsStore)) {
          final documentsStore = db.createObjectStore(_documentsStore, keyPath: 'id');
          documentsStore.createIndex('name', 'name', unique: false);
          documentsStore.createIndex('lastModified', 'lastModified', unique: false);
        }
        
        // Sessions store
        if (!db.objectStoreNames.contains(_sessionsStore)) {
          final sessionsStore = db.createObjectStore(_sessionsStore, keyPath: 'id');
          sessionsStore.createIndex('timestamp', 'timestamp', unique: false);
        }
        
        // Crash recovery store
        if (!db.objectStoreNames.contains(_crashRecoveryStore)) {
          db.createObjectStore(_crashRecoveryStore, keyPath: 'id');
        }
      });
      
      _initialized = true;
      print('✅ Persistence service initialized');
      
    } catch (e) {
      print('❌ Failed to initialize persistence: $e');
      rethrow;
    }
  }

  /// Save document data
  Future<void> saveDocument(DocumentData document, {String? id}) async {
    if (!_initialized) await initialize();
    
    try {
      final transaction = _database!.transaction([_documentsStore], 'readwrite');
      final store = transaction.objectStore(_documentsStore);
      
      final docId = id ?? 'current_document';
      final documentRecord = {
        'id': docId,
        'name': document.name,
        'metadata': document.metadata,
        'elements': document.elements,
        'lastModified': DateTime.now().toIso8601String(),
      };
      
      await store.put(documentRecord);
      await transaction.completed;
      
    } catch (e) {
      print('❌ Failed to save document: $e');
      rethrow;
    }
  }

  /// Load document data
  Future<DocumentData?> loadDocument({String? id}) async {
    if (!_initialized) await initialize();
    
    try {
      final transaction = _database!.transaction([_documentsStore], 'readonly');
      final store = transaction.objectStore(_documentsStore);
      
      final docId = id ?? 'current_document';
      final record = await store.getObject(docId);
      
      if (record != null) {
        return DocumentData(
          name: record['name'] as String? ?? 'Untitled',
          metadata: Map<String, dynamic>.from(record['metadata'] as Map? ?? {}),
          elements: List<dynamic>.from(record['elements'] as List? ?? []),
        );
      }
      
      return null;
      
    } catch (e) {
      print('❌ Failed to load document: $e');
      return null;
    }
  }

  /// Save session data (last opened document, settings, etc.)
  Future<void> saveSession(DocumentData document) async {
    if (!_initialized) await initialize();
    
    try {
      final transaction = _database!.transaction([_sessionsStore], 'readwrite');
      final store = transaction.objectStore(_sessionsStore);
      
      final sessionRecord = {
        'id': 'last_session',
        'document': {
          'name': document.name,
          'metadata': document.metadata,
          'elements': document.elements,
        },
        'timestamp': DateTime.now().toIso8601String(),
      };
      
      await store.put(sessionRecord);
      await transaction.completed;
      
    } catch (e) {
      print('❌ Failed to save session: $e');
      rethrow;
    }
  }

  /// Load last session data
  Future<DocumentData?> loadLastSession() async {
    if (!_initialized) await initialize();
    
    try {
      final transaction = _database!.transaction([_sessionsStore], 'readonly');
      final store = transaction.objectStore(_sessionsStore);
      
      final record = await store.getObject('last_session');
      
      if (record != null) {
        final docData = record['document'] as Map<String, dynamic>;
        return DocumentData(
          name: docData['name'] as String? ?? 'Untitled',
          metadata: Map<String, dynamic>.from(docData['metadata'] as Map? ?? {}),
          elements: List<dynamic>.from(docData['elements'] as List? ?? []),
        );
      }
      
      return null;
      
    } catch (e) {
      print('❌ Failed to load last session: $e');
      return null;
    }
  }

  /// Save crash recovery data
  Future<void> saveCrashRecovery(DocumentData document) async {
    if (!_initialized) await initialize();
    
    try {
      final transaction = _database!.transaction([_crashRecoveryStore], 'readwrite');
      final store = transaction.objectStore(_crashRecoveryStore);
      
      final recoveryRecord = {
        'id': 'crash_recovery',
        'document': {
          'name': document.name,
          'metadata': document.metadata,
          'elements': document.elements,
        },
        'timestamp': DateTime.now().toIso8601String(),
      };
      
      await store.put(recoveryRecord);
      await transaction.completed;
      
    } catch (e) {
      print('❌ Failed to save crash recovery: $e');
    }
  }

  /// Load crash recovery data
  Future<DocumentData?> getCrashRecovery() async {
    if (!_initialized) await initialize();
    
    try {
      final transaction = _database!.transaction([_crashRecoveryStore], 'readonly');
      final store = transaction.objectStore(_crashRecoveryStore);
      
      final record = await store.getObject('crash_recovery');
      
      if (record != null) {
        final docData = record['document'] as Map<String, dynamic>;
        final timestamp = record['timestamp'] as String;
        
        // Check if crash recovery is recent (within last hour)
        final recoveryTime = DateTime.parse(timestamp);
        final now = DateTime.now();
        final difference = now.difference(recoveryTime);
        
        if (difference.inHours < 1) {
          return DocumentData(
            name: docData['name'] as String? ?? 'Untitled',
            metadata: Map<String, dynamic>.from(docData['metadata'] as Map? ?? {}),
            elements: List<dynamic>.from(docData['elements'] as List? ?? []),
          );
        }
      }
      
      return null;
      
    } catch (e) {
      print('❌ Failed to get crash recovery: $e');
      return null;
    }
  }

  /// Clear crash recovery data
  Future<void> clearCrashRecovery() async {
    if (!_initialized) await initialize();
    
    try {
      final transaction = _database!.transaction([_crashRecoveryStore], 'readwrite');
      final store = transaction.objectStore(_crashRecoveryStore);
      
      await store.delete('crash_recovery');
      await transaction.completed;
      
    } catch (e) {
      print('❌ Failed to clear crash recovery: $e');
    }
  }

  /// Get all saved documents
  Future<List<DocumentInfo>> getAllDocuments() async {
    if (!_initialized) await initialize();
    
    try {
      final transaction = _database!.transaction([_documentsStore], 'readonly');
      final store = transaction.objectStore(_documentsStore);
      
      final documents = <DocumentInfo>[];
      final cursor = store.openCursor();
      
      await cursor.listen((cursorWithValue) {
        if (cursorWithValue == null) return;
        
        final record = cursorWithValue.value;
        documents.add(DocumentInfo(
          id: record['id'] as String,
          name: record['name'] as String,
          lastModified: DateTime.parse(record['lastModified'] as String),
        ));
        
        cursorWithValue.next();
      }).asFuture();
      
      // Sort by last modified (newest first)
      documents.sort((a, b) => b.lastModified.compareTo(a.lastModified));
      
      return documents;
      
    } catch (e) {
      print('❌ Failed to get all documents: $e');
      return [];
    }
  }

  /// Delete a document
  Future<void> deleteDocument(String id) async {
    if (!_initialized) await initialize();
    
    try {
      final transaction = _database!.transaction([_documentsStore], 'readwrite');
      final store = transaction.objectStore(_documentsStore);
      
      await store.delete(id);
      await transaction.completed;
      
    } catch (e) {
      print('❌ Failed to delete document: $e');
      rethrow;
    }
  }

  /// Clear all data (for debugging/reset)
  Future<void> clearAllData() async {
    if (!_initialized) await initialize();
    
    try {
      final transaction = _database!.transaction([
        _documentsStore,
        _sessionsStore,
        _crashRecoveryStore,
      ], 'readwrite');
      
      await transaction.objectStore(_documentsStore).clear();
      await transaction.objectStore(_sessionsStore).clear();
      await transaction.objectStore(_crashRecoveryStore).clear();
      
      await transaction.completed;
      
      print('✅ All data cleared');
      
    } catch (e) {
      print('❌ Failed to clear data: $e');
      rethrow;
    }
  }

  /// Close the database connection
  Future<void> close() async {
    if (_database != null) {
      _database!.close();
      _database = null;
      _initialized = false;
    }
  }
}

/// Document information for listing
class DocumentInfo {
  final String id;
  final String name;
  final DateTime lastModified;

  const DocumentInfo({
    required this.id,
    required this.name,
    required this.lastModified,
  });
}