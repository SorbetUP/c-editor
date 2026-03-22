#include "markdown.h"
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char *strdup_safe(const char *s) {
  if (!s)
    return NULL;
  size_t len = strlen(s);
  char *copy = malloc(len + 1);
  memcpy(copy, s, len + 1);
  return copy;
}

static void trim_whitespace(char *str) {
  char *end = str + strlen(str) - 1;
  while (end > str && isspace(*end)) {
    *end = '\0';
    end--;
  }

  char *start = str;
  while (*start && isspace(*start)) {
    start++;
  }

  if (start != str) {
    memmove(str, start, strlen(start) + 1);
  }
}

// Removed uppercase conversion for idempotence

static bool is_note_link_path(const char *href) {
  if (!href)
    return false;
  if (strncmp(href, "/Users/", 7) == 0)
    return true;
  return false;
}

static void ensure_document_capacity(Document *doc, size_t extra) {
  if (!doc)
    return;
  size_t needed = doc->elements_len + extra;
  if (doc->elements_capacity >= needed)
    return;
  size_t new_cap = doc->elements_capacity == 0 ? 8 : doc->elements_capacity;
  while (new_cap < needed)
    new_cap *= 2;
  doc->elements = realloc(doc->elements, new_cap * sizeof(Element));
  doc->elements_capacity = new_cap;
}

int parse_inline_styles(const char *text, InlineSpan *spans, size_t max_spans) {
  if (!text || !spans || max_spans == 0)
    return 0;

  size_t len = strlen(text);
  if (len == 0)
    return 0;

  size_t span_count = 0;
  size_t i = 0;

  while (i < len && span_count < max_spans) {
    bool matched = false;

    // Parse *** (bold+italic) with higher priority
    if (i + 2 < len && text[i] == '*' && text[i + 1] == '*' &&
        text[i + 2] == '*') {
      size_t start = i;
      for (size_t j = i + 3; j + 2 < len; j++) {
        if (text[j] == '*' && text[j + 1] == '*' && text[j + 2] == '*') {
          spans[span_count].style = INLINE_BOLD_ITALIC;
          spans[span_count].start = start;
          spans[span_count].end = j + 3;
          span_count++;
          i = j + 3;
          matched = true;
          break;
        }
      }
    } 
    // Parse ** (bold)
    else if (i + 1 < len && text[i] == '*' && text[i + 1] == '*') {
      size_t start = i;
      for (size_t j = i + 2; j + 1 < len; j++) {
        if (text[j] == '*' && text[j + 1] == '*') {
          spans[span_count].style = INLINE_BOLD;
          spans[span_count].start = start;
          spans[span_count].end = j + 2;
          span_count++;
          i = j + 2;
          matched = true;
          break;
        }
      }
    } 
    // Parse * (italic)
    else if (text[i] == '*') {
      size_t start = i;
      for (size_t j = i + 1; j < len; j++) {
        if (text[j] == '*') {
          spans[span_count].style = INLINE_ITALIC;
          spans[span_count].start = start;
          spans[span_count].end = j + 1;
          span_count++;
          i = j + 1;
          matched = true;
          break;
        }
      }
    } 
    // Parse == (highlight)
    else if (i + 1 < len && text[i] == '=' && text[i + 1] == '=') {
      size_t start = i;
      for (size_t j = i + 2; j + 1 < len; j++) {
        if (text[j] == '=' && text[j + 1] == '=') {
          spans[span_count].style = INLINE_HIGHLIGHT;
          spans[span_count].start = start;
          spans[span_count].end = j + 2;
          span_count++;
          i = j + 2;
          matched = true;
          break;
        }
      }
    } 
    // Parse ++ (underline)
    else if (i + 1 < len && text[i] == '+' && text[i + 1] == '+') {
      size_t start = i;
      for (size_t j = i + 2; j + 1 < len; j++) {
        if (text[j] == '+' && text[j + 1] == '+') {
          spans[span_count].style = INLINE_UNDERLINE;
          spans[span_count].start = start;
          spans[span_count].end = j + 2;
          span_count++;
          i = j + 2;
          matched = true;
          break;
        }
      }
    }
    // Parse ~~ (strikethrough)
    else if (i + 1 < len && text[i] == '~' && text[i + 1] == '~') {
      size_t start = i;
      for (size_t j = i + 2; j + 1 < len; j++) {
        if (text[j] == '~' && text[j + 1] == '~') {
          spans[span_count].style = INLINE_STRIKETHROUGH;
          spans[span_count].start = start;
          spans[span_count].end = j + 2;
          span_count++;
          i = j + 2;
          matched = true;
          break;
        }
      }
    }
    // Parse `code`
    else if (text[i] == '`') {
      size_t start = i;
      for (size_t j = i + 1; j < len; j++) {
        if (text[j] == '`') {
          spans[span_count].style = INLINE_CODE;
          spans[span_count].start = start;
          spans[span_count].end = j + 1;
          span_count++;
          i = j + 1;
          matched = true;
          break;
        }
      }
    }
    // Parse images ![alt](src)
    else if (i + 1 < len && text[i] == '!' && text[i + 1] == '[') {
      size_t start = i;
      size_t bracket_end = 0;
      size_t paren_end = 0;
      for (size_t j = i + 2; j < len; j++) {
        if (text[j] == ']') {
          bracket_end = j;
          break;
        }
      }
      if (bracket_end > 0 && bracket_end + 1 < len && text[bracket_end + 1] == '(') {
        for (size_t j = bracket_end + 2; j < len; j++) {
          if (text[j] == ')') {
            paren_end = j;
            break;
          }
        }
        if (paren_end > 0) {
          spans[span_count].style = INLINE_IMAGE_REF;
          spans[span_count].start = start;
          spans[span_count].end = paren_end + 1;
          span_count++;
          i = paren_end + 1;
          matched = true;
        }
      }
    }
    // Parse links [text](url)
    else if (text[i] == '[') {
      size_t start = i;
      size_t bracket_end = 0;
      size_t paren_end = 0;
      for (size_t j = i + 1; j < len; j++) {
        if (text[j] == ']') {
          bracket_end = j;
          break;
        }
      }
      if (bracket_end > 0 && bracket_end + 1 < len && text[bracket_end + 1] == '(') {
        for (size_t j = bracket_end + 2; j < len; j++) {
          if (text[j] == ')') {
            paren_end = j;
            break;
          }
        }
        if (paren_end > 0) {
          spans[span_count].style = INLINE_LINK;
          spans[span_count].start = start;
          spans[span_count].end = paren_end + 1;
          span_count++;
          i = paren_end + 1;
          matched = true;
        }
      }
    }

    if (!matched) {
      i++;
    }
  }

  return (int)span_count;
}

static char *strip_all_markers(const char *text) {
  size_t len = strlen(text);
  char *result = malloc(len + 1);
  size_t write_pos = 0;

  for (size_t i = 0; i < len;) {
    bool skipped = false;

    // Skip *** markers (bold+italic)
    if (i + 2 < len && text[i] == '*' && text[i + 1] == '*' &&
        text[i + 2] == '*') {
      i += 3;
      skipped = true;
    }
    // Skip ** markers (bold)
    else if (i + 1 < len && text[i] == '*' && text[i + 1] == '*') {
      i += 2;
      skipped = true;
    }
    // Skip * markers (italic)
    else if (text[i] == '*') {
      i++;
      skipped = true;
    }
    // Skip == markers
    else if (i + 1 < len && text[i] == '=' && text[i + 1] == '=') {
      i += 2;
      skipped = true;
    }
    // Skip ++ markers
    else if (i + 1 < len && text[i] == '+' && text[i + 1] == '+') {
      i += 2;
      skipped = true;
    }
    // Skip ~~ markers
    else if (i + 1 < len && text[i] == '~' && text[i + 1] == '~') {
      i += 2;
      skipped = true;
    }
    // Skip ` markers
    else if (text[i] == '`') {
      i += 1;
      skipped = true;
    }

    if (!skipped) {
      result[write_pos++] = text[i];
      i++;
    }
  }

  result[write_pos] = '\0';
  return result;
}

