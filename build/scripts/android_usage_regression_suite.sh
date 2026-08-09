#!/usr/bin/env bash
set -uo pipefail

PACKAGE_ID="${ANDROID_PACKAGE_ID:-com.elephantnote.app}"
ACTIVITY="${ANDROID_ACTIVITY:-com.elephantnote.app/.MainActivity}"
CATALOG="${ANDROID_USAGE_CATALOG:-tests/app/usage/android/scenarios.json}"
DEVICE_UI_DUMP="/sdcard/elephant-usage-window.xml"
RESULTS_TSV="${ANDROID_USAGE_RESULTS_TSV:-android-usage-results.tsv}"
RESULTS_JSON="${ANDROID_USAGE_RESULTS_JSON:-android-usage-summary.json}"
RESULTS_JUNIT="${ANDROID_USAGE_RESULTS_JUNIT:-android-usage-junit.xml}"
LOG_FILE="${ANDROID_USAGE_LOG:-android-usage-logcat.txt}"
CURRENT_SCENARIO=""
CURRENT_STARTED_AT=0
FAILURES=0

: > "$RESULTS_TSV"
adb logcat -c

capture_ui() {
  local destination="$1"
  adb shell uiautomator dump "$DEVICE_UI_DUMP" >/dev/null 2>&1 || true
  adb pull "$DEVICE_UI_DUMP" "$destination" >/dev/null 2>&1 || true
  test -s "$destination"
}

capture_screen() {
  local destination="$1"
  adb exec-out screencap -p > "$destination"
  test -s "$destination"
}

assert_not_blank() {
  local screenshot="$1"
  local label="$2"
  python3 - "$screenshot" "$label" <<'PY'
import sys
from PIL import Image, ImageStat

path, label = sys.argv[1:]
image = Image.open(path).convert('RGB')
width, height = image.size
region = image.crop((max(0, int(width * .03)), max(0, int(height * .04)), min(width, int(width * .97)), min(height, int(height * .94))))
pixels = list(region.getdata())
white = sum(1 for red, green, blue in pixels if red >= 245 and green >= 245 and blue >= 245)
non_white_ratio = 1 - white / max(1, len(pixels))
contrast = max(ImageStat.Stat(region).stddev)
print(f'[android-usage] {label} non_white_ratio={non_white_ratio:.4f} contrast={contrast:.2f}')
if non_white_ratio < 0.005 or contrast < 4.0:
    raise SystemExit(f'{label}: screenshot is effectively a uniform blank surface')
PY
}

capture_checkpoint() {
  local prefix="$1"
  capture_ui "${prefix}.xml"
  capture_screen "${prefix}.png"
  assert_not_blank "${prefix}.png" "$prefix"
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
print(f'[android-usage] {label}_mean_delta={mean_delta:.3f} bbox={bbox}')
if not bbox or mean_delta < minimum:
    raise SystemExit(f'{label}: expected UI transition was not rendered')
PY
}

assert_process_alive() {
  local pid
  pid="$(adb shell pidof "$PACKAGE_ID" | tr -d '\r' || true)"
  test -n "$pid" || {
    echo "Elephant process is no longer alive." >&2
    return 1
  }
}

assert_no_renderer_regression() {
  adb logcat -d -v threadtime > "$LOG_FILE"
  local app_pid
  app_pid="$(adb shell pidof "$PACKAGE_ID" | awk '{print $1}' || true)"

  if ! python3 - "$LOG_FILE" "$PACKAGE_ID" "$app_pid" <<'PY'
import sys
from pathlib import Path

log_path, package_id, app_pid = sys.argv[1], sys.argv[2], sys.argv[3].strip()
lines = Path(log_path).read_text(errors='replace').splitlines()
failures = []
for line in lines:
    fields = line.split()
    package_crash = package_id in line and ('Process:' in line or 'Fatal signal' in line)
    pid_crash = bool(app_pid) and len(fields) > 3 and fields[2] == app_pid and any(
        marker in line for marker in ('FATAL EXCEPTION', 'SIGABRT', 'SIGSEGV')
    )
    if package_crash or pid_crash:
        failures.append(line)
if failures:
    print('A fatal Elephant Android crash was detected during app usage testing.', file=sys.stderr)
    for line in failures[-80:]:
        print(line, file=sys.stderr)
    raise SystemExit(1)
PY
  then
    return 1
  fi
  if grep -Eq 'Tauri/Console:.*(Uncaught|ReferenceError|TypeError|SyntaxError)|Unhandled promise rejection|Command tauri_vault_read_binary not found|search\.initVault is not a function' "$LOG_FILE"; then
    echo "A renderer regression was detected during app usage testing." >&2
    grep -E 'Tauri/Console:.*(Uncaught|ReferenceError|TypeError|SyntaxError)|Unhandled promise rejection|Command tauri_vault_read_binary not found|search\.initVault is not a function' "$LOG_FILE" >&2 || true
    return 1
  fi
}

