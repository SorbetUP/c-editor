#include "vault_core.h"

#include <ctype.h>
#include <string.h>

static bool has_markdown_extension(const char *path) {
  const char *dot = strrchr(path, '.');
  if (!dot) {
    return false;
  }

  return strcmp(dot, ".md") == 0 || strcmp(dot, ".markdown") == 0;
}

void vault_core_init(VaultCoreConfig *config) {
  if (!config) {
    return;
  }

  config->root_path[0] = '\0';
  config->is_configured = false;
}

bool vault_core_is_valid_root(const char *path) {
  size_t i = 0;

  if (!path || path[0] != '/') {
    return false;
  }

  while (path[i] != '\0') {
    if ((unsigned char)path[i] < 32) {
      return false;
    }
    i++;
  }

  return i > 1 && i < VAULT_CORE_MAX_PATH;
}

bool vault_core_set_root(VaultCoreConfig *config, const char *path) {
  size_t len;

  if (!config || !vault_core_is_valid_root(path)) {
    return false;
  }

  len = strlen(path);
  memcpy(config->root_path, path, len + 1);
  config->is_configured = true;
  return true;
}

const char *vault_core_get_root(const VaultCoreConfig *config) {
  if (!config || !config->is_configured) {
    return NULL;
  }

  return config->root_path;
}

bool vault_core_is_note_path(const char *path) {
  if (!path || path[0] == '\0') {
    return false;
  }

  return has_markdown_extension(path);
}
