#include "sync_engine.h"

#include <string.h>

void sync_engine_init(SyncEngineConfig *config) {
  if (!config) {
    return;
  }

  memset(config, 0, sizeof(*config));
  config->mode = SYNC_ENGINE_MODE_LOCAL;
}

bool sync_engine_set_remote(SyncEngineConfig *config, const char *endpoint) {
  size_t len;

  if (!config || !endpoint) {
    return false;
  }

  if (!(strncmp(endpoint, "http://", 7) == 0 ||
        strncmp(endpoint, "https://", 8) == 0)) {
    return false;
  }

  len = strlen(endpoint);
  if (len == 0 || len >= sizeof(config->remote_endpoint)) {
    return false;
  }

  memcpy(config->remote_endpoint, endpoint, len + 1);
  return true;
}

bool sync_engine_should_take_remote(SyncEngineVersion local_version,
                                    SyncEngineVersion remote_version) {
  if (remote_version.updated_at > local_version.updated_at) {
    return true;
  }

  if (remote_version.updated_at < local_version.updated_at) {
    return false;
  }

  return remote_version.device_rank > local_version.device_rank;
}
