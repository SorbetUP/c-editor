#ifndef SYNC_ENGINE_H
#define SYNC_ENGINE_H

#include <stdbool.h>
#include <stdint.h>

typedef enum {
  SYNC_ENGINE_MODE_LOCAL = 0,
  SYNC_ENGINE_MODE_WEB = 1,
  SYNC_ENGINE_MODE_SYNC = 2
} SyncEngineMode;

typedef struct {
  SyncEngineMode mode;
  char remote_endpoint[512];
  bool allow_lan_http;
} SyncEngineConfig;

typedef struct {
  uint64_t updated_at;
  uint64_t device_rank;
} SyncEngineVersion;

void sync_engine_init(SyncEngineConfig *config);
bool sync_engine_set_remote(SyncEngineConfig *config, const char *endpoint);
bool sync_engine_should_take_remote(SyncEngineVersion local_version,
                                    SyncEngineVersion remote_version);

#endif
