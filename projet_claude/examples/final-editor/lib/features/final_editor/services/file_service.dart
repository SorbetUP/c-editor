import 'dart:convert';
import 'dart:typed_data';
import 'package:file_picker/file_picker.dart';
import 'package:web/web.dart' as web;

/// Service for handling file import/export operations
class FileService {
  
  /// Import a file (markdown or JSON) from the file system
  Future<String?> importFile(String type) async {
    try {
      List<String> allowedExtensions;
      String fileTypeDescription;
      
      switch (type.toLowerCase()) {
        case 'markdown':
        case 'md':
          allowedExtensions = ['md', 'markdown', 'txt'];
          fileTypeDescription = 'Markdown files';
          break;
        case 'json':
          allowedExtensions = ['json'];
          fileTypeDescription = 'JSON files';
          break;
        default:
          throw ArgumentError('Unsupported file type: $type');
      }
      
      // Use file_picker to select file
      final result = await FilePicker.platform.pickFiles(
        type: FileType.custom,
        allowedExtensions: allowedExtensions,
        allowMultiple: false,
        withData: true,
      );
      
      if (result != null && result.files.isNotEmpty) {
        final file = result.files.first;
        
        if (file.bytes != null) {
          // Decode bytes to string
          final content = utf8.decode(file.bytes!);
          return content;
        } else {
          throw Exception('Could not read file data');
        }
      }
      
      return null; // User cancelled
      
    } catch (e) {
      print('❌ Error importing file: $e');
      rethrow;
    }
  }
  
  /// Export content to a file
  Future<void> exportFile(String content, String filename, String type) async {
    try {
      // Convert content to bytes
      final bytes = utf8.encode(content);
      final blob = web.Blob([bytes.buffer.asUint8List()].toJS);
      
      // Create download URL
      final url = web.URL.createObjectURL(blob);
      
      // Create temporary anchor element for download
      final anchor = web.document.createElement('a') as web.HTMLAnchorElement;
      anchor.href = url;
      anchor.download = filename;
      anchor.style.display = 'none';
      
      // Add to DOM, click, and remove
      web.document.body!.appendChild(anchor);
      anchor.click();
      web.document.body!.removeChild(anchor);
      
      // Clean up object URL
      web.URL.revokeObjectURL(url);
      
      print('✅ File exported: $filename');
      
    } catch (e) {
      print('❌ Error exporting file: $e');
      rethrow;
    }
  }
  
  /// Validate file content based on type
  bool validateFileContent(String content, String type) {
    switch (type.toLowerCase()) {
      case 'json':
        try {
          jsonDecode(content);
          return true;
        } catch (e) {
          return false;
        }
      case 'markdown':
      case 'md':
        // Markdown is more lenient - just check if it's valid text
        return content.isNotEmpty;
      default:
        return false;
    }
  }
  
  /// Get MIME type for a file type
  String getMimeType(String type) {
    switch (type.toLowerCase()) {
      case 'json':
        return 'application/json';
      case 'markdown':
      case 'md':
        return 'text/markdown';
      default:
        return 'text/plain';
    }
  }
  
  /// Suggest filename based on document name and type
  String suggestFilename(String documentName, String type) {
    // Clean document name
    final cleanName = documentName
        .replaceAll(RegExp(r'[^\w\s-]'), '')
        .replaceAll(RegExp(r'\s+'), '_')
        .toLowerCase();
    
    final baseName = cleanName.isEmpty ? 'document' : cleanName;
    
    switch (type.toLowerCase()) {
      case 'json':
        return '$baseName.json';
      case 'markdown':
      case 'md':
        return '$baseName.md';
      default:
        return '$baseName.txt';
    }
  }
}