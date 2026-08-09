#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TAURI_DIR="$ROOT_DIR/Elephant/backend/tauri"
ANDROID_CONFIG="tauri.android.conf.json"
ANDROID_TARGET="${ANDROID_TARGET:-aarch64}"
ANDROID_BUILD_PROFILE="${ANDROID_BUILD_PROFILE:-release}"
ANDROID_APK_MAX_MIB="${ANDROID_APK_MAX_MIB:-24}"
APK_ROOT="$TAURI_DIR/gen/android/app/build/outputs/apk"
ANDROID_GRADLE_FILE="$TAURI_DIR/gen/android/app/build.gradle.kts"
ANDROID_MANIFEST="$TAURI_DIR/gen/android/app/src/main/AndroidManifest.xml"
ANDROID_JNI_LIBS="$TAURI_DIR/gen/android/app/src/main/jniLibs"
ANDROID_RESOURCES="$TAURI_DIR/gen/android/app/src/main/res"
ANDROID_ICON_SOURCE="$ROOT_DIR/Elephant/assets/static/icon.png"
ANDROID_ICON_DEST="$ANDROID_RESOURCES/drawable-nodpi/elephant_launcher.png"
MAIN_ACTIVITY="$TAURI_DIR/gen/android/app/src/main/java/com/elephantnote/app/MainActivity.kt"
MAIN_ACTIVITY_TEMPLATE="$ROOT_DIR/build/android/MainActivity.kt"
RENDERER_OUT="$ROOT_DIR/build/out/renderer"

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "Missing required command: $1" >&2; exit 1; }
}

file_size_bytes() {
  if stat -c '%s' "$1" >/dev/null 2>&1; then stat -c '%s' "$1"; else stat -f '%z' "$1"; fi
}

expected_android_abi() {
  case "$ANDROID_TARGET" in
    aarch64) printf '%s' 'arm64-v8a' ;;
    armv7) printf '%s' 'armeabi-v7a' ;;
    i686) printf '%s' 'x86' ;;
    x86_64) printf '%s' 'x86_64' ;;
    *) echo "Unsupported Android target: $ANDROID_TARGET" >&2; exit 1 ;;
  esac
}

install_generated_android_contracts() {
  test -f "$ANDROID_MANIFEST"
  test -f "$ANDROID_GRADLE_FILE"
  test -f "$MAIN_ACTIVITY_TEMPLATE"
  test -s "$ANDROID_ICON_SOURCE"

  mkdir -p "$(dirname "$MAIN_ACTIVITY")" "$(dirname "$ANDROID_ICON_DEST")"
  cp "$MAIN_ACTIVITY_TEMPLATE" "$MAIN_ACTIVITY"
  cp "$ANDROID_ICON_SOURCE" "$ANDROID_ICON_DEST"

  node - "$ANDROID_MANIFEST" "$ANDROID_GRADLE_FILE" <<'NODE'
const fs = require('node:fs')
const [manifestPath, gradlePath] = process.argv.slice(2)
let manifest = fs.readFileSync(manifestPath, 'utf8')
if (!manifest.includes('android.permission.CAMERA')) {
  const insertion = manifest.indexOf('<application')
  if (insertion < 0) throw new Error('Android manifest has no application element')
  manifest = `${manifest.slice(0, insertion)}    <uses-permission android:name="android.permission.CAMERA" />\n    <uses-feature android:name="android.hardware.camera.any" android:required="false" />\n\n    ${manifest.slice(insertion)}`
}
const appPattern = /<application\b[^>]*>/
const app = manifest.match(appPattern)?.[0]
if (!app) throw new Error('Android manifest has no application tag')
const setAttr = (tag, name, value) => {
  const pattern = new RegExp(`\\s${name}="[^"]*"`)
  return pattern.test(tag) ? tag.replace(pattern, ` ${name}="${value}"`) : tag.replace(/>$/, ` ${name}="${value}">`)
}
let updated = setAttr(app, 'android:icon', '@drawable/elephant_launcher')
updated = setAttr(updated, 'android:roundIcon', '@drawable/elephant_launcher')
manifest = manifest.replace(app, updated)
fs.writeFileSync(manifestPath, manifest)

let gradle = fs.readFileSync(gradlePath, 'utf8')
if (!gradle.includes('useLegacyPackaging = true')) {
  gradle = gradle.replace(/android\s*\{/, `android {\n    packaging {\n        jniLibs {\n            useLegacyPackaging = true\n        }\n    }`)
  fs.writeFileSync(gradlePath, gradle)
}
NODE

  grep -q 'android.permission.CAMERA' "$ANDROID_MANIFEST"
  grep -q 'android:icon="@drawable/elephant_launcher"' "$ANDROID_MANIFEST"
  grep -q 'useLegacyPackaging = true' "$ANDROID_GRADLE_FILE"
}

verify_android_renderer() {
  test -f "$RENDERER_OUT/index.html"
  if grep -R -l --include='*.js' '__vite-browser-external' "$RENDERER_OUT" >/dev/null 2>&1; then
    echo "Android renderer contains unresolved Node builtin stubs:" >&2
    grep -R -l --include='*.js' '__vite-browser-external' "$RENDERER_OUT" >&2 || true
    exit 1
  fi
  echo '[android-apk] renderer contains no unresolved Vite Node builtin stubs'
}

verify_single_target_abi() {
  local apk="$1"
  local packaged_abis
  packaged_abis="$(unzip -Z1 "$apk" | sed -n 's#^lib/\([^/]*\)/.*#\1#p' | sort -u)"
  test -n "$packaged_abis"
  if [ "$packaged_abis" != "$(expected_android_abi)" ]; then
    echo "Expected only ABI $(expected_android_abi), got: $packaged_abis" >&2
    exit 1
  fi
}

require_cmd cargo
require_cmd node
require_cmd pnpm
require_cmd unzip

SDK_ROOT="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-}}"
test -n "$SDK_ROOT" || { echo 'ANDROID_HOME or ANDROID_SDK_ROOT is not set.' >&2; exit 1; }
test -f "$TAURI_DIR/$ANDROID_CONFIG"

case "$ANDROID_BUILD_PROFILE" in debug|release) ;; *) echo 'ANDROID_BUILD_PROFILE must be debug or release.' >&2; exit 1 ;; esac