TextSpan *convert_spans_to_text_spans(const char *text, const InlineSpan *spans,
                                      size_t span_count, size_t *out_count) {
  if (span_count == 0) {
    TextSpan *result = malloc(sizeof(TextSpan));
    memset(result, 0, sizeof(TextSpan));
    result[0].text = strip_all_markers(text); // Clean up unmatched markers
    *out_count = 1;
    return result;
  }

  TextSpan *result = malloc((span_count * 2 + 1) * sizeof(TextSpan));
  memset(result, 0, (span_count * 2 + 1) * sizeof(TextSpan));
  size_t result_count = 0;
  size_t text_pos = 0;

  for (size_t i = 0; i < span_count; i++) {
    if (text_pos < spans[i].start) {
      size_t len = spans[i].start - text_pos;
      char *temp_text = malloc(len + 1);
      memcpy(temp_text, text + text_pos, len);
      temp_text[len] = '\0';

      char *plain_text = strip_all_markers(temp_text);
      free(temp_text);

      result[result_count].text = plain_text;
      result_count++;
    }

    if (spans[i].style == INLINE_LINK) {
      const char *link_text_start = text + spans[i].start + 1;
      const char *link_text_end = strchr(link_text_start, ']');
      const char *href_start = link_text_end ? link_text_end + 1 : NULL;
      if (href_start && *href_start == '(')
        href_start++;
      if (href_start) {
        while (href_start < text + spans[i].end && isspace(*href_start))
          href_start++;
      }
      const char *href_end = text + spans[i].end;
      if (href_end > text && *(href_end - 1) == ')')
        href_end--;
      while (href_end > href_start && isspace(*(href_end - 1)))
        href_end--;

      if (link_text_end && href_start && href_end &&
          link_text_end > link_text_start && href_end > href_start) {
        size_t link_len = (size_t)(link_text_end - link_text_start);
        size_t href_len = (size_t)(href_end - href_start);

        char *link_raw = malloc(link_len + 1);
        memcpy(link_raw, link_text_start, link_len);
        link_raw[link_len] = '\0';
        char *link_text = strip_all_markers(link_raw);
        free(link_raw);

        char *href = malloc(href_len + 1);
        memcpy(href, href_start, href_len);
        href[href_len] = '\0';
        trim_whitespace(href);

        result[result_count].text = link_text ? link_text : strdup_safe("");
        result[result_count].is_link = true;
        result[result_count].link_href = href;
        result[result_count].is_note_link = is_note_link_path(href);
        result_count++;
        text_pos = spans[i].end;
        continue;
      }
    } else if (spans[i].style == INLINE_IMAGE_REF) {
      const char *alt_start = text + spans[i].start + 2;
      const char *alt_end = strchr(alt_start, ']');
      const char *src_start = alt_end ? alt_end + 1 : NULL;
      if (src_start && *src_start == '(')
        src_start++;
      if (src_start) {
        while (src_start < text + spans[i].end && isspace(*src_start))
          src_start++;
      }
      const char *src_end = text + spans[i].end;
      if (src_end > text && *(src_end - 1) == ')')
        src_end--;
      while (src_end > src_start && isspace(*(src_end - 1)))
        src_end--;

      if (alt_end && src_start && src_end && alt_end > alt_start &&
          src_end >= src_start) {
        size_t alt_len = (size_t)(alt_end - alt_start);
        size_t src_len = (size_t)(src_end - src_start);

        char *alt_raw = malloc(alt_len + 1);
        memcpy(alt_raw, alt_start, alt_len);
        alt_raw[alt_len] = '\0';
        char *alt_text = strip_all_markers(alt_raw);
        free(alt_raw);

        char *src = malloc(src_len + 1);
        memcpy(src, src_start, src_len);
        src[src_len] = '\0';
        trim_whitespace(src);

        result[result_count].text = alt_text ? alt_text : strdup_safe("");
        result[result_count].is_image = true;
        result[result_count].image_src = src;
        result[result_count].image_alt = strdup_safe(result[result_count].text);
        result[result_count].is_link = false;
        result[result_count].is_note_link = false;
        result_count++;
        text_pos = spans[i].end;
        continue;
      }
    }

    size_t content_start = spans[i].start;
    size_t content_end = spans[i].end;

    switch (spans[i].style) {
    case INLINE_BOLD:
      content_start += 2;
      content_end -= 2;
      break;
    case INLINE_ITALIC:
      content_start += 1;
      content_end -= 1;
      break;
    case INLINE_BOLD_ITALIC:
      content_start += 3;
      content_end -= 3;
      break;
    case INLINE_HIGHLIGHT:
      content_start += 2;
      content_end -= 2;
      break;
    case INLINE_UNDERLINE:
      content_start += 2;
      content_end -= 2;
      break;
    case INLINE_STRIKETHROUGH:
      content_start += 2;
      content_end -= 2;
      break;
    case INLINE_CODE:
      content_start += 1;
      content_end -= 1;
      break;
    default:
      break;
    }

    char *styled_text = NULL;
    if (content_end > content_start) {
      size_t styled_len = content_end - content_start;
      char *temp_styled = malloc(styled_len + 1);
      memcpy(temp_styled, text + content_start, styled_len);
      temp_styled[styled_len] = '\0';
      styled_text = strip_all_markers(temp_styled);
      free(temp_styled);
    } else {
      styled_text = strdup_safe("");
    }

    result[result_count].text = styled_text ? styled_text : strdup_safe("");
    result[result_count].bold =
        (spans[i].style == INLINE_BOLD || spans[i].style == INLINE_BOLD_ITALIC);
    result[result_count].italic = (spans[i].style == INLINE_ITALIC ||
                                   spans[i].style == INLINE_BOLD_ITALIC);
    result[result_count].has_highlight = (spans[i].style == INLINE_HIGHLIGHT);
    if (result[result_count].has_highlight) {
      result[result_count].highlight_color = (RGBA){1.0f, 1.0f, 0.0f, 0.3f};
    }
    result[result_count].has_underline = (spans[i].style == INLINE_UNDERLINE);
    if (result[result_count].has_underline) {
      result[result_count].underline_color = (RGBA){0.0f, 0.0f, 0.0f, 0.4f};
      result[result_count].underline_gap = 7;
    }
    if (spans[i].style == INLINE_CODE) {
      result[result_count].code = true;
    }
    if (spans[i].style == INLINE_STRIKETHROUGH) {
      result[result_count].strikethrough = true;
    }

    result_count++;
    text_pos = spans[i].end;
  }

  size_t text_len = strlen(text);
  if (text_pos < text_len) {
    size_t len = text_len - text_pos;
    char *temp_remaining = malloc(len + 1);
    memcpy(temp_remaining, text + text_pos, len);
    temp_remaining[len] = '\0';

    char *remaining_text = strip_all_markers(temp_remaining);
    free(temp_remaining);

    result[result_count].text = remaining_text;
    result_count++;
  }

  *out_count = result_count;
  return result;
}

