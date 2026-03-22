CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g -DNDEBUG
TARGET = liblink_engine.a
OBJS = link_engine.o

all: $(TARGET)

$(TARGET): $(OBJS)
	ar rcs $(TARGET) $(OBJS)
	@echo "✅ Static library built: $(TARGET)"

link_engine.o: link_engine.c link_engine.h
	$(CC) $(CFLAGS) -c link_engine.c -o link_engine.o

static: $(TARGET)

test: $(TARGET)
	$(CC) $(CFLAGS) test_link_engine.c $(TARGET) -o test_link_engine
	./test_link_engine

clean:
	rm -f $(OBJS) $(TARGET) test_link_engine

.PHONY: all static test clean