wait_for_workspace() {
  local destination="$1"
  local deadline=$((SECONDS + 35))
  while [ "$SECONDS" -lt "$deadline" ]; do
    capture_ui "$destination" || true
    if [ -s "$destination" ] && grep -Eq 'Search notes|Open navigation|Getting Started' "$destination"; then
      return 0
    fi
    sleep 2
  done
  echo 'Timed out waiting for the mobile workspace.' >&2
  [ -s "$destination" ] && cat "$destination" >&2
  return 1
}

return_to_workspace() {
  local attempt
  for attempt in 1 2 3 4 5; do
    capture_ui android-usage-workspace.xml || true
    if [ -s android-usage-workspace.xml ] && grep -Eq 'Search notes|Open navigation' android-usage-workspace.xml; then
      return 0
    fi
    adb shell input keyevent 4 >/dev/null 2>&1 || true
    sleep 1
  done

  adb shell am force-stop "$PACKAGE_ID"
  adb shell am start -W -n "$ACTIVITY" >/dev/null
  wait_for_workspace android-usage-workspace.xml
}

open_search() {
  local prefix="$1"
  return_to_workspace
  capture_checkpoint "${prefix}-workspace"
  tap_ui_node "${prefix}-workspace.xml" 'Search notes'
  sleep 2
  capture_checkpoint "${prefix}-open"
  assert_screens_differ "${prefix}-workspace.png" "${prefix}-open.png" 1.0 "${prefix}_open"
}

close_search() {
  local screenshot="$1"
  adb shell input keyevent 4 >/dev/null 2>&1 || true
  sleep 1
  tap_relative_to_screenshot "$screenshot" 0.50 0.84
  sleep 2
}

open_seeded_note() {
  local prefix="$1"
  return_to_workspace
  capture_checkpoint "${prefix}-workspace"
  tap_ui_node "${prefix}-workspace.xml" 'Open navigation'
  sleep 2
  capture_checkpoint "${prefix}-drawer"
  grep -q 'Getting Started' "${prefix}-drawer.xml"
  tap_ui_node "${prefix}-drawer.xml" 'Getting Started'
  sleep 2
  capture_checkpoint "${prefix}-expanded"

  if grep -q 'Welcome' "${prefix}-expanded.xml"; then
    tap_ui_node "${prefix}-expanded.xml" 'Welcome'
  elif grep -q 'Getting Started.md' "${prefix}-expanded.xml"; then
    tap_ui_node "${prefix}-expanded.xml" 'Getting Started.md'
  else
    echo 'The Getting Started folder exposed no seeded note.' >&2
    cat "${prefix}-expanded.xml" >&2
    return 1
  fi

  local deadline=$((SECONDS + 35))
  while [ "$SECONDS" -lt "$deadline" ]; do
    capture_ui "${prefix}-open.xml" || true
    if [ -s "${prefix}-open.xml" ] && grep -Fq 'Close note' "${prefix}-open.xml"; then
      adb exec-out screencap -p > "${prefix}-open.png"
      assert_screens_differ "${prefix}-expanded.png" "${prefix}-open.png" 1.0 "${prefix}_open"
      return 0
    fi
    sleep 2
  done
  echo 'The seeded note did not open in the Android editor.' >&2
  [ -s "${prefix}-open.xml" ] && cat "${prefix}-open.xml" >&2
  return 1
}

