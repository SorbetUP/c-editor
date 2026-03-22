# Makefile for Search Interface Library
CC = clang
CFLAGS = -Wall -Wextra -std=c11 -O2 -fPIC
LDFLAGS = -lm

SRCDIR = .
BUILDDIR = build
SOURCES = search_interface.c simple_optimizations.c
OBJECTS = $(SOURCES:%.c=$(BUILDDIR)/%.o)
LIBRARY = $(BUILDDIR)/libsearch_interface.a
TEST_BINARY = $(BUILDDIR)/test_search_interface
INTEGRATION_BINARY = $(BUILDDIR)/search_integration
BENCHMARK_BINARY = $(BUILDDIR)/large_corpus_benchmark
REALISTIC_BINARY = $(BUILDDIR)/realistic_benchmark
OPTIMIZATION_BINARY = $(BUILDDIR)/optimization_benchmark
SIMPLE_OPT_BINARY = $(BUILDDIR)/simple_optimizations

.PHONY: all clean static test integration benchmark realistic optimization simple_opt

all: static test integration benchmark realistic simple_opt

static: $(LIBRARY)

$(LIBRARY): $(OBJECTS) | $(BUILDDIR)
	ar rcs $@ $^
	ranlib $@
	@echo "✅ Search Interface library built: $@"

$(BUILDDIR)/%.o: $(SRCDIR)/%.c | $(BUILDDIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

test: $(TEST_BINARY)

$(TEST_BINARY): test_search_interface.c $(LIBRARY) | $(BUILDDIR)
	$(CC) $(CFLAGS) $< -L$(BUILDDIR) -lsearch_interface $(LDFLAGS) -o $@
	@echo "✅ Test binary built: $@"

integration: $(INTEGRATION_BINARY)

$(INTEGRATION_BINARY): search_integration.c $(LIBRARY) | $(BUILDDIR)
	$(CC) $(CFLAGS) $< -L$(BUILDDIR) -lsearch_interface -L../advanced_search/build -ladvanced_search $(LDFLAGS) -o $@
	@echo "✅ Integration binary built: $@"

benchmark: $(BENCHMARK_BINARY)

$(BENCHMARK_BINARY): large_corpus_benchmark.c $(LIBRARY) | $(BUILDDIR)
	$(CC) $(CFLAGS) $< -L$(BUILDDIR) -lsearch_interface -L../advanced_search/build -ladvanced_search $(LDFLAGS) -o $@
	@echo "✅ Benchmark binary built: $@"

realistic: $(REALISTIC_BINARY)

$(REALISTIC_BINARY): realistic_benchmark.c $(LIBRARY) | $(BUILDDIR)
	$(CC) $(CFLAGS) $< -L$(BUILDDIR) -lsearch_interface -L../advanced_search/build -ladvanced_search $(LDFLAGS) -o $@
	@echo "✅ Realistic benchmark binary built: $@"

optimization: $(OPTIMIZATION_BINARY)

$(OPTIMIZATION_BINARY): optimization_benchmark.c $(LIBRARY) | $(BUILDDIR)
	$(CC) $(CFLAGS) $< -L$(BUILDDIR) -lsearch_interface -L../advanced_search/build -ladvanced_search $(LDFLAGS) -lpthread -o $@
	@echo "✅ Optimization benchmark binary built: $@"

simple_opt: $(SIMPLE_OPT_BINARY)

$(SIMPLE_OPT_BINARY): simple_optimizations.c $(LIBRARY) | $(BUILDDIR)
	$(CC) $(CFLAGS) $< -L$(BUILDDIR) -lsearch_interface -L../advanced_search/build -ladvanced_search $(LDFLAGS) -o $@
	@echo "✅ Simple optimizations binary built: $@"

clean:
	rm -rf $(BUILDDIR)
	@echo "🧹 Cleaned Search Interface build directory"

install: $(LIBRARY)
	@echo "📦 Search Interface library ready for integration"

# Create test data
testdata:
	@echo "📝 Creating test directory structure..."
	mkdir -p test_vault/Notes/Projects
	mkdir -p test_vault/Notes/Archive
	mkdir -p test_vault/Templates
	echo "# Project Alpha" > test_vault/Notes/Projects/alpha.md
	echo "This is the Alpha project documentation." >> test_vault/Notes/Projects/alpha.md
	echo "# Project Beta" > test_vault/Notes/Projects/beta.md
	echo "This is the Beta project documentation." >> test_vault/Notes/Projects/beta.md
	echo "# Old Note" > test_vault/Notes/Archive/old_note.md
	echo "This is an archived note." >> test_vault/Notes/Archive/old_note.md
	echo "# Daily Notes Template" > test_vault/Templates/daily.md
	echo "Template for daily notes." >> test_vault/Templates/daily.md
	echo "# Meeting Template" > test_vault/Templates/meeting.md
	echo "Template for meeting notes." >> test_vault/Templates/meeting.md
	@echo "✅ Test directory structure created in test_vault/"