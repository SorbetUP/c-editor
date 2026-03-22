import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'dart:convert';
import '../services/editor_service.dart';

class JsonPanel extends ConsumerStatefulWidget {
  final VoidCallback onClose;

  const JsonPanel({
    super.key,
    required this.onClose,
  });

  @override
  ConsumerState<JsonPanel> createState() => _JsonPanelState();
}

class _JsonPanelState extends ConsumerState<JsonPanel> {
  late TextEditingController _controller;
  late ScrollController _scrollController;
  String _lastKnownJson = '';
  bool _hasJsonError = false;
  String? _jsonErrorMessage;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController();
    _scrollController = ScrollController();
    
    // Initialize with current JSON
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _updateJsonFromService();
    });
  }

  @override
  void dispose() {
    _controller.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _updateJsonFromService() {
    final editorService = ref.read(editorServiceProvider.notifier);
    final newJson = editorService.getCanonicalJson();
    
    if (newJson != _lastKnownJson) {
      _lastKnownJson = newJson;
      _controller.text = _formatJson(newJson);
      _hasJsonError = false;
      _jsonErrorMessage = null;
    }
  }

  void _updateServiceFromJson() {
    final json = _controller.text;
    if (json.trim().isEmpty) {
      return;
    }
    
    try {
      // Validate JSON first
      jsonDecode(json);
      
      // Update service if valid
      if (json != _lastKnownJson) {
        ref.read(editorServiceProvider.notifier).updateFromJson(json);
        _lastKnownJson = json;
        _hasJsonError = false;
        _jsonErrorMessage = null;
      }
    } catch (e) {
      _hasJsonError = true;
      _jsonErrorMessage = e.toString();
    }
  }

  String _formatJson(String json) {
    try {
      final decoded = jsonDecode(json);
      const encoder = JsonEncoder.withIndent('  ');
      return encoder.convert(decoded);
    } catch (e) {
      return json;
    }
  }

  @override
  Widget build(BuildContext context) {
    // Listen for changes to update JSON display
    ref.listen(editorServiceProvider, (_, __) {
      _updateJsonFromService();
    });

    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: _hasJsonError
            ? Theme.of(context).colorScheme.error.withOpacity(0.5)
            : Theme.of(context).colorScheme.outline.withOpacity(0.2),
        ),
      ),
      child: Column(
        children: [
          // Panel header
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: _hasJsonError
                ? Theme.of(context).colorScheme.errorContainer.withOpacity(0.1)
                : Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(8),
                topRight: Radius.circular(8),
              ),
            ),
            child: Row(
              children: [
                Icon(
                  _hasJsonError ? Icons.error_outline : Icons.code,
                  size: 16,
                  color: _hasJsonError
                    ? Theme.of(context).colorScheme.error
                    : Theme.of(context).colorScheme.onSurfaceVariant,
                ),
                const SizedBox(width: 8),
                Text(
                  'JSON Structure',
                  style: Theme.of(context).textTheme.titleSmall?.copyWith(
                    fontWeight: FontWeight.w600,
                    color: _hasJsonError
                      ? Theme.of(context).colorScheme.error
                      : null,
                  ),
                ),
                const Spacer(),
                if (_hasJsonError)
                  Tooltip(
                    message: _jsonErrorMessage ?? 'Invalid JSON',
                    child: Icon(
                      Icons.warning,
                      size: 16,
                      color: Theme.of(context).colorScheme.error,
                    ),
                  ),
                const SizedBox(width: 8),
                IconButton(
                  onPressed: widget.onClose,
                  iconSize: 16,
                  constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
                  padding: EdgeInsets.zero,
                  icon: Icon(
                    Icons.close,
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
                  tooltip: 'Fermer le panneau JSON',
                ),
              ],
            ),
          ),
          
          // JSON editor
          Expanded(
            child: Scrollbar(
              controller: _scrollController,
              child: TextField(
                controller: _controller,
                scrollController: _scrollController,
                maxLines: null,
                expands: true,
                textAlignVertical: TextAlignVertical.top,
                onChanged: (_) => _updateServiceFromJson(),
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                  fontFamily: 'RobotoMono',
                  fontSize: 12,
                  height: 1.4,
                  color: _hasJsonError
                    ? Theme.of(context).colorScheme.error
                    : null,
                ),
                decoration: InputDecoration(
                  hintText: '{\n  "name": "Document",\n  "meta": {},\n  "elements": []\n}',
                  hintStyle: TextStyle(
                    color: Theme.of(context).colorScheme.onSurfaceVariant.withOpacity(0.6),
                  ),
                  border: InputBorder.none,
                  contentPadding: const EdgeInsets.all(16),
                  fillColor: _hasJsonError
                    ? Theme.of(context).colorScheme.errorContainer.withOpacity(0.05)
                    : null,
                  filled: _hasJsonError,
                ),
              ),
            ),
          ),
          
          // Panel footer with info/error
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: _hasJsonError
                ? Theme.of(context).colorScheme.errorContainer.withOpacity(0.1)
                : Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.1),
              borderRadius: const BorderRadius.only(
                bottomLeft: Radius.circular(8),
                bottomRight: Radius.circular(8),
              ),
            ),
            child: _hasJsonError ? _buildErrorFooter() : _buildInfoFooter(),
          ),
        ],
      ),
    );
  }

  Widget _buildInfoFooter() {
    return Row(
      children: [
        Icon(
          Icons.info_outline,
          size: 12,
          color: Theme.of(context).colorScheme.onSurfaceVariant.withOpacity(0.7),
        ),
        const SizedBox(width: 4),
        Expanded(
          child: Text(
            'Édition directe de la structure du document',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(context).colorScheme.onSurfaceVariant.withOpacity(0.7),
            ),
          ),
        ),
        Text(
          '${_controller.text.length} caractères',
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
            color: Theme.of(context).colorScheme.onSurfaceVariant.withOpacity(0.7),
            fontFamily: 'RobotoMono',
          ),
        ),
      ],
    );
  }

  Widget _buildErrorFooter() {
    return Row(
      children: [
        Icon(
          Icons.error_outline,
          size: 12,
          color: Theme.of(context).colorScheme.error,
        ),
        const SizedBox(width: 4),
        Expanded(
          child: Text(
            'JSON invalide: ${_jsonErrorMessage?.substring(0, 50) ?? 'Erreur de syntaxe'}${(_jsonErrorMessage?.length ?? 0) > 50 ? '...' : ''}',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(context).colorScheme.error,
            ),
          ),
        ),
        TextButton(
          onPressed: () {
            // Format JSON to fix common issues
            try {
              final formatted = _formatJson(_controller.text);
              _controller.text = formatted;
              _updateServiceFromJson();
            } catch (e) {
              // Ignore if still invalid
            }
          },
          style: TextButton.styleFrom(
            minimumSize: Size.zero,
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          ),
          child: Text(
            'Format',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(context).colorScheme.primary,
            ),
          ),
        ),
      ],
    );
  }
}