scenario_library_layout_toggle() {
  return_to_workspace
  capture_checkpoint android-layout-before

  local initial_label target_label
  if grep -q 'Show notes as list' android-layout-before.xml; then
    initial_label='Show notes as list'
    target_label='Show notes as grid'
  elif grep -q 'Show notes as grid' android-layout-before.xml; then
    initial_label='Show notes as grid'
    target_label='Show notes as list'
  else
    echo 'The mobile library layout control is absent.' >&2
    cat android-layout-before.xml >&2
    return 1
  fi

  tap_ui_node android-layout-before.xml "$initial_label"
  sleep 2
  capture_checkpoint android-layout-after
  assert_screens_differ android-layout-before.png android-layout-after.png 0.10 library_layout_toggle
  grep -q "$target_label" android-layout-after.xml

  tap_ui_node android-layout-after.xml "$target_label"
  sleep 2
  capture_checkpoint android-layout-restored
  grep -q "$initial_label" android-layout-restored.xml
}

scenario_library_sort_sheet() {
  return_to_workspace
  capture_checkpoint android-sort-before
  tap_ui_node android-sort-before.xml 'Sort notes'
  sleep 2
  capture_checkpoint android-sort-sheet
  grep -q 'Title A' android-sort-sheet.xml
  assert_screens_differ android-sort-before.png android-sort-sheet.png 1.0 library_sort_sheet_open

  tap_ui_node android-sort-sheet.xml 'Title A'
  sleep 2
  capture_checkpoint android-sort-after
  if grep -q 'Title A' android-sort-after.xml; then
    echo 'The mobile sort sheet did not close after selecting title ordering.' >&2
    cat android-sort-after.xml >&2
    return 1
  fi
}

scenario_open_existing_note() {
  open_seeded_note android-note
}

scenario_drawer_back_close() {
  return_to_workspace
  capture_checkpoint android-drawer-back-before
  tap_ui_node android-drawer-back-before.xml 'Open navigation'
  sleep 2
  capture_checkpoint android-drawer-back-open
  grep -q 'Getting Started' android-drawer-back-open.xml
  adb shell input keyevent 4
  sleep 2
  capture_checkpoint android-drawer-back-closed
  assert_screens_differ android-drawer-back-open.png android-drawer-back-closed.png 1.0 drawer_back_close
  grep -Eq 'Search notes|Open navigation' android-drawer-back-closed.xml
}

scenario_note_back_roundtrip() {
  open_seeded_note android-note-back
  adb shell input keyevent 4
  sleep 3
  capture_checkpoint android-note-back-workspace
  grep -Eq 'Search notes|Open navigation' android-note-back-workspace.xml
  assert_screens_differ android-note-back-open.png android-note-back-workspace.png 1.0 note_back_roundtrip
}

scenario_app_background_resume() {
  return_to_workspace
  capture_checkpoint android-resume-before
  adb shell input keyevent 3
  sleep 3
  adb shell am start -W -n "$ACTIVITY" >/dev/null
  wait_for_workspace android-resume-after.xml
  capture_screen android-resume-after.png
  assert_not_blank android-resume-after.png app_background_resume
}

scenario_process_relaunch() {
  return_to_workspace
  adb shell am force-stop "$PACKAGE_ID"
  sleep 2
  adb shell am start -W -n "$ACTIVITY" >/dev/null
  wait_for_workspace android-relaunch-after.xml
  capture_screen android-relaunch-after.png
  assert_not_blank android-relaunch-after.png process_relaunch
}

scenario_search_no_results() {
  open_search android-search-empty
  tap_relative_to_screenshot android-search-empty-open.png 0.50 0.17
  adb shell input text 'zzzxxyy-no-match-9173'
  sleep 4
  capture_checkpoint android-search-empty-results
  assert_screens_differ android-search-empty-open.png android-search-empty-results.png 0.25 search_no_results
  close_search android-search-empty-results.png
  wait_for_workspace android-search-empty-closed.xml
}

