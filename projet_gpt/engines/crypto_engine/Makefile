CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g -DNDEBUG -DDEBUG_CRYPTO=0
TEST_CFLAGS = $(filter-out -DNDEBUG,$(CFLAGS))
TARGET = libcrypto_engine.a
OBJS = crypto_engine.o

all: $(TARGET)

$(TARGET): $(OBJS)
	ar rcs $(TARGET) $(OBJS)
	@echo "✅ Static library built: $(TARGET)"

crypto_engine.o: crypto_engine.c crypto_engine.h
	$(CC) $(CFLAGS) -c crypto_engine.c -o crypto_engine.o

static: $(TARGET)

test: $(TARGET)
	$(CC) $(TEST_CFLAGS) test_crypto.c $(TARGET) -o test_crypto
	./test_crypto

clean:
	rm -f $(OBJS) $(TARGET) test_crypto

.PHONY: all static test clean
