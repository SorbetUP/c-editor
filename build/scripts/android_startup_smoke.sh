#!/usr/bin/env bash
set -euo pipefail

PACKAGE_ID="${ANDROID_PACKAGE_ID:-com.elephantnote.app}"
ACTIVITY="${ANDROID_ACTIVITY:-com.elephantnote.app/.MainActivity}"
APK_ROOT="${ANDROID_APK_ROOT:-Elephant/backend/tauri/gen/android/app/build/outputs/apk}"
STARTUP_WAIT_SECONDS="${ANDROID_STARTUP_WAIT_SECONDS:-45}"
LOG_FILE="${ANDROID_STARTUP_LOG:-android-startup-logcat.txt}"
SCREENSHOT_FILE="${ANDROID_STARTUP_SCREENSHOT:-android-startup-screen.png}"
UI_DUMP_FILE="${ANDROID_STARTUP_UI_DUMP:-android-startup-window.xml}"
DEVICE_UI_DUMP="/sdcard/elephant-startup-window.xml"

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Missing required command: $1" >&2
    exit 1
  }
}

require_cmd adb

capture_ui() {
  local destination="$1"
  adb shell uiautomator dump "$DEVICE_UI_DUMP" >/dev/null 2>&1 || true
  adb pull "$DEVICE_UI_DUMP" "$destination" >/dev/null 2>&1 || true
  test -s "$destination"
}

tap_ui_node() {
  local dump_file="$1"
  local needle="$2"
  local coordinates
  coordinates="$(python3 - "$dump_file" "$needle" <<'PY'
import re
import sys
import xml.etree.ElementTree as ET

path, needle = sys.argv[1], sys.argv[2].casefold()
root = ET.parse(path).getroot()
for node in root.iter('node'):
    haystack = ' '.join([
        node.attrib.get('text', ''),
        node.attrib.get('content-desc', ''),
        node.attrib.get('hint', '')
    ]).casefold()
    if needle not in haystack:
        continue
    match = re.fullmatch(r'\[(\d+),(\d+)\]\[(\d+),(\d+)\]', node.attrib.get('bounds', ''))
    if not match:
        continue
    left, top, right, bottom = map(int, match.groups())
    print((left + right) // 2, (top + bottom) // 2)
    break
PY
)"
  if [ -z "$coordinates" ]; then
    echo "Unable to find Android UI node containing: $needle" >&2
    cat "$dump_file" >&2
    return 1
  fi
  read -r x y <<<"$coordinates"
  adb shell input tap "$x" "$y"
}

tap_relative_to_screenshot() {
  local screenshot="$1"
  local x_ratio="$2"
  local y_ratio="$3"
  local coordinates
  coordinates="$(python3 - "$screenshot" "$x_ratio" "$y_ratio" <<'PY'
import sys
from PIL import Image

path, x_ratio, y_ratio = sys.argv[1], float(sys.argv[2]), float(sys.argv[3])
width, height = Image.open(path).size
print(round(width * x_ratio), round(height * y_ratio))
PY
)"
  read -r x y <<<"$coordinates"
  adb shell input tap "$x" "$y"
}

assert_screens_differ() {
  local before="$1"
  local after="$2"
  local minimum="$3"
  local label="$4"
  python3 - "$before" "$after" "$minimum" "$label" <<'PY'
import sys
from PIL import Image, ImageChops, ImageStat

before_path, after_path, minimum, label = sys.argv[1], sys.argv[2], float(sys.argv[3]), sys.argv[4]
before = Image.open(before_path).convert('RGB')
after = Image.open(after_path).convert('RGB')
if before.size != after.size:
    raise SystemExit(f'{label}: screenshot dimensions changed unexpectedly')
difference = ImageChops.difference(before, after)
mean_delta = sum(ImageStat.Stat(difference).mean) / 3
bbox = difference.getbbox()
print(f'[android-startup] {label}_mean_delta={mean_delta:.3f} bbox={bbox}')
if not bbox or mean_delta < minimum:
    raise SystemExit(f'{label}: expected UI transition was not rendered')
PY
}

