import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../services/editor_service.dart';

class WysiwygEditor extends ConsumerStatefulWidget {
  const WysiwygEditor({super.key});

  @override
  ConsumerState<WysiwygEditor> createState() => _WysiwygEditorState();
}

class _WysiwygEditorState extends ConsumerState<WysiwygEditor> {
  late ScrollController _scrollController;
  
  @override
  void initState() {
    super.initState();
    _scrollController = ScrollController();
  }
  
  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final editorState = ref.watch(editorServiceProvider);
    
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Editor header
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(8),
                topRight: Radius.circular(8),
              ),
            ),
            child: Row(
              children: [
                Icon(
                  Icons.edit_document,
                  size: 16,
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
                const SizedBox(width: 8),
                Text(
                  editorState.document.name,
                  style: Theme.of(context).textTheme.titleSmall?.copyWith(
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const Spacer(),
                if (editorState.hasUnsavedChanges)
                  Container(
                    width: 8,
                    height: 8,
                    decoration: BoxDecoration(
                      color: Theme.of(context).colorScheme.primary,
                      shape: BoxShape.circle,
                    ),
                  ),
              ],
            ),
          ),
          
          // Editor content
          Expanded(
            child: Scrollbar(
              controller: _scrollController,
              child: SingleChildScrollView(
                controller: _scrollController,
                padding: const EdgeInsets.all(24),
                child: _buildDocumentView(editorState.document),
              ),
            ),
          ),
        ],
      ),
    );
  }
  
  Widget _buildDocumentView(DocumentData document) {
    if (document.elements.isEmpty) {
      return _buildEmptyState();
    }
    
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: document.elements.map<Widget>((element) {
        return Padding(
          padding: const EdgeInsets.only(bottom: 16),
          child: _buildElement(element),
        );
      }).toList(),
    );
  }
  
  Widget _buildEmptyState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.description_outlined,
            size: 64,
            color: Theme.of(context).colorScheme.onSurfaceVariant.withOpacity(0.5),
          ),
          const SizedBox(height: 16),
          Text(
            'Document vide',
            style: Theme.of(context).textTheme.headlineSmall?.copyWith(
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            'Commencez à écrire ou importez un fichier',
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              color: Theme.of(context).colorScheme.onSurfaceVariant.withOpacity(0.7),
            ),
          ),
        ],
      ),
    );
  }
  
  Widget _buildElement(dynamic element) {
    if (element is! Map<String, dynamic>) {
      return Text('Invalid element: $element');
    }
    
    final type = element['type'] as String?;
    final content = element['content'];
    
    switch (type) {
      case 'paragraph':
        return _buildParagraph(content);
      case 'heading':
        return _buildHeading(element);
      case 'list':
        return _buildList(element);
      case 'code_block':
        return _buildCodeBlock(element);
      case 'blockquote':
        return _buildBlockquote(content);
      default:
        return _buildGenericElement(element);
    }
  }
  
  Widget _buildParagraph(dynamic content) {
    return SelectableText(
      _extractTextContent(content),
      style: Theme.of(context).textTheme.bodyLarge?.copyWith(
        height: 1.6,
      ),
    );
  }
  
  Widget _buildHeading(Map<String, dynamic> element) {
    final level = element['level'] as int? ?? 1;
    final content = element['content'];
    
    TextStyle? style;
    switch (level) {
      case 1:
        style = Theme.of(context).textTheme.headlineLarge;
        break;
      case 2:
        style = Theme.of(context).textTheme.headlineMedium;
        break;
      case 3:
        style = Theme.of(context).textTheme.headlineSmall;
        break;
      case 4:
        style = Theme.of(context).textTheme.titleLarge;
        break;
      case 5:
        style = Theme.of(context).textTheme.titleMedium;
        break;
      default:
        style = Theme.of(context).textTheme.titleSmall;
    }
    
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: SelectableText(
        _extractTextContent(content),
        style: style?.copyWith(fontWeight: FontWeight.bold),
      ),
    );
  }
  
  Widget _buildList(Map<String, dynamic> element) {
    final items = element['items'] as List<dynamic>? ?? [];
    final isOrdered = element['ordered'] as bool? ?? false;
    
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: items.asMap().entries.map<Widget>((entry) {
        final index = entry.key;
        final item = entry.value;
        final marker = isOrdered ? '${index + 1}. ' : '• ';
        
        return Padding(
          padding: const EdgeInsets.only(left: 16, bottom: 4),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                marker,
                style: Theme.of(context).textTheme.bodyLarge,
              ),
              Expanded(
                child: SelectableText(
                  _extractTextContent(item),
                  style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                    height: 1.6,
                  ),
                ),
              ),
            ],
          ),
        );
      }).toList(),
    );
  }
  
  Widget _buildCodeBlock(Map<String, dynamic> element) {
    final code = element['content'] as String? ?? '';
    final language = element['language'] as String?;
    
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (language != null)
            Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Text(
                language,
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
          SelectableText(
            code,
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              fontFamily: 'RobotoMono',
              fontSize: 14,
            ),
          ),
        ],
      ),
    );
  }
  
  Widget _buildBlockquote(dynamic content) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        border: Border(
          left: BorderSide(
            color: Theme.of(context).colorScheme.primary,
            width: 4,
          ),
        ),
        color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.1),
      ),
      child: SelectableText(
        _extractTextContent(content),
        style: Theme.of(context).textTheme.bodyLarge?.copyWith(
          fontStyle: FontStyle.italic,
          height: 1.6,
        ),
      ),
    );
  }
  
  Widget _buildGenericElement(Map<String, dynamic> element) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.errorContainer.withOpacity(0.1),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(
          color: Theme.of(context).colorScheme.error.withOpacity(0.3),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Unknown element: ${element['type'] ?? 'no type'}',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(context).colorScheme.error,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 4),
          SelectableText(
            element.toString(),
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              fontFamily: 'RobotoMono',
            ),
          ),
        ],
      ),
    );
  }
  
  String _extractTextContent(dynamic content) {
    if (content is String) {
      return content;
    } else if (content is List) {
      return content.map((item) => _extractTextContent(item)).join('');
    } else if (content is Map<String, dynamic>) {
      if (content.containsKey('text')) {
        return content['text'] as String? ?? '';
      } else if (content.containsKey('content')) {
        return _extractTextContent(content['content']);
      }
    }
    return content?.toString() ?? '';
  }
}