scenario_search_repeat_stability() {
  local cycle
  for cycle in 1 2 3; do
    open_search "android-search-repeat-${cycle}"
    close_search "android-search-repeat-${cycle}-open.png"
    wait_for_workspace "android-search-repeat-${cycle}-closed.xml"
    capture_screen "android-search-repeat-${cycle}-closed.png"
    assert_not_blank "android-search-repeat-${cycle}-closed.png" "search_repeat_${cycle}"
    assert_process_alive
    assert_no_renderer_regression
  done
}

scenario_settings_back_navigation() {
  return_to_workspace
  capture_checkpoint android-settings-back-before
  tap_ui_node android-settings-back-before.xml 'Settings'
  sleep 2
  capture_checkpoint android-settings-back-open
  grep -q 'Close settings' android-settings-back-open.xml
  adb shell input keyevent 4
  sleep 2
  capture_checkpoint android-settings-back-closed
  grep -Eq 'Search notes|Open navigation' android-settings-back-closed.xml
  assert_screens_differ android-settings-back-open.png android-settings-back-closed.png 1.0 settings_back_navigation
}

scenario_keyboard_dismissal() {
  open_search android-keyboard
  tap_relative_to_screenshot android-keyboard-open.png 0.50 0.17
  adb shell input text 'Getting'
  sleep 2
  capture_checkpoint android-keyboard-visible
  adb shell input keyevent 4
  sleep 2
  capture_checkpoint android-keyboard-hidden
  assert_screens_differ android-keyboard-visible.png android-keyboard-hidden.png 0.15 keyboard_dismissal
  tap_relative_to_screenshot android-keyboard-hidden.png 0.50 0.84
  sleep 2
  wait_for_workspace android-keyboard-closed.xml
}

scenario_navigation_stress() {
  local cycle
  return_to_workspace
  for cycle in 1 2 3; do
    capture_ui "android-stress-${cycle}-workspace.xml"
    tap_ui_node "android-stress-${cycle}-workspace.xml" 'Open navigation'
    sleep 1
    capture_checkpoint "android-stress-${cycle}-drawer"
    adb shell input keyevent 4
    sleep 1
    wait_for_workspace "android-stress-${cycle}-after-drawer.xml"

    tap_ui_node "android-stress-${cycle}-after-drawer.xml" 'Settings'
    sleep 1
    capture_checkpoint "android-stress-${cycle}-settings"
    tap_ui_node "android-stress-${cycle}-settings.xml" 'Close settings'
    sleep 1
    wait_for_workspace "android-stress-${cycle}-after-settings.xml"

    tap_ui_node "android-stress-${cycle}-after-settings.xml" 'Search notes'
    sleep 1
    capture_checkpoint "android-stress-${cycle}-search"
    tap_relative_to_screenshot "android-stress-${cycle}-search.png" 0.50 0.84
    sleep 1
    wait_for_workspace "android-stress-${cycle}-after-search.xml"
    assert_process_alive
    assert_no_renderer_regression
  done
  capture_checkpoint android-stress-complete
}

record_result() {
  local id="$1"
  local status="$2"
  local duration="$3"
  local message="$4"
  message="${message//$'\t'/ }"
  message="${message//$'\n'/ }"
  printf '%s\t%s\t%s\t%s\n' "$id" "$status" "$duration" "$message" >> "$RESULTS_TSV"
}

