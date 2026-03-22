#ifndef RENDER_EXT_H
#define RENDER_EXT_H

#include <stdbool.h>

typedef enum {
  RENDER_EXT_UNKNOWN = 0,
  RENDER_EXT_MERMAID,
  RENDER_EXT_MARKMAP,
  RENDER_EXT_KATEX,
  RENDER_EXT_GRAPHVIZ,
  RENDER_EXT_ECHARTS,
  RENDER_EXT_FLASHCARD
} RenderExtensionType;

RenderExtensionType render_ext_detect_fence(const char *language);
const char *render_ext_type_name(RenderExtensionType type);
bool render_ext_is_supported(RenderExtensionType type);

#endif
