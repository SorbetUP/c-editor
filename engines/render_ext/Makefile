CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g -DNDEBUG
TARGET = librender_ext.a
OBJS = render_ext.o

all: $(TARGET)

$(TARGET): $(OBJS)
	ar rcs $(TARGET) $(OBJS)
	@echo "✅ Static library built: $(TARGET)"

render_ext.o: render_ext.c render_ext.h
	$(CC) $(CFLAGS) -c render_ext.c -o render_ext.o

static: $(TARGET)

test: $(TARGET)
	$(CC) $(CFLAGS) test_render_ext.c $(TARGET) -o test_render_ext
	./test_render_ext

clean:
	rm -f $(OBJS) $(TARGET) test_render_ext

.PHONY: all static test clean
