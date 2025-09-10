#!/bin/bash
set -e

# Simulate GitHub Pages build process
echo "=== Testing GitHub Pages Build Locally ==="

echo "🌐 Step 1: Building WASM..."
if command -v emcc >/dev/null 2>&1; then
  cd wasm
  ./build.sh
  cd ..
  echo "✅ WASM build successful"
else
  echo "⚠️ Emscripten not available, using existing WASM files"
  if [ ! -f "flutter/web/editor.wasm" ] || [ ! -f "flutter/web/editor_wasm.js" ]; then
    echo "❌ Required WASM files missing and Emscripten not available"
    exit 1
  fi
fi

echo "📱 Step 2: Building Flutter web..."
cd flutter
flutter pub get
flutter build web --base-href '/c-editor/'

echo "📋 Step 3: Checking build output..."
if [ ! -d "build/web" ]; then
  echo "❌ Flutter web build failed - no build/web directory"
  exit 1
fi

echo "Build contents:"
ls -la build/web/

echo "📊 Step 4: Checking critical files..."
CRITICAL_FILES=(
  "build/web/index.html"
  "build/web/main.dart.js"
  "build/web/assets/NOTICES"
  "build/web/assets/AssetManifest.json"
)

for file in "${CRITICAL_FILES[@]}"; do
  if [ -f "$file" ]; then
    echo "✅ $file exists"
  else
    echo "❌ $file missing"
    exit 1
  fi
done

echo "🔍 Step 5: Checking WASM assets in build..."
if [ -f "build/web/assets/web/editor.wasm" ]; then
  echo "✅ WASM file included in build"
else
  echo "⚠️ WASM file not found in build assets"
fi

echo "📏 Step 6: Build size analysis..."
BUILD_SIZE=$(du -sh build/web | cut -f1)
echo "Total build size: $BUILD_SIZE"

INDEX_SIZE=$(ls -lh build/web/index.html | awk '{print $5}')
echo "Index.html size: $INDEX_SIZE"

if [ -f "build/web/main.dart.js" ]; then
  JS_SIZE=$(ls -lh build/web/main.dart.js | awk '{print $5}')
  echo "Main JS size: $JS_SIZE"
fi

echo ""
echo "🎉 GitHub Pages build test completed successfully!"
echo "✅ Ready for deployment to https://your-username.github.io/c-editor/"