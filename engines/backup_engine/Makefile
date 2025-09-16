CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g -DNDEBUG
TARGET = libbackup_engine.a

all: $(TARGET)

$(TARGET):
	@echo "✅ Stub library built: $(TARGET)"
	touch $(TARGET)

static: $(TARGET)

clean:
	rm -f $(TARGET)

.PHONY: all static clean
