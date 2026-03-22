import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:c_editor_flutter/models/models.dart';
import '../../../widgets/editor_widget.dart';
import '../../../core/state/app_state.dart';
import '../../../widgets/adapters.dart';

class DocumentRenderer extends ConsumerWidget {
  final Document document;
  final ScrollController? scrollController;
  final bool isEditable;

  const DocumentRenderer({
    super.key,
    required this.document,
    this.scrollController,
    this.isEditable = false,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final appConfig = ref.watch(appStateProvider).value;
    
    if (appConfig == null) {
      return const Center(child: CircularProgressIndicator());
    }

    return Container(
      width: double.infinity,
      child: SingleChildScrollView(
        controller: scrollController,
        padding: const EdgeInsets.all(16),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 800),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              for (int i = 0; i < document.elements.length; i++) ...[
                _buildElement(
                  context,
                  document.elements[i],
                  appConfig,
                  i,
                ),
                if (i < document.elements.length - 1)
                  const SizedBox(height: 16),
              ],
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildElement(
    BuildContext context,
    DocElement element,
    AppConfig config,
    int index,
  ) {
    switch (element) {
      case DocTextElement():
        return _buildTextElement(context, element, config);
      case DocImageElement():
        return _buildImageElement(context, element);
      case DocTableElement():
        return _buildTableElement(context, element);
      default:
        return const SizedBox.shrink();
    }
  }

  Widget _buildTextElement(
    BuildContext context,
    DocTextElement element,
    AppConfig config,
  ) {
    // Use header sizes from config
    final fontSize = element.level > 0 
        ? config.headerSizes[element.level] ?? config.fontSize
        : config.fontSize;

    final baseStyle = Theme.of(context).textTheme.bodyMedium?.copyWith(
      fontSize: fontSize,
      fontWeight: element.level > 0 ? FontWeight.bold : null,
      color: element.color != null ? Color.fromRGBO(
        (element.color![0] * 255).round(),
        (element.color![1] * 255).round(),
        (element.color![2] * 255).round(),
        element.color![3],
      ) : null,
      fontFamily: element.font,
    );

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4.0),
      child: RichText(
        textAlign: mapDocAlign(element.align),
        text: TextSpan(
          style: baseStyle,
          children: element.spans.map((span) => mapDocSpan(span)).toList(),
        ),
      ),
    );
  }


  Widget _buildImageElement(BuildContext context, DocImageElement element) {
    return Container(
      width: double.infinity,
      alignment: _getAlignment(mapDocAlign(element.align)),
      child: ConstrainedBox(
        constraints: BoxConstraints(
          maxWidth: element.width?.toDouble() ?? double.infinity,
          maxHeight: element.height?.toDouble() ?? double.infinity,
        ),
        child: Opacity(
          opacity: element.alpha,
          child: element.src.startsWith('http')
              ? Image.network(
                  element.src,
                  errorBuilder: (context, error, stackTrace) => Container(
                    padding: const EdgeInsets.all(16),
                    decoration: BoxDecoration(
                      color: Colors.grey[300],
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const Icon(Icons.broken_image, size: 48),
                        const SizedBox(height: 8),
                        Text(element.alt.isNotEmpty ? element.alt : 'Image'),
                      ],
                    ),
                  ),
                )
              : Image.asset(
                  element.src,
                  errorBuilder: (context, error, stackTrace) => Container(
                    padding: const EdgeInsets.all(16),
                    decoration: BoxDecoration(
                      color: Colors.grey[300],
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const Icon(Icons.broken_image, size: 48),
                        const SizedBox(height: 8),
                        Text(element.alt.isNotEmpty ? element.alt : 'Image'),
                      ],
                    ),
                  ),
                ),
        ),
      ),
    );
  }

  Widget _buildTableElement(BuildContext context, DocTableElement element) {
    return Container(
      width: double.infinity,
      decoration: BoxDecoration(
        border: Border.all(color: Theme.of(context).dividerColor),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Table(
        border: TableBorder.all(
          color: Theme.of(context).dividerColor,
          width: 1,
        ),
        children: element.rows.asMap().entries.map((rowEntry) {
          final isHeader = rowEntry.key == 0;
          final row = rowEntry.value;
          
          return TableRow(
            decoration: isHeader
                ? BoxDecoration(
                    color: Theme.of(context).colorScheme.surface,
                  )
                : null,
            children: row.map((cell) {
              return TableCell(
                child: Padding(
                  padding: const EdgeInsets.all(8.0),
                  child: RichText(
                    text: TextSpan(
                      style: isHeader 
                          ? Theme.of(context).textTheme.titleSmall
                          : Theme.of(context).textTheme.bodyMedium,
                      children: cell.map((span) => mapDocSpan(span)).toList(),
                    ),
                  ),
                ),
              );
            }).toList(),
          );
        }).toList(),
      ),
    );
  }

  Alignment _getAlignment(TextAlign align) {
    switch (align) {
      case TextAlign.left:
        return Alignment.centerLeft;
      case TextAlign.center:
        return Alignment.center;
      case TextAlign.right:
        return Alignment.centerRight;
      case TextAlign.justify:
        return Alignment.centerLeft;
      default:
        return Alignment.centerLeft;
    }
  }
}