#include "link_engine.h"

#include <string.h>

void link_engine_init(LinkEngineReferenceList *list) {
  if (!list) {
    return;
  }

  memset(list, 0, sizeof(*list));
}

bool link_engine_extract_wikilinks(const char *markdown,
                                   LinkEngineReferenceList *list) {
  size_t i = 0;

  if (!markdown || !list) {
    return false;
  }

  link_engine_init(list);

  while (markdown[i] != '\0' && list->count < LINK_ENGINE_MAX_LINKS) {
    size_t j;
    size_t len;
    LinkEngineReference *ref;

    if (!(markdown[i] == '[' && markdown[i + 1] == '[')) {
      i++;
      continue;
    }

    j = i + 2;
    while (markdown[j] != '\0' && !(markdown[j] == ']' && markdown[j + 1] == ']')) {
      j++;
    }

    if (markdown[j] == '\0') {
      break;
    }

    len = j - (i + 2);
    if (len == 0 || len >= LINK_ENGINE_MAX_LABEL) {
      i = j + 2;
      continue;
    }

    ref = &list->items[list->count++];
    memcpy(ref->target, markdown + i + 2, len);
    ref->target[len] = '\0';
    ref->start_offset = i;
    ref->end_offset = j + 2;
    i = j + 2;
  }

  return true;
}
