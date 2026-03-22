#ifndef LINK_ENGINE_H
#define LINK_ENGINE_H

#include <stdbool.h>
#include <stddef.h>

#define LINK_ENGINE_MAX_LINKS 64
#define LINK_ENGINE_MAX_LABEL 256

typedef struct {
  char target[LINK_ENGINE_MAX_LABEL];
  size_t start_offset;
  size_t end_offset;
} LinkEngineReference;

typedef struct {
  LinkEngineReference items[LINK_ENGINE_MAX_LINKS];
  size_t count;
} LinkEngineReferenceList;

void link_engine_init(LinkEngineReferenceList *list);
bool link_engine_extract_wikilinks(const char *markdown,
                                   LinkEngineReferenceList *list);

#endif
