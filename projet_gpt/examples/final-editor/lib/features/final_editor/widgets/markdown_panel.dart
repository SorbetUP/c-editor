import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../services/editor_service.dart';

class MarkdownPanel extends ConsumerStatefulWidget {
  final VoidCallback onClose;

  const MarkdownPanel({
    super.key,
    required this.onClose,
  });

  @override
  ConsumerState<MarkdownPanel> createState() => _MarkdownPanelState();
}

class _MarkdownPanelState extends ConsumerState<MarkdownPanel> {
  late TextEditingController _controller;
  late ScrollController _scrollController;
  String _lastKnownMarkdown = '';

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController();
    _scrollController = ScrollController();
    
    // Initialize with current markdown
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _updateMarkdownFromService();
    });
  }

  @override
  void dispose() {
    _controller.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _updateMarkdownFromService() {
    final editorService = ref.read(editorServiceProvider.notifier);
    final newMarkdown = editorService.getMarkdown();
    
    if (newMarkdown != _lastKnownMarkdown) {
      _lastKnownMarkdown = newMarkdown;
      _controller.text = newMarkdown;
    }
  }

  void _updateServiceFromMarkdown() {
    final markdown = _controller.text;
    if (markdown != _lastKnownMarkdown) {
      _lastKnownMarkdown = markdown;
      ref.read(editorServiceProvider.notifier).updateFromMarkdown(markdown);
    }
  }

  @override
  Widget build(BuildContext context) {
    // Listen for changes to update markdown display
    ref.listen(editorServiceProvider, (_, __) {
      _updateMarkdownFromService();
    });

    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
        ),
      ),
      child: Column(
        children: [
          // Panel header
          Container(
            padding: const EdgeInsets.all(12),
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
                  Icons.notes,
                  size: 16,
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
                const SizedBox(width: 8),
                Text(
                  'Markdown',
                  style: Theme.of(context).textTheme.titleSmall?.copyWith(
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const Spacer(),
                IconButton(
                  onPressed: widget.onClose,
                  iconSize: 16,
                  constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
                  padding: EdgeInsets.zero,
                  icon: Icon(
                    Icons.close,
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
                  tooltip: 'Fermer le panneau Markdown',
                ),
              ],
            ),
          ),
          
          // Markdown editor
          Expanded(
            child: Scrollbar(
              controller: _scrollController,
              child: TextField(
                controller: _controller,
                scrollController: _scrollController,
                maxLines: null,
                expands: true,
                textAlignVertical: TextAlignVertical.top,
                onChanged: (_) => _updateServiceFromMarkdown(),
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                  fontFamily: 'RobotoMono',
                  fontSize: 13,
                  height: 1.4,
                ),
                decoration: InputDecoration(
                  hintText: 'Tapez votre Markdown ici...',
                  hintStyle: TextStyle(
                    color: Theme.of(context).colorScheme.onSurfaceVariant.withOpacity(0.6),
                  ),
                  border: InputBorder.none,
                  contentPadding: const EdgeInsets.all(16),
                ),
              ),
            ),
          ),
          
          // Panel footer with info
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.1),
              borderRadius: const BorderRadius.only(
                bottomLeft: Radius.circular(8),
                bottomRight: Radius.circular(8),
              ),
            ),
            child: Row(
              children: [
                Icon(
                  Icons.info_outline,
                  size: 12,
                  color: Theme.of(context).colorScheme.onSurfaceVariant.withOpacity(0.7),
                ),
                const SizedBox(width: 4),
                Text(
                  'Les changements sont synchronisés automatiquement',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant.withOpacity(0.7),
                  ),
                ),
                const Spacer(),
                Text(
                  '${_controller.text.length} caractères',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant.withOpacity(0.7),
                    fontFamily: 'RobotoMono',
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}