static void populate_text_spans(ElementText *text) {
  if (!text)
    return;

  if (!text->text) {
    text->text = strdup_safe("");
  }

  InlineSpan spans_buffer[128];
  int parsed = parse_inline_styles(text->text, spans_buffer, 128);
  size_t span_count = parsed > 0 ? (size_t)parsed : 0;

  size_t new_span_count = 0;
  TextSpan *converted = NULL;
  if (span_count > 0) {
    converted = convert_spans_to_text_spans(text->text, spans_buffer, span_count,
                                            &new_span_count);
  } else {
    converted = convert_spans_to_text_spans(text->text, NULL, 0, &new_span_count);
  }

  bool preserve_bold = text->bold;
  bool preserve_italic = text->italic;
  bool preserve_highlight = text->has_highlight;
  bool preserve_underline = text->has_underline;
  RGBA preserve_highlight_color = text->highlight_color;
  RGBA preserve_underline_color = text->underline_color;
  int preserve_underline_gap = text->underline_gap;

  free(text->text);

  size_t total_len = 0;
  for (size_t i = 0; i < new_span_count; i++) {
    if (converted[i].text)
      total_len += strlen(converted[i].text);
  }

  text->text = malloc(total_len + 1);
  text->text[0] = '\0';
  for (size_t i = 0; i < new_span_count; i++) {
    if (converted[i].text)
      strcat(text->text, converted[i].text);
  }

  bool any_bold = false;
  bool any_italic = false;
  bool any_highlight = false;
  bool any_underline = false;
  RGBA highlight_color = text->highlight_color;
  RGBA underline_color = text->underline_color;
  int underline_gap = text->underline_gap;

  for (size_t i = 0; i < new_span_count; i++) {
    if (converted[i].bold)
      any_bold = true;
    if (converted[i].italic)
      any_italic = true;
    if (converted[i].has_highlight) {
      any_highlight = true;
      highlight_color = converted[i].highlight_color;
    }
    if (converted[i].has_underline) {
      any_underline = true;
      underline_color = converted[i].underline_color;
      underline_gap = converted[i].underline_gap;
    }
  }

  text->bold = preserve_bold || any_bold;
  text->italic = preserve_italic || any_italic;

  if (any_highlight || preserve_highlight) {
    text->has_highlight = true;
    text->highlight_color = any_highlight ? highlight_color : preserve_highlight_color;
  } else {
    text->has_highlight = false;
  }

  if (any_underline || preserve_underline) {
    text->has_underline = true;
    text->underline_color = any_underline ? underline_color : preserve_underline_color;
    text->underline_gap = any_underline ? underline_gap : preserve_underline_gap;
  } else {
    text->has_underline = false;
  }

  text->spans = converted;
  text->spans_count = new_span_count;
}

static ElementText build_text_element(const char *content, bool allow_headers) {
  ElementText text;
  memset(&text, 0, sizeof(text));

  if (!content)
    content = "";

  if (allow_headers && parse_header_line(content, &text) == 0) {
    int header_level = text.level;
    int header_font_size = text.font_size;
    Align header_align = text.align;
    RGBA header_color = text.color;
    bool header_bold = text.bold;

    populate_text_spans(&text);

    text.level = header_level;
    text.font_size = header_font_size;
    text.align = header_align;
    text.color = header_color;
    text.bold = header_bold || text.bold;
    return text;
  }

  text.text = strdup_safe(content);
  text.font = NULL;
  text.align = ALIGN_LEFT;
  text.level = 0;
  text.bold = false;
  text.italic = false;
  text.has_highlight = false;
  text.has_underline = false;
  text.font_size = 16;
  text.color = (RGBA){0.0f, 0.0f, 0.0f, 1.0f};
  text.highlight_color = (RGBA){1.0f, 1.0f, 0.0f, 0.3f};
  text.underline_color = (RGBA){0.0f, 0.0f, 0.0f, 0.4f};
  text.underline_gap = 7;

  populate_text_spans(&text);
  return text;
}

static bool is_horizontal_rule_line(const char *line) {
  if (!line)
    return false;

  const char *p = line;
  char marker = 0;
  int count = 0;

  while (*p) {
    if (*p == ' ' || *p == '\t') {
      p++;
      continue;
    }
    if (*p == '-' || *p == '*' || *p == '_') {
      if (!marker)
        marker = *p;
      else if (*p != marker)
        return false;
      count++;
      p++;
      continue;
    }
    return false;
  }

  return marker != 0 && count >= 3;
}

static bool parse_list_marker(const char *line, bool *ordered, int *indent_level,
                              bool *has_checkbox, bool *checkbox_checked,
                              int *number, const char **content_start) {
  if (!line)
    return false;

  const char *p = line;
  int indent = 0;
  while (*p == ' ' || *p == '\t') {
    indent += (*p == '\t') ? 4 : 1;
    p++;
  }

  if (*p == '\0')
    return false;

  bool is_ordered = false;
  int parsed_number = 0;
  const char *marker_end = NULL;

  if (isdigit(*p)) {
    const char *num_start = p;
    while (isdigit(*p))
      p++;
    if (*p == '.' || *p == ')') {
      marker_end = p + 1;
      is_ordered = true;
      parsed_number = atoi(num_start);
    } else {
      return false;
    }
  } else if (*p == '-' || *p == '*' || *p == '+') {
    marker_end = p + 1;
  } else {
    return false;
  }

  const char *after_marker = marker_end;
  if (!after_marker)
    return false;

  if (*after_marker == ' ' || *after_marker == '\t') {
    while (*after_marker == ' ' || *after_marker == '\t')
      after_marker++;
  } else {
    return false;
  }

  bool checkbox = false;
  bool checkbox_state = false;
  if (after_marker[0] == '[' && after_marker[1] && after_marker[2] == ']') {
    checkbox = true;
    checkbox_state = (after_marker[1] == 'x' || after_marker[1] == 'X');
    after_marker += 3;
    while (*after_marker == ' ' || *after_marker == '\t')
      after_marker++;
  }

  if (ordered)
    *ordered = is_ordered;
  if (indent_level)
    *indent_level = indent / 2;
  if (has_checkbox)
    *has_checkbox = checkbox;
  if (checkbox_checked)
    *checkbox_checked = checkbox_state;
  if (number)
    *number = is_ordered ? (parsed_number > 0 ? parsed_number : 1) : 0;
  if (content_start)
    *content_start = after_marker;

  return true;
}

static bool parse_blockquote_line(const char *line, const char **content_start) {
  if (!line)
    return false;

  const char *p = line;
  while (*p == ' ' || *p == '\t')
    p++;
  if (*p != '>')
    return false;
  p++;
  if (*p == ' ' || *p == '\t')
    p++;
  if (content_start)
    *content_start = p;
  return true;
}

