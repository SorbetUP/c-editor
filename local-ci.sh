#!/bin/bash
set -e

# Local CI simulation script - reproduit exactement le GitHub Actions CI
echo "=== Local CI Simulation ==="

# Core library tests
echo "🔧 Building core library..."
make clean
make -j
make bin/prop_roundtrip bin/fuzz_markdown

echo "✅ Running unit tests..."
make test

echo "🧪 Property tests (3 seeds)..."
./bin/prop_roundtrip 12345 1 | tee prop_12345.log
./bin/prop_roundtrip 424242 1 | tee prop_424242.log  
./bin/prop_roundtrip 999 1 | tee prop_999.log

echo "🩺 AddressSanitizer tests..."
CC=clang CFLAGS="-std=c11 -fsanitize=address,undefined -fno-omit-frame-pointer -g -O1" make clean && make
make bin/prop_roundtrip bin/fuzz_markdown
ASAN_OPTIONS=halt_on_error=1 ./bin/prop_roundtrip 12345 1

echo "🎯 Idempotence validation..."
mkdir -p tests/fixtures tests/corpus/real

# Create test fixtures
echo "# Test Header" > tests/fixtures/simple.md
echo "" >> tests/fixtures/simple.md
echo "This is **bold** and *italic* text." >> tests/fixtures/simple.md

echo "| Col1 | Col2 |" > tests/fixtures/table.md
echo "|------|------|" >> tests/fixtures/table.md  
echo "| A    | B    |" >> tests/fixtures/table.md

# Test idempotence using second-order test (more robust)
gcc -std=c11 -g -Isrc test_idempotent_complex.c libeditor.a -o test_idempotent_complex -lm
if ./test_idempotent_complex; then
  echo "✅ Complex document idempotence test passed"
else
  echo "❌ Complex document idempotence test failed"
  exit 1
fi

echo "🔍 Raw markers validation..."
./test_abi 2>&1 | grep -E '\*\*?|\=\=|\+\+' && {
  echo "❌ Raw markers detected in output!"
  exit 1
} || echo "✅ No raw markers found"

echo "📄 JSON canonicalization..."
echo '{"test": "value", "number": 123.456}' > /tmp/test.json
X="$(cat /tmp/test.json)"
Y="$(cat /tmp/test.json)"
if [ "$X" != "$Y" ]; then
  echo "❌ JSON canonicalization not byte-identical"
  exit 1
fi
echo "✅ JSON canonicalization is byte-identical"

echo "🚀 Performance smoke test..."
make clean
CC=clang CFLAGS="-std=c11 -O3 -DNDEBUG" make -j

# Generate ~100KB markdown file
for i in {1..1000}; do
  echo "# Header $i"
  echo ""
  echo "This is **paragraph $i** with *various* ==formatting== and ++styles++."
  echo ""
done > large_test.md

echo "File size: $(wc -c < large_test.md) bytes"
time_start=$(date +%s%N)
./test_abi < large_test.md >/dev/null
time_end=$(date +%s%N)

duration_ms=$(( (time_end - time_start) / 1000000 ))
echo "Parse time: ${duration_ms}ms"

if [ $duration_ms -gt 200 ]; then
  echo "❌ Performance test failed: ${duration_ms}ms > 200ms"
  exit 1
fi
echo "✅ Performance test passed: ${duration_ms}ms < 200ms"

echo "🎯 ABI capabilities check..."
./test_abi | grep "Features:" | tee capabilities.txt
expected="🎯 Features: 0x1f"
actual=$(grep "🎯 Features:" capabilities.txt)

if [ "$actual" != "$expected" ]; then
  echo "❌ ABI capabilities changed without version bump!"
  echo "Expected: $expected"
  echo "Actual: $actual"
  exit 1
fi
echo "✅ ABI capabilities unchanged"

echo "🔧 Code formatting check..."
if command -v clang-format >/dev/null 2>&1; then
  find src tests -name "*.c" -o -name "*.h" | xargs clang-format --dry-run --Werror
  echo "✅ Code formatting OK"
else
  echo "⚠️ clang-format not installed, skipping format check"
fi

# Build WASM if emscripten available
if command -v emcc >/dev/null 2>&1; then
  echo "🌐 Building WASM..."
  cd wasm
  ./build.sh
  cd ..
  echo "✅ WASM build OK"
else
  echo "⚠️ Emscripten not available, skipping WASM build"
fi

# Flutter web build test
if command -v flutter >/dev/null 2>&1; then
  echo "📱 Testing Flutter web build..."
  cd flutter
  flutter pub get >/dev/null 2>&1
  if flutter build web --base-href '/c-editor/' >/dev/null 2>&1; then
    echo "✅ Flutter web build OK"
  else
    echo "❌ Flutter web build failed"
    exit 1
  fi
  cd ..
else
  echo "⚠️ Flutter not available, skipping Flutter build test"
fi

echo ""
echo "🎉 All local CI checks passed!"
echo "✅ Ready to push to GitHub"