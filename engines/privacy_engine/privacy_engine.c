#include "privacy_engine.h"

#include <string.h>

static const char *k_placeholder = "# (credentials)";

bool privacy_engine_is_sensitive(const char *markdown) {
  return markdown && strstr(markdown, "#credentials") != NULL;
}

bool privacy_engine_mask_preview(const char *markdown,
                                 char *output,
                                 size_t output_size) {
  const char *line_end;
  size_t len;

  if (!output || output_size == 0) {
    return false;
  }

  if (!privacy_engine_is_sensitive(markdown)) {
    output[0] = '\0';
    return false;
  }

  if (!markdown) {
    return false;
  }

  if (markdown[0] == '#' && markdown[1] == ' ') {
    line_end = strchr(markdown, '\n');
    len = line_end ? (size_t)(line_end - markdown) : strlen(markdown);
    if (len >= output_size) {
      len = output_size - 1;
    }
    memcpy(output, markdown, len);
    output[len] = '\0';
    return true;
  }

  len = strlen(k_placeholder);
  if (len >= output_size) {
    len = output_size - 1;
  }
  memcpy(output, k_placeholder, len);
  output[len] = '\0';
  return true;
}