if [ ! -d "$TAURI_DIR/gen/android" ]; then
  cd "$TAURI_DIR"
  cargo tauri android init --config "$ANDROID_CONFIG"
fi

install_generated_android_contracts
rm -rf "$ANDROID_JNI_LIBS" "$APK_ROOT"
mkdir -p "$ANDROID_JNI_LIBS" "$APK_ROOT"

cd "$TAURI_DIR"
BUILD_ARGS=(android build --apk --target "$ANDROID_TARGET" --config "$ANDROID_CONFIG")
if [ "$ANDROID_BUILD_PROFILE" = debug ]; then BUILD_ARGS+=(--debug); fi

printf '[android-apk] profile=%s target=%s\n' "$ANDROID_BUILD_PROFILE" "$ANDROID_TARGET"
ELEPHANTNOTE_ANDROID_BUILD=1 cargo tauri "${BUILD_ARGS[@]}"
verify_android_renderer

if [ "$ANDROID_BUILD_PROFILE" = release ]; then
  UNSIGNED_APK="$(find "$APK_ROOT" -type f -name '*-release-unsigned.apk' -print | head -n 1)"
  test -s "$UNSIGNED_APK"
  require_cmd keytool
  APKSIGNER="$(find "$SDK_ROOT/build-tools" -type f -name apksigner -print 2>/dev/null | sort | tail -n 1)"
  test -x "$APKSIGNER"
  DEBUG_KEYSTORE="${ANDROID_DEBUG_KEYSTORE:-$HOME/.android/debug.keystore}"
  mkdir -p "$(dirname "$DEBUG_KEYSTORE")"
  if [ ! -f "$DEBUG_KEYSTORE" ]; then
    keytool -genkeypair -noprompt -keystore "$DEBUG_KEYSTORE" -storepass android \
      -alias androiddebugkey -keypass android -dname 'CN=Android Debug,O=Android,C=US' \
      -keyalg RSA -keysize 2048 -validity 10000 >/dev/null
  fi
  FINAL_APK="${UNSIGNED_APK%-unsigned.apk}-review-signed.apk"
  "$APKSIGNER" sign --ks "$DEBUG_KEYSTORE" --ks-key-alias androiddebugkey \
    --ks-pass pass:android --key-pass pass:android --out "$FINAL_APK" "$UNSIGNED_APK"
  "$APKSIGNER" verify --verbose "$FINAL_APK"
  rm -f "$UNSIGNED_APK"
else
  FINAL_APK="$(find "$APK_ROOT" -type f -name '*-debug.apk' -print | head -n 1)"
fi

test -s "$FINAL_APK"
verify_single_target_abi "$FINAL_APK"
APK_BYTES="$(file_size_bytes "$FINAL_APK")"
MAX_BYTES=$((ANDROID_APK_MAX_MIB * 1024 * 1024))
APK_MIB="$(awk -v bytes="$APK_BYTES" 'BEGIN { printf "%.2f", bytes / 1024 / 1024 }')"
printf '[android-apk] output=%s\n' "$FINAL_APK"
printf '[android-apk] size=%sMiB bytes=%s\n' "$APK_MIB" "$APK_BYTES"
if [ "$ANDROID_BUILD_PROFILE" = release ] && [ "$APK_BYTES" -gt "$MAX_BYTES" ]; then
  echo "Release APK is ${APK_MIB}MiB, above the ${ANDROID_APK_MAX_MIB}MiB limit." >&2
  exit 1
fi
