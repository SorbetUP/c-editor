CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g -DNDEBUG -DDEBUG_CRYPTO=0
TARGET = libcrypto_engine.a
OBJS = crypto_basic.o

all: $(TARGET)

$(TARGET): $(OBJS)
	ar rcs $(TARGET) $(OBJS)
	@echo "✅ Static library built: $(TARGET)"

crypto_basic.o: crypto_basic.c crypto_engine.h
	$(CC) $(CFLAGS) -c crypto_basic.c -o crypto_basic.o

static: $(TARGET)

test: $(TARGET)
	$(CC) $(CFLAGS) test_crypto.c $(TARGET) -o test_crypto
	./test_crypto

clean:
	rm -f $(OBJS) $(TARGET) test_crypto crypto_basic.o

.PHONY: all static test clean