int parse_image_line(const char *line, ElementImage *image) {
  memset(image, 0, sizeof(*image));
  image->alpha = 1.0f;
  image->align = ALIGN_LEFT;

  const char *p = line;
  while (*p && isspace(*p))
    p++;

  if (p[0] != '!' || p[1] != '[') {
    return -1;
  }
  p += 2;

  const char *alt_start = p;
  while (*p && *p != ']')
    p++;
  if (*p != ']')
    return -1;

  size_t alt_len = p - alt_start;
  image->alt = malloc(alt_len + 1);
  memcpy(image->alt, alt_start, alt_len);
  image->alt[alt_len] = '\0';

  p++;
  if (*p != '(') {
    free(image->alt);
    image->alt = NULL;
    return -1;
  }
  p++;

  const char *src_start = p;
  while (*p && *p != ')')
    p++;
  if (*p != ')') {
    free(image->alt);
    image->alt = NULL;
    return -1;
  }

  size_t src_len = p - src_start;
  image->src = malloc(src_len + 1);
  memcpy(image->src, src_start, src_len);
  image->src[src_len] = '\0';

  p++;

  if (*p == '{') {
    p++;
    while (*p && *p != '}') {
      while (*p && isspace(*p))
        p++;

      if (strncmp(p, "w=", 2) == 0) {
        p += 2;
        image->width = atoi(p);
        while (*p && isdigit(*p))
          p++;
      } else if (strncmp(p, "h=", 2) == 0) {
        p += 2;
        image->height = atoi(p);
        while (*p && isdigit(*p))
          p++;
      } else if (strncmp(p, "a=", 2) == 0) {
        p += 2;
        image->alpha = atof(p);
        while (*p && (isdigit(*p) || *p == '.'))
          p++;
      } else if (strncmp(p, "align=", 6) == 0) {
        p += 6;
        if (strncmp(p, "left", 4) == 0) {
          image->align = ALIGN_LEFT;
          p += 4;
        } else if (strncmp(p, "center", 6) == 0) {
          image->align = ALIGN_CENTER;
          p += 6;
        } else if (strncmp(p, "right", 5) == 0) {
          image->align = ALIGN_RIGHT;
          p += 5;
        }
      } else {
        while (*p && !isspace(*p) && *p != '}')
          p++;
      }

      while (*p && isspace(*p))
        p++;
    }
  }

  return 0;
}

int parse_header_line(const char *line, ElementText *text) {
  memset(text, 0, sizeof(*text));

  int level = 0;
  const char *p = line;

  while (*p == '#' && level < 6) {
    level++;
    p++;
  }

  if (level == 0 || (*p != ' ' && *p != '\t')) {
    return -1;
  }

  while (*p && (*p == ' ' || *p == '\t'))
    p++;

  text->text = strdup_safe(p);
  text->level = level;
  text->bold = true;
  text->align = ALIGN_LEFT;
  text->font_size = 28 - (level - 1) * 4;
  text->color = (RGBA){0.0f, 0.0f, 0.0f, 1.0f};

  return 0;
}

bool is_table_separator_line(const char *line) {
  const char *p = line;
  while (*p && isspace(*p))
    p++;

  if (*p == '|')
    p++;

  while (*p) {
    while (*p && isspace(*p))
      p++;

    if (*p == ':')
      p++;

    bool has_dash = false;
    while (*p && *p == '-') {
      has_dash = true;
      p++;
    }

    if (!has_dash)
      return false;

    if (*p == ':')
      p++;

    while (*p && isspace(*p))
      p++;

    if (*p == '|') {
      p++;
    } else if (*p == '\0') {
      break;
    } else {
      return false;
    }
  }

  return true;
}

static int count_table_columns(const char *line) {
  int count = 0;
  const char *p = line;

  while (*p && isspace(*p))
    p++;
  if (*p == '|')
    p++;

  while (*p) {
    while (*p && *p != '|' && *p != '\0')
      p++;
    count++;

    if (*p == '|') {
      p++;
    } else {
      break;
    }
  }

  return count;
}

char **split_table_row(const char *line, int *col_count) {
  int max_cols = count_table_columns(line);
  char **cells = malloc(max_cols * sizeof(char *));

  const char *p = line;
  while (*p && isspace(*p))
    p++;
  if (*p == '|')
    p++;

  int col = 0;
  while (*p && col < max_cols) {
    const char *start = p;
    while (*p && *p != '|')
      p++;

    size_t len = p - start;
    cells[col] = malloc(len + 1);
    memcpy(cells[col], start, len);
    cells[col][len] = '\0';
    trim_whitespace(cells[col]);

    col++;
    if (*p == '|')
      p++;
  }

  *col_count = col;
  return cells;
}

int parse_table_block(MarkdownParser *parser, ElementTable *table) {
  memset(table, 0, sizeof(*table));

  table->grid_color = (RGBA){0.0f, 0.0f, 0.0f, 0.0f};
  table->background_color = (RGBA){1.0f, 1.0f, 1.0f, 1.0f};
  table->grid_size = 1;

  size_t start_pos = parser->pos;
  const char *line_start = parser->text + parser->pos;
  const char *line_end = strchr(line_start, '\n');
  if (!line_end)
    line_end = parser->text + parser->len;

  char *first_line = malloc(line_end - line_start + 1);
  memcpy(first_line, line_start, line_end - line_start);
  first_line[line_end - line_start] = '\0';

  int first_row_cols;
  char **first_row = split_table_row(first_line, &first_row_cols);

  parser->pos = line_end - parser->text;
  if (*line_end == '\n')
    parser->pos++;

  if (parser->pos >= parser->len) {
    for (int i = 0; i < first_row_cols; i++) {
      free(first_row[i]);
    }
    free(first_row);
    free(first_line);
    return -1;
  }

  line_start = parser->text + parser->pos;
  line_end = strchr(line_start, '\n');
  if (!line_end)
    line_end = parser->text + parser->len;

  char *sep_line = malloc(line_end - line_start + 1);
  memcpy(sep_line, line_start, line_end - line_start);
  sep_line[line_end - line_start] = '\0';

  if (!is_table_separator_line(sep_line)) {
    for (int i = 0; i < first_row_cols; i++) {
      free(first_row[i]);
    }
    free(first_row);
    free(first_line);
    free(sep_line);
    parser->pos = start_pos;
    return -1;
  }

  parser->pos = line_end - parser->text;
  if (*line_end == '\n')
    parser->pos++;

  table->cols = first_row_cols;
  table->rows = 1;

  char ***all_rows = malloc(sizeof(char **));
  all_rows[0] = first_row;

  // Track column counts per row
  int *row_col_counts = malloc(sizeof(int));
  row_col_counts[0] = first_row_cols;

  while (parser->pos < parser->len) {
    line_start = parser->text + parser->pos;
    line_end = strchr(line_start, '\n');
    if (!line_end)
      line_end = parser->text + parser->len;

    char *line = malloc(line_end - line_start + 1);
    memcpy(line, line_start, line_end - line_start);
    line[line_end - line_start] = '\0';

    // Check if this line looks like a table row (must contain '|' or start with
    // '|')
    bool is_table_line = false;
    const char *p = line;
    while (*p && isspace(*p))
      p++; // skip whitespace
    if (*p == '|') {
      is_table_line = true;
    } else {
      // Check if line contains any '|' characters
      while (*p && *p != '\n') {
        if (*p == '|') {
          is_table_line = true;
          break;
        }
        p++;
      }
    }

    if (!is_table_line || count_table_columns(line) == 0) {
      free(line);
      break;
    }

    int row_cols;
    char **row = split_table_row(line, &row_cols);

    all_rows = realloc(all_rows, (table->rows + 1) * sizeof(char **));
    all_rows[table->rows] = row;

    row_col_counts = realloc(row_col_counts, (table->rows + 1) * sizeof(int));
    row_col_counts[table->rows] = row_cols;

    table->rows++;

    parser->pos = line_end - parser->text;
    if (*line_end == '\n')
      parser->pos++;

    free(line);
  }

  table->cells = malloc(table->rows * sizeof(ElementText **));
  for (size_t r = 0; r < table->rows; r++) {
    table->cells[r] = malloc(table->cols * sizeof(ElementText *));

    for (size_t c = 0; c < table->cols; c++) {
      table->cells[r][c] = malloc(sizeof(ElementText));
      memset(table->cells[r][c], 0, sizeof(ElementText));

      if (c < (size_t)row_col_counts[r] && all_rows[r][c]) {
        table->cells[r][c]->text = strdup_safe(all_rows[r][c]);
        table->cells[r][c]->level = 0;
        table->cells[r][c]->align = ALIGN_LEFT;
        table->cells[r][c]->color = (RGBA){0.0f, 0.0f, 0.0f, 1.0f};
      } else {
        table->cells[r][c]->text = strdup_safe("");
        table->cells[r][c]->level = 0;
        table->cells[r][c]->align = ALIGN_LEFT;
        table->cells[r][c]->color = (RGBA){0.0f, 0.0f, 0.0f, 1.0f};
      }
    }
  }

  for (size_t r = 0; r < table->rows; r++) {
    for (int c = 0; c < row_col_counts[r]; c++) {
      free(all_rows[r][c]);
    }
    free(all_rows[r]);
  }
  free(all_rows);
  free(row_col_counts);
  free(first_line);
  free(sep_line);

  return 0;
}