assert_no_renderer_regression() {
  adb logcat -d -v threadtime > "$LOG_FILE"
  local app_pid
  app_pid="$(adb shell pidof "$PACKAGE_ID" | awk '{print $1}' || true)"

  if ! python3 - "$LOG_FILE" "$PACKAGE_ID" "$app_pid" <<'PY'
import sys
from pathlib import Path

log_path, package_id, app_pid = sys.argv[1], sys.argv[2], sys.argv[3].strip()
failures = []
for line in Path(log_path).read_text(errors='replace').splitlines():
    fields = line.split()
    package_crash = package_id in line and ('Process:' in line or 'Fatal signal' in line)
    pid_crash = bool(app_pid) and len(fields) > 3 and fields[2] == app_pid and any(
        marker in line for marker in ('FATAL EXCEPTION', 'SIGABRT', 'SIGSEGV')
    )
    if package_crash or pid_crash:
        failures.append(line)
if failures:
    print('A fatal Elephant Android crash was detected.', file=sys.stderr)
    for line in failures[-80:]:
        print(line, file=sys.stderr)
    raise SystemExit(1)
PY
  then
    return 1
  fi
  if grep -Eq 'Tauri/Console:.*(Uncaught|ReferenceError|TypeError|SyntaxError)|Unhandled promise rejection|Command tauri_vault_read_binary not found|search\.initVault is not a function' "$LOG_FILE"; then
    echo "A renderer contract regression was detected during Android interaction testing." >&2
    grep -E 'Tauri/Console:.*(Uncaught|ReferenceError|TypeError|SyntaxError)|Unhandled promise rejection|Command tauri_vault_read_binary not found|search\.initVault is not a function' "$LOG_FILE" >&2 || true
    return 1
  fi
}

APK="${ANDROID_APK_PATH:-}"
if [ -z "$APK" ]; then
  APK="$(find "$APK_ROOT" -type f -name '*-debug.apk' -print | head -n 1)"
fi
if [ -z "$APK" ] || [ ! -s "$APK" ]; then
  echo "No non-empty Android debug APK found under $APK_ROOT" >&2
  exit 1
fi

printf '[android-startup] apk=%s\n' "$APK"
adb wait-for-device
adb install -r "$APK"
adb shell pm clear "$PACKAGE_ID" >/dev/null
adb shell settings put secure immersive_mode_confirmations confirmed >/dev/null 2>&1 || true
# The hosted Pixel image occasionally ANRs its launcher during boot. Stop only
# the launcher packages before testing Elephant so a foreign system dialog
# cannot cover the application or create a false product failure.
adb shell am force-stop com.google.android.apps.nexuslauncher >/dev/null 2>&1 || true
adb shell am force-stop com.android.launcher3 >/dev/null 2>&1 || true
adb shell input keyevent 4 >/dev/null 2>&1 || true
adb logcat -c
adb shell am force-stop "$PACKAGE_ID"

START_OUTPUT="$(adb shell am start -W -n "$ACTIVITY" 2>&1)"
printf '%s\n' "$START_OUTPUT"
if printf '%s\n' "$START_OUTPUT" | grep -Eqi 'Error|Exception|does not exist'; then
  echo "Android Activity Manager failed to launch Elephant." >&2
  exit 1
fi

