CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g -DNDEBUG
TARGET = libprivacy_engine.a
OBJS = privacy_engine.o

all: $(TARGET)

$(TARGET): $(OBJS)
	ar rcs $(TARGET) $(OBJS)
	@echo "✅ Static library built: $(TARGET)"

privacy_engine.o: privacy_engine.c privacy_engine.h
	$(CC) $(CFLAGS) -c privacy_engine.c -o privacy_engine.o

static: $(TARGET)

test: $(TARGET)
	$(CC) $(CFLAGS) test_privacy_engine.c $(TARGET) -o test_privacy_engine
	./test_privacy_engine

clean:
	rm -f $(OBJS) $(TARGET) test_privacy_engine

.PHONY: all static test clean