int markdown_to_json(const char *markdown, Document *doc) {
  if (!doc) {
    return -1;
  }

  editor_init(doc);
  if (!markdown) {
    return 0;
  }

  const char *cursor = markdown;
  const char *end = markdown + strlen(markdown);

  while (cursor < end) {
    const char *line_end = strchr(cursor, '
');
    if (!line_end) {
      line_end = end;
    }

    size_t raw_len = (size_t)(line_end - cursor);
    char *raw_line = malloc(raw_len + 1);
    if (!raw_line) {
      return -1;
    }
    memcpy(raw_line, cursor, raw_len);
    raw_line[raw_len] = ' ';

    char *line = strdup_safe(raw_line);
    if (!line) {
      free(raw_line);
      return -1;
    }
    trim_whitespace(line);
    const char *trimmed = line;

    const char *next_cursor = (line_end < end) ? line_end + 1 : end;
    bool handled = false;

    if (raw_len == 0 || *trimmed == ' ') {
      ElementText empty = build_text_element("", false);
      ensure_document_capacity(doc, 1);
      doc->elements[doc->elements_len].kind = T_TEXT;
      doc->elements[doc->elements_len].as.text = empty;
      doc->elements_len++;
      handled = true;
    }

    if (!handled && strncmp(trimmed, "```", 3) == 0) {
      const char *language_start = trimmed + 3;
      while (*language_start == ' ' || *language_start == '	') {
        language_start++;
      }
      char *language = strdup_safe(language_start);
      trim_whitespace(language);

      const char *local_cursor = (line_end < end) ? line_end + 1 : end;
      size_t content_len = 0;
      size_t content_cap = 0;
      char *content = NULL;
      bool closed = false;

      while (local_cursor < end) {
        const char *next_line_end = strchr(local_cursor, '
');
        if (!next_line_end) {
          next_line_end = end;
        }
        size_t segment_len = (size_t)(next_line_end - local_cursor);
        char *segment = malloc(segment_len + 1);
        if (!segment) {
          break;
        }
        memcpy(segment, local_cursor, segment_len);
        segment[segment_len] = ' ';

        char *segment_trim = segment;
        while (*segment_trim && isspace(*segment_trim)) {
          segment_trim++;
        }

        if (strncmp(segment_trim, "```", 3) == 0) {
          closed = true;
          free(segment);
          local_cursor = (next_line_end < end) ? next_line_end + 1 : end;
          break;
        }

        size_t needed = segment_len + 1;
        if (content_len + needed + 1 > content_cap) {
          size_t new_cap = content_cap == 0 ? (needed + 64) : content_cap;
          while (content_len + needed + 1 > new_cap) {
            new_cap *= 2;
          }
          char *new_buf = realloc(content, new_cap);
          if (!new_buf) {
            free(segment);
            free(content);
            content = NULL;
            content_cap = 0;
            content_len = 0;
            break;
          }
          content = new_buf;
          content_cap = new_cap;
        }

        memcpy(content + content_len, local_cursor, segment_len);
        content_len += segment_len;
        content[content_len++] = '
';

        free(segment);
        local_cursor = (next_line_end < end) ? next_line_end + 1 : end;
      }

      if (closed) {
        if (!content) {
          content = strdup_safe("");
        } else {
          if (content_len > 0 && content[content_len - 1] == '
') {
            content_len--;
          }
          content[content_len] = ' ';
        }

        ElementCode code;
        memset(&code, 0, sizeof(code));
        code.language = language;
        code.content = content;
        code.fenced = true;

        ensure_document_capacity(doc, 1);
        doc->elements[doc->elements_len].kind = T_CODE;
        doc->elements[doc->elements_len].as.code = code;
        doc->elements_len++;

        handled = true;
        next_cursor = local_cursor;
        if (next_cursor <= cursor) {
          next_cursor = (line_end < end) ? line_end + 1 : end;
        }
      } else {
        free(language);
        free(content);
      }
    }

    if (!handled && is_horizontal_rule_line(trimmed)) {
      ElementDivider divider;
      divider.thickness = 1;
      divider.color = (RGBA){0.7f, 0.7f, 0.7f, 1.0f};

      ensure_document_capacity(doc, 1);
      doc->elements[doc->elements_len].kind = T_DIVIDER;
      doc->elements[doc->elements_len].as.divider = divider;
      doc->elements_len++;
      handled = true;
    }

    bool ordered = false;
    int indent_level = 0;
    bool has_checkbox = false;
    bool checkbox_checked = false;
    int list_number = 0;
    const char *list_content = NULL;

    if (!handled &&
        parse_list_marker(raw_line, &ordered, &indent_level, &has_checkbox,
                          &checkbox_checked, &list_number, &list_content)) {
      ElementList list;
      memset(&list, 0, sizeof(list));
      list.ordered = ordered;
      list.start_index = ordered ? (list_number > 0 ? list_number : 1) : 1;
      list.item_capacity = 4;
      list.items = calloc(list.item_capacity, sizeof(ElementListItem));

      ElementListItem first;
      memset(&first, 0, sizeof(first));
      first.indent_level = 0;
      first.has_checkbox = has_checkbox;
      first.checkbox_checked = checkbox_checked;
      first.number = ordered ? (list_number > 0 ? list_number : list.start_index)
                             : 0;
      first.text = build_text_element(list_content ? list_content : "", false);
      list.items[list.item_count++] = first;

      const char *local_cursor = (line_end < end) ? line_end + 1 : end;
      int base_indent = indent_level;

      while (local_cursor < end) {
        const char *next_line_end = strchr(local_cursor, '
');
        if (!next_line_end) {
          next_line_end = end;
        }
        size_t local_len = (size_t)(next_line_end - local_cursor);
        char *local_raw = malloc(local_len + 1);
        if (!local_raw) {
          break;
        }
        memcpy(local_raw, local_cursor, local_len);
        local_raw[local_len] = ' ';

        bool local_ordered = false;
        int local_indent = 0;
        bool local_has_checkbox = false;
        bool local_checkbox_checked = false;
        int local_number = 0;
        const char *local_content = NULL;

        bool parsed = parse_list_marker(local_raw, &local_ordered, &local_indent,
                                        &local_has_checkbox,
                                        &local_checkbox_checked, &local_number,
                                        &local_content);

        if (!parsed || local_ordered != ordered || local_indent < base_indent) {
          free(local_raw);
          break;
        }

        if (list.item_count >= list.item_capacity) {
          size_t new_cap = list.item_capacity * 2;
          ElementListItem *new_items =
              realloc(list.items, new_cap * sizeof(ElementListItem));
          if (!new_items) {
            free(local_raw);
            break;
          }
          memset(new_items + list.item_capacity, 0,
                 (new_cap - list.item_capacity) * sizeof(ElementListItem));
          list.items = new_items;
          list.item_capacity = new_cap;
        }

        ElementListItem item;
        memset(&item, 0, sizeof(item));
        item.indent_level = local_indent - base_indent;
        item.has_checkbox = local_has_checkbox;
        item.checkbox_checked = local_checkbox_checked;
        item.number = local_ordered
                          ? (local_number > 0 ? local_number
                                               : list.start_index + (int)list.item_count)
                          : 0;
        item.text =
            build_text_element(local_content ? local_content : "", false);

        list.items[list.item_count++] = item;

        free(local_raw);
        local_cursor = (next_line_end < end) ? next_line_end + 1 : end;
      }

      if (list.item_count > 0) {
        ensure_document_capacity(doc, 1);
        doc->elements[doc->elements_len].kind = T_LIST;
        doc->elements[doc->elements_len].as.list = list;
        doc->elements_len++;
        handled = true;
        next_cursor = local_cursor;
        if (next_cursor <= cursor) {
          next_cursor = (line_end < end) ? line_end + 1 : end;
        }
      } else {
        free(list.items);
      }
    }

    if (!handled) {
      const char *quote_content = NULL;
      if (parse_blockquote_line(raw_line, &quote_content)) {
        ElementQuote quote;
        memset(&quote, 0, sizeof(quote));
        quote.item_capacity = 4;
        quote.items = calloc(quote.item_capacity, sizeof(ElementText));
        quote.items[quote.item_count++] =
            build_text_element(quote_content ? quote_content : "", false);

        const char *local_cursor = (line_end < end) ? line_end + 1 : end;

        while (local_cursor < end) {
          const char *next_line_end = strchr(local_cursor, '
');
          if (!next_line_end) {
            next_line_end = end;
          }
          size_t local_len = (size_t)(next_line_end - local_cursor);
          char *local_raw = malloc(local_len + 1);
          if (!local_raw) {
            break;
          }
          memcpy(local_raw, local_cursor, local_len);
          local_raw[local_len] = ' ';

          const char *local_quote_content = NULL;
          if (!parse_blockquote_line(local_raw, &local_quote_content)) {
            free(local_raw);
            break;
          }

          if (quote.item_count >= quote.item_capacity) {
            size_t new_cap = quote.item_capacity * 2;
            ElementText *new_items =
                realloc(quote.items, new_cap * sizeof(ElementText));
            if (!new_items) {
              free(local_raw);
              break;
            }
            quote.items = new_items;
            quote.item_capacity = new_cap;
          }

          quote.items[quote.item_count++] =
              build_text_element(local_quote_content ? local_quote_content : "",
                                 false);

          free(local_raw);
          local_cursor = (next_line_end < end) ? next_line_end + 1 : end;
        }

        if (quote.item_count > 0) {
          ensure_document_capacity(doc, 1);
          doc->elements[doc->elements_len].kind = T_QUOTE;
          doc->elements[doc->elements_len].as.quote = quote;
          doc->elements_len++;
          handled = true;
          next_cursor = local_cursor;
          if (next_cursor <= cursor) {
            next_cursor = (line_end < end) ? line_end + 1 : end;
          }
        } else {
          free(quote.items);
        }
      }
    }

    if (!handled) {
      ElementImage image;
      if (parse_image_line(trimmed, &image) == 0) {
        ensure_document_capacity(doc, 1);
        doc->elements[doc->elements_len].kind = T_IMAGE;
        doc->elements[doc->elements_len].as.image = image;
        doc->elements_len++;
        handled = true;
      }
    }

    if (!handled && strchr(trimmed, '|')) {
      MarkdownParser parser;
      parser.text = cursor;
      parser.pos = 0;
      parser.len = end - cursor;

      ElementTable table;
      if (parse_table_block(&parser, &table) == 0) {
        ensure_document_capacity(doc, 1);
        doc->elements[doc->elements_len].kind = T_TABLE;
        doc->elements[doc->elements_len].as.table = table;
        doc->elements_len++;
        handled = true;
        next_cursor = cursor + parser.pos;
        if (next_cursor <= cursor) {
          next_cursor = (line_end < end) ? line_end + 1 : end;
        }
      }
    }

    if (!handled) {
      ElementText text_elem = build_text_element(trimmed, true);
      ensure_document_capacity(doc, 1);
      doc->elements[doc->elements_len].kind = T_TEXT;
      doc->elements[doc->elements_len].as.text = text_elem;
      doc->elements_len++;
    }

    free(raw_line);
    free(line);
    cursor = next_cursor;
  }

  return 0;
}



static void write_inline_span_text(FILE *fp, const TextSpan *span) {
  if (!span) {
    return;
  }

  const char *content = span->text ? span->text : "";

  if (span->code) {
    fprintf(fp, "`%s`", content);
    return;
  }

  if (span->strikethrough) {
    fprintf(fp, "~~");
  }
  if (span->has_highlight) {
    fprintf(fp, "==");
  }
  if (span->has_underline) {
    fprintf(fp, "++");
  }

  if (span->bold && span->italic) {
    fprintf(fp, "***%s***", content);
  } else if (span->bold) {
    fprintf(fp, "**%s**", content);
  } else if (span->italic) {
    fprintf(fp, "*%s*", content);
  } else {
    fprintf(fp, "%s", content);
  }

  if (span->has_underline) {
    fprintf(fp, "++");
  }
  if (span->has_highlight) {
    fprintf(fp, "==");
  }
  if (span->strikethrough) {
    fprintf(fp, "~~");
  }
}

static void write_inline_markup(FILE *fp, const TextSpan *span) {
  if (!span) {
    return;
  }

  if (span->is_image && span->image_src) {
    const char *alt = span->image_alt && span->image_alt[0]
                          ? span->image_alt
                          : (span->text ? span->text : "");
    fprintf(fp, "![%s](%s)", alt, span->image_src);
    return;
  }

  if (span->is_link && span->link_href) {
    fprintf(fp, "[");
    TextSpan inner = *span;
    inner.is_link = false;
    inner.link_href = NULL;
    inner.is_image = false;
    inner.image_src = NULL;
    inner.image_alt = NULL;
    write_inline_span_text(fp, &inner);
    fprintf(fp, "](%s)", span->link_href);
    return;
  }

  write_inline_span_text(fp, span);
}

static void write_element_text_inline(FILE *fp, const ElementText *text) {
  if (!text) {
    return;
  }

  if (text->spans && text->spans_count > 0) {
    for (size_t i = 0; i < text->spans_count; i++) {
      write_inline_markup(fp, &text->spans[i]);
    }
  } else {
    TextSpan temp;
    memset(&temp, 0, sizeof(temp));
    temp.text = text->text ? text->text : (char *)"";
    temp.bold = text->bold;
    temp.italic = text->italic;
    temp.has_highlight = text->has_highlight;
    temp.highlight_color = text->highlight_color;
    temp.has_underline = text->has_underline;
    temp.underline_color = text->underline_color;
    temp.underline_gap = text->underline_gap;
    write_inline_span_text(fp, &temp);
  }
}

int json_to_markdown(const Document *doc, char **out_markdown) {
  FILE *fp = tmpfile();
  if (!fp)
    return -1;

  for (size_t i = 0; i < doc->elements_len; i++) {
    const Element *elem = &doc->elements[i];

    switch (elem->kind) {
    case T_TEXT: {
      const ElementText *text = &elem->as.text;
      bool has_spans = text->spans && text->spans_count > 0;
      bool has_non_empty_span = false;
      if (has_spans) {
        for (size_t s = 0; s < text->spans_count; s++) {
          const TextSpan *span = &text->spans[s];
          if ((span->text && span->text[0] != '\0') || span->is_link ||
              span->is_image || span->code) {
            has_non_empty_span = true;
            break;
          }
        }
      }

      const char *plain = text->text ? text->text : "";
      bool has_content = has_non_empty_span || (!has_spans && plain[0] != '\0');

      if (!has_content && text->level == 0) {
        fprintf(fp, "\n");
        break;
      }

      if (text->level > 0) {
        for (int j = 0; j < text->level; j++) {
          fputc('#', fp);
        }
        fputc(' ', fp);
      }

      if (has_spans) {
        for (size_t s = 0; s < text->spans_count; s++) {
          write_inline_markup(fp, &text->spans[s]);
        }
      } else {
        TextSpan temp;
        memset(&temp, 0, sizeof(temp));
        temp.text = text->text ? text->text : (char *)"";
        temp.bold = text->bold;
        temp.italic = text->italic;
        temp.has_highlight = text->has_highlight;
        temp.highlight_color = text->highlight_color;
        temp.has_underline = text->has_underline;
        temp.underline_color = text->underline_color;
        temp.underline_gap = text->underline_gap;
        write_inline_span_text(fp, &temp);
      }

      fprintf(fp, "\n");
      break;
    }

    case T_IMAGE: {
      const ElementImage *image = &elem->as.image;
      fprintf(fp, "![%s](%s)", image->alt ? image->alt : "",
              image->src ? image->src : "");

      if (image->width > 0 || image->height > 0 || image->alpha != 1.0f ||
          image->align != ALIGN_LEFT) {
        fprintf(fp, "{");
        bool first = true;

        if (image->width > 0) {
          fprintf(fp, "w=%d", image->width);
          first = false;
        }
        if (image->height > 0) {
          if (!first)
            fprintf(fp, " ");
          fprintf(fp, "h=%d", image->height);
          first = false;
        }
        if (image->alpha != 1.0f) {
          if (!first)
            fprintf(fp, " ");
          fprintf(fp, "a=%.3f", image->alpha);
          first = false;
        }
        if (image->align != ALIGN_LEFT) {
          if (!first)
            fprintf(fp, " ");
          const char *align_str = "left";
          switch (image->align) {
          case ALIGN_CENTER:
            align_str = "center";
            break;
          case ALIGN_RIGHT:
            align_str = "right";
            break;
          default:
            align_str = "left";
            break;
          }
          fprintf(fp, "align=%s", align_str);
        }

        fprintf(fp, "}");
      }

      fprintf(fp, "\n");
      break;
    }

    case T_TABLE: {
      const ElementTable *table = &elem->as.table;

      for (size_t c = 0; c < table->cols; c++) {
        fprintf(fp, "| ");
        if (table->cells[0] && table->cells[0][c]) {
          write_element_text_inline(fp, table->cells[0][c]);
        }
        fprintf(fp, " ");
      }
      fprintf(fp, "|\n");

      for (size_t c = 0; c < table->cols; c++) {
        fprintf(fp, "|---");
      }
      fprintf(fp, "|\n");

      for (size_t r = 1; r < table->rows; r++) {
        for (size_t c = 0; c < table->cols; c++) {
          fprintf(fp, "| ");
          if (table->cells[r] && table->cells[r][c]) {
            write_element_text_inline(fp, table->cells[r][c]);
          }
          fprintf(fp, " ");
        }
        fprintf(fp, "|\n");
      }
      break;
    }
    case T_LIST: {
      const ElementList *list = &elem->as.list;
      int counter = list->start_index > 0 ? list->start_index : 1;
      for (size_t item_index = 0; item_index < list->item_count; item_index++) {
        const ElementListItem *item = &list->items[item_index];
        int indent_spaces = item->indent_level > 0 ? item->indent_level * 2 : 0;
        for (int s = 0; s < indent_spaces; s++) {
          fputc(' ', fp);
        }
        if (list->ordered) {
          int number = item->number > 0 ? item->number : counter;
          fprintf(fp, "%d. ", number);
          counter = number + 1;
        } else {
          fprintf(fp, "- ");
        }
        if (item->has_checkbox) {
          fprintf(fp, "[%c] ", item->checkbox_checked ? 'x' : ' ');
        }
        write_element_text_inline(fp, &item->text);
        fprintf(fp, "\n");
      }
      break;
    }
    case T_QUOTE: {
      const ElementQuote *quote = &elem->as.quote;
      for (size_t q = 0; q < quote->item_count; q++) {
        fprintf(fp, "> ");
        write_element_text_inline(fp, &quote->items[q]);
        fprintf(fp, "\n");
      }
      break;
    }
    case T_DIVIDER: {
      fprintf(fp, "---\n");
      break;
    }
    case T_CODE: {
      const ElementCode *code = &elem->as.code;
      fprintf(fp, "```%s\n", (code->language && code->language[0]) ? code->language : "");
      if (code->content && code->content[0]) {
        fprintf(fp, "%s", code->content);
        if (code->content[strlen(code->content) - 1] != '\n') {
          fprintf(fp, "\n");
        }
      }
      fprintf(fp, "```\n");
      break;
    }

    }
  }

  fseek(fp, 0, SEEK_END);
  long len = ftell(fp);
  fseek(fp, 0, SEEK_SET);

  *out_markdown = malloc(len + 1);
  size_t bytes_read = fread(*out_markdown, 1, len, fp);
  (void)bytes_read; // Suppress unused variable warning
  (*out_markdown)[len] = '\0';

  fclose(fp);
  return 0;
}

// ============= NEW ADVANCED MARKDOWN FUNCTIONS =============

// Parse inline code (`code`)
int parse_code_blocks(const char *text, InlineSpan *spans, size_t max_spans) {
  if (!text || !spans || max_spans == 0) return 0;
  
  size_t len = strlen(text);
  size_t span_count = 0;
  
  for (size_t i = 0; i < len && span_count < max_spans; i++) {
    if (text[i] == '`') {
      size_t start = i;
      
      // Find closing `
      for (size_t j = i + 1; j < len; j++) {
        if (text[j] == '`') {
          spans[span_count].style = INLINE_CODE;
          spans[span_count].start = start;
          spans[span_count].end = j + 1;
          span_count++;
          i = j;
          break;
        }
      }
    }
  }
  
  return span_count;
}

// Parse strikethrough (~~text~~)
int parse_strikethrough(const char *text, InlineSpan *spans, size_t max_spans) {
  if (!text || !spans || max_spans == 0) return 0;
  
  size_t len = strlen(text);
  size_t span_count = 0;
  
  for (size_t i = 0; i + 1 < len && span_count < max_spans; i++) {
    if (text[i] == '~' && text[i + 1] == '~') {
      size_t start = i;
      
      // Find closing ~~
      for (size_t j = i + 2; j + 1 < len; j++) {
        if (text[j] == '~' && text[j + 1] == '~') {
          spans[span_count].style = INLINE_STRIKETHROUGH;
          spans[span_count].start = start;
          spans[span_count].end = j + 2;
          span_count++;
          i = j + 1;
          break;
        }
      }
    }
  }
  
  return span_count;
}

// Parse links [text](url) and images ![alt](url)
int parse_links_and_images(const char *text, InlineSpan *spans, size_t max_spans) {
  if (!text || !spans || max_spans == 0) return 0;
  
  size_t len = strlen(text);
  size_t span_count = 0;
  
  for (size_t i = 0; i < len && span_count < max_spans; i++) {
    // Check for image ![
    if (i + 1 < len && text[i] == '!' && text[i + 1] == '[') {
      size_t start = i;
      size_t bracket_end = 0;
      size_t paren_start = 0;
      size_t paren_end = 0;
      
      // Find closing ]
      for (size_t j = i + 2; j < len; j++) {
        if (text[j] == ']') {
          bracket_end = j;
          break;
        }
      }
      
      // Find opening (
      if (bracket_end > 0 && bracket_end + 1 < len && text[bracket_end + 1] == '(') {
        paren_start = bracket_end + 1;
        
        // Find closing )
        for (size_t j = paren_start + 1; j < len; j++) {
          if (text[j] == ')') {
            paren_end = j;
            break;
          }
        }
        
        if (paren_end > paren_start) {
          spans[span_count].style = INLINE_IMAGE_REF;
          spans[span_count].start = start;
          spans[span_count].end = paren_end + 1;
          span_count++;
          i = paren_end;
        }
      }
    }
    // Check for link [
    else if (text[i] == '[') {
      size_t start = i;
      size_t bracket_end = 0;
      size_t paren_start = 0;
      size_t paren_end = 0;
      
      // Find closing ]
      for (size_t j = i + 1; j < len; j++) {
        if (text[j] == ']') {
          bracket_end = j;
          break;
        }
      }
      
      // Find opening (
      if (bracket_end > 0 && bracket_end + 1 < len && text[bracket_end + 1] == '(') {
        paren_start = bracket_end + 1;
        
        // Find closing )
        for (size_t j = paren_start + 1; j < len; j++) {
          if (text[j] == ')') {
            paren_end = j;
            break;
          }
        }
        
        if (paren_end > paren_start) {
          spans[span_count].style = INLINE_LINK;
          spans[span_count].start = start;
          spans[span_count].end = paren_end + 1;
          span_count++;
          i = paren_end;
        }
      }
    }
  }
  
  return span_count;
}

// Validate URL format
bool is_valid_url(const char *url) {
  if (!url || strlen(url) < 4) return false;
  
  // Check for common protocols
  if (strncmp(url, "http://", 7) == 0 || 
      strncmp(url, "https://", 8) == 0 ||
      strncmp(url, "ftp://", 6) == 0 ||
      strncmp(url, "file://", 7) == 0) {
    return true;
  }
  
  // Check for relative URLs starting with /
  if (url[0] == '/') return true;
  
  // Check for anchor links
  if (url[0] == '#') return true;
  
  return false;
}

// Check if line is a markdown heading
bool is_markdown_heading(const char *line) {
  if (!line) return false;
  
  // Skip leading whitespace
  while (*line == ' ' || *line == '\t') line++;
  
  return (*line == '#');
}

// Get heading level (1-6)
int get_heading_level(const char *line) {
  if (!line) return 0;
  
  // Skip leading whitespace
  while (*line == ' ' || *line == '\t') line++;
  
  int level = 0;
  while (*line == '#' && level < 6) {
    level++;
    line++;
  }
  
  // Must be followed by space or end of line
  if (*line == ' ' || *line == '\t' || *line == '\0' || *line == '\n') {
    return level;
  }
  
  return 0;
}

// Extract heading text (without # symbols)
char* extract_heading_text(const char *line) {
  if (!line) return NULL;
  
  // Skip leading whitespace
  while (*line == ' ' || *line == '\t') line++;
  
  // Skip # symbols
  while (*line == '#') line++;
  
  // Skip space after #
  while (*line == ' ' || *line == '\t') line++;
  
  // Find end of line
  const char *end = line;
  while (*end != '\0' && *end != '\n' && *end != '\r') {
    end++;
  }
  
  // Remove trailing whitespace
  while (end > line && (*(end - 1) == ' ' || *(end - 1) == '\t')) {
    end--;
  }
  
  // Create result string
  size_t len = end - line;
  char *result = malloc(len + 1);
  if (result) {
    strncpy(result, line, len);
    result[len] = '\0';
  }
  
  return result;
}

// Validate markdown structure
bool validate_markdown_structure(const char *markdown) {
  if (!markdown) return false;
  
  // Check for balanced brackets
  int bracket_count = 0;
  int paren_count = 0;
  int code_block_count = 0;
  
  const char *p = markdown;
  while (*p) {
    switch (*p) {
      case '[':
        bracket_count++;
        break;
      case ']':
        bracket_count--;
        if (bracket_count < 0) return false;
        break;
      case '(':
        paren_count++;
        break;
      case ')':
        paren_count--;
        if (paren_count < 0) return false;
        break;
      case '`':
        // Check for triple backticks
        if (p[1] == '`' && p[2] == '`') {
          code_block_count++;
          p += 2; // Skip the other two backticks
        }
        break;
    }
    p++;
  }
  
  // All brackets and parentheses should be balanced
  // Code blocks should be even (paired)
  return (bracket_count == 0 && paren_count == 0 && (code_block_count % 2) == 0);
}

// Enhance markdown formatting
char* enhance_markdown_formatting(const char *markdown) {
  if (!markdown) return NULL;
  
  size_t len = strlen(markdown);
  // Allocate extra space for enhancements
  char *result = malloc(len * 2 + 1000);
  if (!result) return NULL;
  
  const char *src = markdown;
  char *dst = result;
  
  while (*src) {
    // Auto-format headings
    if (*src == '#') {
      // Count heading level
      int level = 0;
      const char *p = src;
      while (*p == '#' && level < 6) {
        level++;
        p++;
      }
      
      // Add proper spacing
      for (int i = 0; i < level; i++) {
        *dst++ = '#';
      }
      
      if (*p != ' ' && *p != '\0' && *p != '\n') {
        *dst++ = ' ';
      }
      
      src = p;
      continue;
    }
    
    // Auto-format list items
    if ((*src == '-' || *src == '*' || *src == '+') && 
        (src == markdown || *(src - 1) == '\n')) {
      *dst++ = *src++;
      if (*src != ' ') {
        *dst++ = ' ';
      }
      continue;
    }
    
    *dst++ = *src++;
  }
  
  *dst = '\0';
  return result;
}

// Auto-format lists
char* auto_format_lists(const char *markdown) {
  if (!markdown) return NULL;
  
  size_t len = strlen(markdown);
  char *result = malloc(len * 2 + 1);
  if (!result) return NULL;
  
  const char *src = markdown;
  char *dst = result;
  bool at_line_start = true;
  
  while (*src) {
    if (at_line_start && (*src == '-' || *src == '*' || *src == '+')) {
      *dst++ = '-'; // Standardize to dash
      *dst++ = ' ';
      src++;
      // Skip existing space if present
      if (*src == ' ') src++;
      at_line_start = false;
      continue;
    }
    
    *dst++ = *src;
    at_line_start = (*src == '\n');
    src++;
  }
  
  *dst = '\0';
  return result;
}

// Fix markdown spacing
char* fix_markdown_spacing(const char *markdown) {
  if (!markdown) return NULL;
  
  size_t len = strlen(markdown);
  char *result = malloc(len * 2 + 1);
  if (!result) return NULL;
  
  const char *src = markdown;
  char *dst = result;
  char prev_char = '\0';
  
  while (*src) {
    char curr_char = *src;
    
    // Add proper spacing around formatting markers
    if ((curr_char == '*' || curr_char == '_' || curr_char == '`') &&
        prev_char != ' ' && prev_char != '\0' && prev_char != '\n' &&
        !isalnum(prev_char)) {
      // Don't add space if it's part of markdown syntax
    }
    
    // Remove multiple consecutive newlines (max 2)
    if (curr_char == '\n' && prev_char == '\n') {
      // Look ahead for more newlines
      const char *p = src + 1;
      while (*p == '\n') p++;
      if (p > src + 1) {
        // Skip extra newlines, keep only one more
        src = p - 1;
      }
    }
    
    *dst++ = *src;
    prev_char = curr_char;
    src++;
  }
  
  *dst = '\0';
  return result;
}