READY=0
DEADLINE=$((SECONDS + STARTUP_WAIT_SECONDS))
while [ "$SECONDS" -lt "$DEADLINE" ]; do
  capture_ui "$UI_DUMP_FILE" || true
  if [ -s "$UI_DUMP_FILE" ]; then
    if grep -Eq 'Elephant.*isn.t responding|com\.elephantnote\.app' "$UI_DUMP_FILE" && grep -Eq 'Close app|Wait' "$UI_DUMP_FILE"; then
      echo "Elephant itself triggered an Android ANR dialog." >&2
      cat "$UI_DUMP_FILE" >&2
      exit 1
    fi
    if grep -Eq 'Pixel Launcher isn.t responding|System UI isn.t responding' "$UI_DUMP_FILE"; then
      tap_ui_node "$UI_DUMP_FILE" 'Close app' || tap_ui_node "$UI_DUMP_FILE" 'Wait' || true
      sleep 2
      continue
    fi
    if grep -Eq 'Choose your first vault|Stockage privé|Dossier Android|Search notes' "$UI_DUMP_FILE"; then
      READY=1
      break
    fi
    if grep -Eq 'Elephant n[^<]*(pas pu démarrer|a pas pu démarrer)' "$UI_DUMP_FILE"; then
      echo "Elephant rendered its startup failure surface." >&2
      cat "$UI_DUMP_FILE" >&2
      exit 1
    fi
  fi
  sleep 3
done

adb exec-out screencap -p > "$SCREENSHOT_FILE"
test -s "$SCREENSHOT_FILE"

PID="$(adb shell pidof "$PACKAGE_ID" | tr -d '\r' || true)"
if [ -z "$PID" ]; then
  echo "Elephant process is not alive after startup." >&2
  assert_no_renderer_regression
  exit 1
fi
printf '[android-startup] pid=%s\n' "$PID"

ACTIVITY_DUMP="$(adb shell dumpsys activity activities || true)"
WINDOW_DUMP="$(adb shell dumpsys window windows || true)"
RESUMED="$(printf '%s\n' "$ACTIVITY_DUMP" | grep -E -m 1 'topResumedActivity|mResumedActivity|ResumedActivity' || true)"
FOCUSED="$(printf '%s\n' "$WINDOW_DUMP" | grep -E -m 1 'mCurrentFocus|mFocusedApp' || true)"
VISIBLE="$(printf '%s\n' "$ACTIVITY_DUMP" | grep -E -m 1 "ActivityRecord.*${PACKAGE_ID}.*(RESUMED|visible=true)" || true)"
PERMISSION_UI="$(printf '%s\n' "$ACTIVITY_DUMP" | grep -Ei -m 1 'permissioncontroller.*(GrantPermissionsActivity|permission\.ui)' || true)"

{
  printf '\n[android-startup-diagnostics]\n'
  printf 'pid=%s\n' "$PID"
  printf 'resumed=%s\n' "$RESUMED"
  printf 'focused=%s\n' "$FOCUSED"
  printf 'visible=%s\n' "$VISIBLE"
  printf 'permission_ui=%s\n' "$PERMISSION_UI"
  printf 'mobile_shell_ready=%s\n' "$READY"
} >> "$LOG_FILE"

if ! printf '%s\n' "$RESUMED" "$FOCUSED" "$VISIBLE" | grep -q "$PACKAGE_ID"; then
  echo "Elephant is alive but no resumed, focused, or visible Activity record was found." >&2
  exit 1
fi

if [ -n "$PERMISSION_UI" ]; then
  echo "Android permission UI interrupted startup: $PERMISSION_UI" >&2
  exit 1
fi

if [ "$READY" -ne 1 ]; then
  echo "Elephant never rendered the mobile vault or workspace UI within ${STARTUP_WAIT_SECONDS}s." >&2
  if [ -s "$UI_DUMP_FILE" ]; then cat "$UI_DUMP_FILE" >&2; fi
  exit 1
fi

CAMERA_LINE="$(adb shell dumpsys package "$PACKAGE_ID" | grep -m 1 'android.permission.CAMERA:' || true)"
printf '[android-startup] camera=%s\n' "$CAMERA_LINE"
printf 'camera=%s\n' "$CAMERA_LINE" >> "$LOG_FILE"
if printf '%s' "$CAMERA_LINE" | grep -q 'granted=true'; then
  echo "Camera permission was granted during startup; it must only be requested by a user action." >&2
  exit 1
fi

assert_no_renderer_regression

