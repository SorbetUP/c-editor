#!/bin/bash
set -e

echo "=== Testing Page Functionality ==="

# Start server in background
cd flutter/build/web
python3 -m http.server 8001 --bind 127.0.0.1 >/dev/null 2>&1 &
SERVER_PID=$!
cd ../../..

echo "🌐 Started local server (PID: $SERVER_PID)"

# Wait for server to start
sleep 2

echo "📋 Testing HTTP responses..."

# Test main page
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8001/)
if [ "$RESPONSE" = "200" ]; then
  echo "✅ Main page: 200 OK"
else
  echo "❌ Main page failed: $RESPONSE"
  kill $SERVER_PID
  exit 1
fi

# Test critical assets
ASSETS=("main.dart.js" "editor.wasm" "editor_wasm.js" "flutter.js" "manifest.json")

for asset in "${ASSETS[@]}"; do
  RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:8001/$asset")
  if [ "$RESPONSE" = "200" ]; then
    echo "✅ $asset: 200 OK"
  else
    echo "❌ $asset failed: $RESPONSE"
    kill $SERVER_PID
    exit 1
  fi
done

echo "📊 Testing asset sizes..."

# Check that main assets have reasonable sizes
MAIN_JS_SIZE=$(curl -s http://127.0.0.1:8001/main.dart.js | wc -c)
WASM_SIZE=$(curl -s http://127.0.0.1:8001/editor.wasm | wc -c)

echo "Main JS size: $MAIN_JS_SIZE bytes"
echo "WASM size: $WASM_SIZE bytes"

if [ $MAIN_JS_SIZE -lt 100000 ]; then
  echo "❌ Main JS too small: $MAIN_JS_SIZE bytes"
  kill $SERVER_PID
  exit 1
fi

if [ $WASM_SIZE -lt 10000 ]; then
  echo "❌ WASM too small: $WASM_SIZE bytes"  
  kill $SERVER_PID
  exit 1
fi

echo "✅ Asset sizes look good"

# Check HTML content structure
HTML_CONTENT=$(curl -s http://127.0.0.1:8001/)
if echo "$HTML_CONTENT" | grep -q "flutter"; then
  echo "✅ HTML contains Flutter content"
else
  echo "❌ HTML missing Flutter content"
  kill $SERVER_PID
  exit 1
fi

if echo "$HTML_CONTENT" | grep -q "c-editor"; then
  echo "✅ HTML contains c-editor base href"
else
  echo "❌ HTML missing base href"
  kill $SERVER_PID
  exit 1
fi

# Test WASM assets in Flutter assets
FLUTTER_WASM_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:8001/assets/web/editor.wasm")
if [ "$FLUTTER_WASM_RESPONSE" = "200" ]; then
  echo "✅ Flutter assets WASM: 200 OK"
else
  echo "⚠️ Flutter assets WASM: $FLUTTER_WASM_RESPONSE (may be normal)"
fi

# Cleanup
kill $SERVER_PID
echo "🧹 Stopped server"

echo ""
echo "🎉 Page functionality test completed successfully!"
echo "✅ The page should work properly on GitHub Pages"
echo ""
echo "🌐 Local test URL was: http://127.0.0.1:8001/"
echo "🚀 GitHub Pages URL will be: https://your-username.github.io/c-editor/"