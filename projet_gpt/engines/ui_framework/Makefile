# Makefile for UI Framework
CC = clang
CFLAGS = -Wall -Wextra -std=c11 -fPIC
OBJCFLAGS = -Wall -Wextra -fobjc-arc -fPIC

SRCDIR = .
BUILDDIR = build
SOURCES = $(wildcard $(SRCDIR)/*.m)
OBJECTS = $(SOURCES:$(SRCDIR)/%.m=$(BUILDDIR)/%.o)
LIBRARY = $(BUILDDIR)/libui_framework.a

.PHONY: all clean static

all: static

static: $(LIBRARY)

$(LIBRARY): $(OBJECTS) | $(BUILDDIR)
	ar rcs $@ $^
	ranlib $@
	@echo "✅ UI Framework library built: $@"

$(BUILDDIR)/%.o: $(SRCDIR)/%.m | $(BUILDDIR)
	$(CC) $(OBJCFLAGS) -c $< -o $@

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

clean:
	rm -rf $(BUILDDIR)
	@echo "🧹 Cleaned UI Framework build directory"

install: $(LIBRARY)
	@echo "📦 UI Framework library ready for linking"