python3 - "$SCREENSHOT_FILE" <<'PY'
import sys
from PIL import Image, ImageStat

path = sys.argv[1]
image = Image.open(path).convert('RGB')
width, height = image.size
left = max(0, int(width * 0.03))
top = max(0, int(height * 0.04))
right = min(width, int(width * 0.97))
bottom = min(height, int(height * 0.94))
region = image.crop((left, top, right, bottom))
pixels = list(region.getdata())
white = sum(1 for red, green, blue in pixels if red >= 245 and green >= 245 and blue >= 245)
non_white_ratio = 1 - white / max(1, len(pixels))
contrast = max(ImageStat.Stat(region).stddev)
print(
    f'[android-startup] screenshot={width}x{height} '
    f'non_white_ratio={non_white_ratio:.4f} contrast={contrast:.2f}'
)
if non_white_ratio < 0.005 or contrast < 4.0:
    raise SystemExit('Android startup screenshot is effectively a uniform blank surface')
PY

# Exercise the exact interactions that regressed on the physical phone.
capture_ui "$UI_DUMP_FILE"
tap_ui_node "$UI_DUMP_FILE" 'Search notes'
sleep 3
capture_ui android-search-window.xml
adb exec-out screencap -p > android-search-screen.png
assert_screens_differ "$SCREENSHOT_FILE" android-search-screen.png 3.0 search_open

# Element Plus teleports the dialog outside the accessible WebView subtree on
# Android, so uiautomator still reports the background. Focus the visibly
# rendered search field from its screenshot-relative position instead.
tap_relative_to_screenshot android-search-screen.png 0.50 0.17
adb shell input text 'Untitled'
sleep 4
capture_ui android-search-results-window.xml || true
adb exec-out screencap -p > android-search-results-screen.png
assert_screens_differ android-search-screen.png android-search-results-screen.png 0.35 search_query
assert_no_renderer_regression

# Hide the software keyboard first, then close the dialog by tapping its
# backdrop. Otherwise the backdrop tap can land on the keyboard itself.
adb shell input keyevent 4
sleep 1
tap_relative_to_screenshot android-search-results-screen.png 0.50 0.82
sleep 2
capture_ui android-workspace-after-search.xml
adb exec-out screencap -p > android-workspace-after-search.png
assert_screens_differ android-search-results-screen.png android-workspace-after-search.png 2.0 search_close

tap_ui_node android-workspace-after-search.xml 'Settings'
sleep 3
capture_ui android-settings-window.xml
adb exec-out screencap -p > android-settings-screen.png
assert_screens_differ android-workspace-after-search.png android-settings-screen.png 3.0 settings_open
assert_no_renderer_regression
if ! grep -Eq 'ElephantNote settings|Close settings|Color mode|Settings sections' android-settings-window.xml; then
  echo "The Android Settings page did not expose its dialog controls." >&2
  cat android-settings-window.xml >&2
  exit 1
fi

# Close settings through its explicit accessible control before opening the drawer.
tap_ui_node android-settings-window.xml 'Close settings'
sleep 2
capture_ui android-workspace-after-settings.xml
adb exec-out screencap -p > android-workspace-after-settings.png
assert_screens_differ android-settings-screen.png android-workspace-after-settings.png 3.0 settings_close

tap_ui_node android-workspace-after-settings.xml 'Open navigation'
sleep 2
capture_ui android-drawer-window.xml
adb exec-out screencap -p > android-drawer-screen.png
assert_screens_differ android-workspace-after-settings.png android-drawer-screen.png 3.0 drawer_open
assert_no_renderer_regression
if ! grep -Eq 'All notes|Getting Started' android-drawer-window.xml; then
  echo "The Android navigation drawer did not expose the vault tree." >&2
  cat android-drawer-window.xml >&2
  exit 1
fi

printf '[android-startup] success package=%s pid=%s mobile_shell_ready=true search_ready=true settings_ready=true drawer_ready=true camera_not_requested=true white_screen=false\n' "$PACKAGE_ID" "$PID"
