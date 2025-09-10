#include "../src/markdown.h"
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

static void timeout_handler(int sig) {
  (void)sig;
  printf("TIMEOUT: Infinite loop detected in parse_inline_styles\n");
  exit(1);
}

int main(void) {
  signal(SIGALRM, timeout_handler);
  alarm(2); // 2 second timeout

  printf("Testing parse_inline_styles with simple text...\n");

  const char *test_cases[] = {"Hello World",
                              "Simple text with no markers",
                              "Text with single *",
                              "Text with *unclosed italic",
                              "Text with **unclosed bold",
                              "Text with ==unclosed highlight",
                              "Text with ++unclosed underline"};

  for (size_t i = 0; i < sizeof(test_cases) / sizeof(test_cases[0]); i++) {
    printf("Testing: '%s' ... ", test_cases[i]);
    fflush(stdout);

    InlineSpan spans[32];
    int span_count = parse_inline_styles(test_cases[i], spans, 32);

    printf("OK (spans: %d)\n", span_count);
  }

  printf("All tests passed - no infinite loop detected\n");
  return 0;
}
