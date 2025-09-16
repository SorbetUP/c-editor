CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g -DDEBUG_SEARCH=0 -DNDEBUG
TARGET = libsearch_engine.a
OBJS = search_engine.o

all: $(TARGET)

$(TARGET): $(OBJS)
	ar rcs $(TARGET) $(OBJS)
	@echo "✅ Static library built: $(TARGET)"

search_engine.o: search_engine.c search_engine.h
	$(CC) $(CFLAGS) -c search_engine.c -o search_engine.o

static: $(TARGET)

test: $(TARGET)
	$(CC) $(CFLAGS) test_search.c $(TARGET) -o test_search
	./test_search

clean:
	rm -f $(OBJS) $(TARGET) test_search

.PHONY: all static test clean