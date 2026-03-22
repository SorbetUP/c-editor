// SearchBridge.h - Bridge entre les structures C et Objective-C

#ifndef SearchBridge_h
#define SearchBridge_h

#include "../advanced_search/advanced_search.h"
#include "search_interface.h"

// Redéfinition des types C pour éviter conflits avec Objective-C
typedef SearchResult SearchResult_C;
typedef FileTreeNode FileTreeNode_C;

#endif /* SearchBridge_h */