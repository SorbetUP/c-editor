#ifndef PRIVACY_ENGINE_H
#define PRIVACY_ENGINE_H

#include <stdbool.h>
#include <stddef.h>

bool privacy_engine_is_sensitive(const char *markdown);
bool privacy_engine_mask_preview(const char *markdown,
                                 char *output,
                                 size_t output_size);

#endif
