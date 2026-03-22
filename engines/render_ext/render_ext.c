#include "render_ext.h"

#include <string.h>

RenderExtensionType render_ext_detect_fence(const char *language) {
  if (!language) {
    return RENDER_EXT_UNKNOWN;
  }

  if (strcmp(language, "mermaid") == 0) return RENDER_EXT_MERMAID;
  if (strcmp(language, "markmap") == 0) return RENDER_EXT_MARKMAP;
  if (strcmp(language, "katex") == 0 || strcmp(language, "math") == 0)
    return RENDER_EXT_KATEX;
  if (strcmp(language, "graphviz") == 0 || strcmp(language, "dot") == 0)
    return RENDER_EXT_GRAPHVIZ;
  if (strcmp(language, "echarts") == 0) return RENDER_EXT_ECHARTS;
  if (strcmp(language, "flashcard") == 0) return RENDER_EXT_FLASHCARD;

  return RENDER_EXT_UNKNOWN;
}

const char *render_ext_type_name(RenderExtensionType type) {
  switch (type) {
  case RENDER_EXT_MERMAID:
    return "mermaid";
  case RENDER_EXT_MARKMAP:
    return "markmap";
  case RENDER_EXT_KATEX:
    return "katex";
  case RENDER_EXT_GRAPHVIZ:
    return "graphviz";
  case RENDER_EXT_ECHARTS:
    return "echarts";
  case RENDER_EXT_FLASHCARD:
    return "flashcard";
  default:
    return "unknown";
  }
}

bool render_ext_is_supported(RenderExtensionType type) {
  return type != RENDER_EXT_UNKNOWN;
}
