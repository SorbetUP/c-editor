CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g -DNDEBUG
TARGET = libvault_core.a
OBJS = vault_core.o

all: $(TARGET)

$(TARGET): $(OBJS)
	ar rcs $(TARGET) $(OBJS)
	@echo "✅ Static library built: $(TARGET)"

vault_core.o: vault_core.c vault_core.h
	$(CC) $(CFLAGS) -c vault_core.c -o vault_core.o

static: $(TARGET)

test: $(TARGET)
	$(CC) $(CFLAGS) test_vault_core.c $(TARGET) -o test_vault_core
	./test_vault_core

clean:
	rm -f $(OBJS) $(TARGET) test_vault_core

.PHONY: all static test clean
