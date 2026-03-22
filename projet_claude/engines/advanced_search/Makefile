# Makefile for Advanced Search Library
CC = clang
CFLAGS = -Wall -Wextra -std=c11 -O2 -fPIC
LDFLAGS = -lm

SRCDIR = .
BUILDDIR = build
SOURCES = $(wildcard $(SRCDIR)/*.c)
OBJECTS = $(SOURCES:$(SRCDIR)/%.c=$(BUILDDIR)/%.o)
LIBRARY = $(BUILDDIR)/libadvanced_search.a
TEST_BINARY = $(BUILDDIR)/test_search

.PHONY: all clean static test

all: static test

static: $(LIBRARY)

$(LIBRARY): $(OBJECTS) | $(BUILDDIR)
	ar rcs $@ $^
	ranlib $@
	@echo "✅ Advanced Search library built: $@"

$(BUILDDIR)/%.o: $(SRCDIR)/%.c | $(BUILDDIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

test: $(TEST_BINARY)

$(TEST_BINARY): test_search.c $(LIBRARY) | $(BUILDDIR)
	$(CC) $(CFLAGS) $< -L$(BUILDDIR) -ladvanced_search $(LDFLAGS) -o $@
	@echo "✅ Test binary built: $@"

clean:
	rm -rf $(BUILDDIR)
	@echo "🧹 Cleaned Advanced Search build directory"

install: $(LIBRARY)
	@echo "📦 Advanced Search library ready for integration"

# Run performance tests
benchmark: $(TEST_BINARY)
	@echo "🚀 Running Advanced Search benchmarks..."
	./$(TEST_BINARY) --benchmark

# Create test data
testdata:
	@echo "📝 Creating test data..."
	mkdir -p test_vault/Notes
	echo "# Test Document 1" > test_vault/Notes/document1.md
	echo "This is a test document about artificial intelligence and machine learning." >> test_vault/Notes/document1.md
	echo "It contains various topics like neural networks, deep learning, and natural language processing." >> test_vault/Notes/document1.md
	echo "" >> test_vault/Notes/document1.md
	echo "# Test Document 2" > test_vault/Notes/document2.md  
	echo "This document discusses software engineering practices and methodologies." >> test_vault/Notes/document2.md
	echo "Topics include agile development, continuous integration, and code quality." >> test_vault/Notes/document2.md
	echo "" >> test_vault/Notes/document2.md
	echo "# Research Notes" > test_vault/Notes/research.md
	echo "Research notes on embedding models and vector similarity search." >> test_vault/Notes/research.md
	echo "Covers FAISS, ScaNN, and other similarity search algorithms." >> test_vault/Notes/research.md
	echo "" >> test_vault/Notes/research.md
	echo "# Meeting Notes" > test_vault/Notes/meeting_20250917.md
	echo "Meeting discussion about search functionality implementation." >> test_vault/Notes/meeting_20250917.md
	echo "Key points: performance optimization, user experience, embedding techniques." >> test_vault/Notes/meeting_20250917.md
	@echo "✅ Test data created in test_vault/Notes/"