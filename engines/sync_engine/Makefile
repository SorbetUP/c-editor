CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g -DNDEBUG
TARGET = libsync_engine.a
OBJS = sync_engine.o

all: $(TARGET)

$(TARGET): $(OBJS)
	ar rcs $(TARGET) $(OBJS)
	@echo "✅ Static library built: $(TARGET)"

sync_engine.o: sync_engine.c sync_engine.h
	$(CC) $(CFLAGS) -c sync_engine.c -o sync_engine.o

static: $(TARGET)

test: $(TARGET)
	$(CC) $(CFLAGS) test_sync_engine.c $(TARGET) -o test_sync_engine
	./test_sync_engine

clean:
	rm -f $(OBJS) $(TARGET) test_sync_engine

.PHONY: all static test clean