emit_reports() {
  python3 - "$CATALOG" "$RESULTS_TSV" "$RESULTS_JSON" "$RESULTS_JUNIT" <<'PY'
import json
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

catalog_path, tsv_path, json_path, junit_path = map(Path, sys.argv[1:])
catalog = json.loads(catalog_path.read_text())
metadata = {item['id']: item for item in catalog['scenarios']}
results = []
if tsv_path.exists():
    for raw in tsv_path.read_text().splitlines():
        if not raw.strip():
            continue
        scenario_id, status, duration, message = (raw.split('\t', 3) + [''])[:4]
        results.append({
            'id': scenario_id,
            'status': status,
            'durationSeconds': int(duration),
            'message': message,
            'description': metadata.get(scenario_id, {}).get('description', ''),
            'regression': metadata.get(scenario_id, {}).get('regression', ''),
            'tags': metadata.get(scenario_id, {}).get('tags', [])
        })
summary = {
    'schemaVersion': 1,
    'platform': 'android',
    'passed': sum(item['status'] == 'passed' for item in results),
    'failed': sum(item['status'] == 'failed' for item in results),
    'results': results
}
json_path.write_text(json.dumps(summary, indent=2) + '\n')

testsuite = ET.Element('testsuite', {
    'name': 'Elephant Android app usage regressions',
    'tests': str(len(results)),
    'failures': str(summary['failed']),
    'time': str(sum(item['durationSeconds'] for item in results))
})
for item in results:
    case = ET.SubElement(testsuite, 'testcase', {
        'classname': 'android.app-usage',
        'name': item['id'],
        'time': str(item['durationSeconds'])
    })
    if item['status'] == 'failed':
        failure = ET.SubElement(case, 'failure', {'message': item['message'] or 'scenario failed'})
        failure.text = item['regression']
ET.ElementTree(testsuite).write(junit_path, encoding='utf-8', xml_declaration=True)
PY
}

run_scenario() {
  local id="$1"
  local function_name="$2"
  local scenario_log="android-usage-${id}.log"
  CURRENT_SCENARIO="$id"
  CURRENT_STARTED_AT=$SECONDS
  printf '[android-usage] START %s\n' "$id"

  set +e
  (
    set -euo pipefail
    "$function_name"
  ) > >(tee "$scenario_log") 2>&1
  local status=$?
  set -e

  local duration=$((SECONDS - CURRENT_STARTED_AT))
  if [ "$status" -eq 0 ]; then
    if assert_process_alive && assert_no_renderer_regression; then
      status=0
    else
      status=$?
    fi
  fi
  if [ "$status" -eq 0 ]; then
    record_result "$id" passed "$duration" ''
    printf '[android-usage] PASS %s (%ss)\n' "$id" "$duration"
  else
    local message
    message="$(tail -n 8 "$scenario_log" | tr '\n' ' ' | sed 's/[[:space:]]\+/ /g')"
    record_result "$id" failed "$duration" "$message"
    FAILURES=$((FAILURES + 1))
    printf '[android-usage] FAIL %s (%ss)\n' "$id" >&2
    return_to_workspace >/dev/null 2>&1 || true
  fi
  CURRENT_SCENARIO=""
  return 0
}

mapfile -t SCENARIOS < <(python3 - "$CATALOG" <<'PY'
import json
import sys
for scenario in json.load(open(sys.argv[1]))['scenarios']:
    if scenario.get('runner') == 'android-regression':
        print(scenario['id'])
PY
)

set -e
for scenario in "${SCENARIOS[@]}"; do
  case "$scenario" in
    library-layout-toggle) run_scenario "$scenario" scenario_library_layout_toggle ;;
    library-sort-sheet) run_scenario "$scenario" scenario_library_sort_sheet ;;
    open-existing-note) run_scenario "$scenario" scenario_open_existing_note ;;
    drawer-back-close) run_scenario "$scenario" scenario_drawer_back_close ;;
    note-back-roundtrip) run_scenario "$scenario" scenario_note_back_roundtrip ;;
    app-background-resume) run_scenario "$scenario" scenario_app_background_resume ;;
    process-relaunch) run_scenario "$scenario" scenario_process_relaunch ;;
    search-no-results) run_scenario "$scenario" scenario_search_no_results ;;
    search-repeat-stability) run_scenario "$scenario" scenario_search_repeat_stability ;;
    settings-back-navigation) run_scenario "$scenario" scenario_settings_back_navigation ;;
    keyboard-dismissal) run_scenario "$scenario" scenario_keyboard_dismissal ;;
    navigation-stress) run_scenario "$scenario" scenario_navigation_stress ;;
    *)
      record_result "$scenario" failed 0 "No Android usage implementation exists"
      FAILURES=$((FAILURES + 1))
      ;;
  esac
done

emit_reports
adb logcat -d -v threadtime > "$LOG_FILE" || true
printf '[android-usage] complete scenarios=%s failures=%s\n' "${#SCENARIOS[@]}" "$FAILURES"
[ "$FAILURES" -eq 0 ]
