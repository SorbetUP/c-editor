#ifndef VAULT_CORE_H
#define VAULT_CORE_H

#include <stdbool.h>
#include <stddef.h>

#define VAULT_CORE_MAX_PATH 1024

typedef struct {
  char root_path[VAULT_CORE_MAX_PATH];
  bool is_configured;
} VaultCoreConfig;

void vault_core_init(VaultCoreConfig *config);
bool vault_core_set_root(VaultCoreConfig *config, const char *path);
const char *vault_core_get_root(const VaultCoreConfig *config);
bool vault_core_is_valid_root(const char *path);
bool vault_core_is_note_path(const char *path);

#endif
