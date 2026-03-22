import 'package:flutter/material.dart';

class EditorToolbar extends StatelessWidget {
  final VoidCallback onImportMarkdown;
  final VoidCallback onImportJson;
  final VoidCallback onExportMarkdown;
  final VoidCallback onExportJson;
  final VoidCallback onToggleMarkdown;
  final VoidCallback onToggleJson;
  final bool showMarkdownPanel;
  final bool showJsonPanel;

  const EditorToolbar({
    super.key,
    required this.onImportMarkdown,
    required this.onImportJson,
    required this.onExportMarkdown,
    required this.onExportJson,
    required this.onToggleMarkdown,
    required this.onToggleJson,
    required this.showMarkdownPanel,
    required this.showJsonPanel,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      height: 56,
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
        border: Border(
          bottom: BorderSide(
            color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
          ),
        ),
      ),
      child: Row(
        children: [
          const SizedBox(width: 16),
          
          // App icon and title
          Icon(
            Icons.edit_document,
            color: Theme.of(context).colorScheme.primary,
            size: 24,
          ),
          const SizedBox(width: 8),
          Text(
            'Final Editor',
            style: Theme.of(context).textTheme.titleMedium?.copyWith(
              fontWeight: FontWeight.w600,
              color: Theme.of(context).colorScheme.primary,
            ),
          ),
          
          const SizedBox(width: 32),
          
          // Import section
          _buildSection(
            context,
            'Import',
            [
              _ToolbarButton(
                icon: Icons.file_open,
                tooltip: 'Import Markdown (Ctrl+O)',
                onPressed: onImportMarkdown,
              ),
              _ToolbarButton(
                icon: Icons.data_object,
                tooltip: 'Import JSON',
                onPressed: onImportJson,
              ),
            ],
          ),
          
          _buildDivider(context),
          
          // Export section
          _buildSection(
            context,
            'Export',
            [
              _ToolbarButton(
                icon: Icons.download,
                tooltip: 'Export Markdown',
                onPressed: onExportMarkdown,
              ),
              _ToolbarButton(
                icon: Icons.save_alt,
                tooltip: 'Export JSON (Ctrl+S)',
                onPressed: onExportJson,
              ),
            ],
          ),
          
          _buildDivider(context),
          
          // View panels section
          _buildSection(
            context,
            'Panels',
            [
              _ToolbarButton(
                icon: Icons.notes,
                tooltip: 'Toggle Markdown Panel',
                onPressed: onToggleMarkdown,
                isActive: showMarkdownPanel,
              ),
              _ToolbarButton(
                icon: Icons.code,
                tooltip: 'Toggle JSON Panel',
                onPressed: onToggleJson,
                isActive: showJsonPanel,
              ),
            ],
          ),
          
          const Spacer(),
          
          // Help section
          _ToolbarButton(
            icon: Icons.help_outline,
            tooltip: 'Help & Shortcuts',
            onPressed: () => _showHelpDialog(context),
          ),
          
          const SizedBox(width: 16),
        ],
      ),
    );
  }
  
  Widget _buildSection(
    BuildContext context,
    String title,
    List<Widget> buttons,
  ) {
    return Row(
      children: [
        Text(
          title,
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
            color: Theme.of(context).colorScheme.onSurfaceVariant,
            fontWeight: FontWeight.w600,
          ),
        ),
        const SizedBox(width: 8),
        ...buttons,
      ],
    );
  }
  
  Widget _buildDivider(BuildContext context) {
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      width: 1,
      color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
    );
  }
  
  void _showHelpDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Raccourcis clavier'),
        content: SizedBox(
          width: 400,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildShortcutItem('Ctrl/Cmd + B', 'Gras'),
              _buildShortcutItem('Ctrl/Cmd + I', 'Italique'),
              _buildShortcutItem('Ctrl/Cmd + U', 'Souligné'),
              _buildShortcutItem('Ctrl/Cmd + Z', 'Annuler'),
              _buildShortcutItem('Ctrl/Cmd + Y', 'Rétablir'),
              _buildShortcutItem('Ctrl/Cmd + A', 'Tout sélectionner'),
              _buildShortcutItem('Ctrl/Cmd + S', 'Exporter JSON'),
              _buildShortcutItem('Ctrl/Cmd + O', 'Importer Markdown'),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Fermer'),
          ),
        ],
      ),
    );
  }
  
  Widget _buildShortcutItem(String shortcut, String description) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          SizedBox(
            width: 120,
            child: Text(
              shortcut,
              style: const TextStyle(
                fontFamily: 'RobotoMono',
                fontSize: 12,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
          Text(description),
        ],
      ),
    );
  }
}

class _ToolbarButton extends StatelessWidget {
  final IconData icon;
  final String tooltip;
  final VoidCallback onPressed;
  final bool isActive;

  const _ToolbarButton({
    required this.icon,
    required this.tooltip,
    required this.onPressed,
    this.isActive = false,
  });

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: tooltip,
      child: Container(
        margin: const EdgeInsets.symmetric(horizontal: 2),
        child: Material(
          color: isActive 
            ? Theme.of(context).colorScheme.primary.withOpacity(0.1)
            : Colors.transparent,
          borderRadius: BorderRadius.circular(6),
          child: InkWell(
            onTap: onPressed,
            borderRadius: BorderRadius.circular(6),
            child: Container(
              padding: const EdgeInsets.all(8),
              child: Icon(
                icon,
                size: 18,
                color: isActive
                  ? Theme.of(context).colorScheme.primary
                  : Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ),
        ),
      ),
    );
  }
}