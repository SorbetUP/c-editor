use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .ancestors()
    .nth(3)
    .expect("Elephant/backend/tauri must live three levels under the repository root")
    .to_path_buf()
}

fn read_json(path: &str) -> Value {
  let raw = fs::read_to_string(repo_root().join(path)).expect("platform contract JSON file must be readable");
  serde_json::from_str(&raw).expect("platform contract JSON file must be valid JSON")
}

fn read_text(path: &str) -> String {
  fs::read_to_string(repo_root().join(path)).expect("platform contract file must be readable")
}

fn assert_workspace_frontend_hook(config: &Value, pointer: &str) {
  assert_eq!(
    config.pointer(&format!("{pointer}/script")).and_then(Value::as_str),
    Some("pnpm tauri:web:build"),
    "Tauri builds must use the shared frontend build"
  );
  assert_eq!(
    config.pointer(&format!("{pointer}/cwd")).and_then(Value::as_str),
    Some("../../.."),
    "Tauri frontend hooks must execute from the repository workspace root"
  );
}

#[test]
fn desktop_tauri_config_bundles_controlled_official_addon_runtimes() {
  let config = read_json("Elephant/backend/tauri/tauri.conf.json");
  let resources = config
    .pointer("/bundle/resources")
    .and_then(Value::as_array)
    .expect("desktop resources must be an array");
  assert_eq!(resources.len(), 1, "desktop should bundle one controlled addon resource root");
  assert_eq!(resources[0].as_str(), Some("resources/official-addons"), "desktop addon resources must use the controlled staging root");
  assert_workspace_frontend_hook(&config, "/build/beforeBuildCommand");

  let icons = config.pointer("/bundle/icon").and_then(Value::as_array).expect("desktop icons must be an array");
  assert!(icons.iter().any(|item| item.as_str() == Some("../../assets/static/icon.png")), "desktop Tauri config must include a PNG icon for Linux");
  assert!(icons.iter().any(|item| item.as_str() == Some("../../assets/static/icon.icns")), "desktop Tauri config must keep the macOS icon");
}

#[test]
fn linux_override_does_not_reuse_macos_window_chrome() {
  let config = read_json("Elephant/backend/tauri/tauri.linux.conf.json");
  let targets = config.pointer("/bundle/targets").and_then(Value::as_array).expect("Linux targets must be explicit");
  assert!(targets.iter().any(|item| item.as_str() == Some("deb")), "Linux build must produce a deb package");
  assert!(targets.iter().any(|item| item.as_str() == Some("appimage")), "Linux build must produce an AppImage package");

  let windows = config.pointer("/app/windows").and_then(Value::as_array).expect("Linux windows must be explicit");
  let first = windows.first().expect("Linux config must define a main window");
  assert!(first.get("titleBarStyle").is_none(), "Linux override must not inherit macOS titleBarStyle");
  assert_eq!(first.get("decorations").and_then(Value::as_bool), Some(false));
}

#[test]
fn android_override_is_mobile_only_and_never_bundles_process_services() {
  let config = read_json("Elephant/backend/tauri/tauri.android.conf.json");
  assert_workspace_frontend_hook(&config, "/build/beforeBuildCommand");

  let resources = config.pointer("/bundle/resources").and_then(Value::as_array).expect("Android resources must be explicit");
  assert!(resources.is_empty(), "Android must not include desktop addon process resources");

  let icons = config.pointer("/bundle/icon").and_then(Value::as_array).expect("Android icons must be explicit");
  assert_eq!(icons.len(), 1, "Android should only use the portable PNG icon");
  assert_eq!(icons[0].as_str(), Some("../../assets/static/icon.png"));
}

#[test]
fn android_scripts_always_use_the_android_config() {
  let package_json = read_json("package.json");
  let scripts = package_json.get("scripts").and_then(Value::as_object).expect("package scripts must exist");
  let android_init = scripts.get("tauri:android:init").and_then(Value::as_str).unwrap_or("");
  let android_dev = scripts.get("tauri:android:dev").and_then(Value::as_str).unwrap_or("");
  assert!(android_init.contains("tauri.android.conf.json"), "Android init script must use the Android config");
  assert!(android_dev.contains("tauri.android.conf.json"), "Android dev script must use the Android config");

  let build_script = read_text("build/scripts/build_dev_apk.sh");
  assert!(build_script.contains("cargo tauri android init --config \"$ANDROID_CONFIG\""), "APK script must initialize Android with the Android config");
  assert!(
    build_script.contains("BUILD_ARGS=(android build --apk --target \"$ANDROID_TARGET\" --config \"$ANDROID_CONFIG\")"),
    "APK script must always build Android with the Android config"
  );
  assert!(
    build_script.contains("if [ \"$ANDROID_BUILD_PROFILE\" = debug ]; then BUILD_ARGS+=(--debug); fi"),
    "APK script must add debug mode only for debug builds"
  );
  assert!(
    build_script.contains("cargo tauri \"${BUILD_ARGS[@]}\""),
    "APK script must execute the profile-aware Android build arguments"
  );
  assert!(!build_script.contains("ELEPHANTNOTE_SKIP_LLAMA_BUNDLE"), "APK script must not carry obsolete core llama bundle switches");
}

#[test]
fn extracted_ai_process_and_configuration_runtimes_are_absent_from_core() {
  let root = repo_root();
  let lib_min = read_text("Elephant/backend/tauri/src/lib_min.rs");
  let core_commands = read_text("Elephant/backend/tauri/src/core_commands.rs");
  let compatibility = read_text("Elephant/frontend/app/services/elephantnoteClient/compatibilityCalls.js");
  let api_contracts = read_text("Elephant/shared/apiContracts.js");
  assert!(!root.join("Elephant/backend/tauri/src/tauri_extra_commands.rs").exists());
  assert!(!lib_min.contains("pub mod local_llama_runtime;"), "Open Models runtime must be owned by its physical addon package");
  assert!(!lib_min.contains("pub mod chat_runtime;"), "legacy core chat runtime must stay removed");
  assert!(!lib_min.contains("tauri_extra_commands::tauri_ai_config_"));
  assert!(!lib_min.contains("tauri_extra_commands::tauri_models_"));
  for marker in [
    "tauri_ai_config_get",
    "tauri_ai_config_set",
    "tauri_ai_config_test",
    "tauri_models_get_selection",
    "tauri_models_set_selection",
    "test_codex_cli",
    "test_tcp_endpoint",
    "PROVIDER_CONFIG_CATEGORY",
  ] {
    assert!(!core_commands.contains(marker), "optional core implementation leaked: {marker}");
  }
  for action in [
    "ai.config.get",
    "ai.config.set",
    "ai.config.test",
    "models.selection.get",
    "models.selection.set",
    "models.local.list",
    "models.download",
    "ocr.extract",
  ] {
    assert!(!compatibility.contains(action), "optional compatibility call leaked: {action}");
    assert!(!api_contracts.contains(action), "optional core API contract leaked: {action}");
  }

  let desktop_config = read_text("Elephant/backend/tauri/tauri.conf.json");
  let dev_script = read_text("build/scripts/build_dev.sh");
  assert!(!desktop_config.contains("ensure-tauri-llama-server"), "core Tauri config must not install llama-server");
  assert!(!desktop_config.contains("ensure-tauri-codex-runtime"), "core Tauri config must not install Codex");
  assert!(!dev_script.contains("ensure-tauri-llama-server"), "core development startup must not install llama-server");
  assert!(!dev_script.contains("ensure-tauri-codex-runtime"), "core development startup must not install Codex");
}

#[test]
fn extracted_search_inspection_index_is_absent_from_core() {
  let lib_min = read_text("Elephant/backend/tauri/src/lib_min.rs");
  let core_commands = read_text("Elephant/backend/tauri/src/core_commands.rs");
  let compatibility = read_text("Elephant/frontend/app/services/elephantnoteClient/compatibilityCalls.js");
  let api_contracts = read_text("Elephant/shared/apiContracts.js");
  assert!(!lib_min.contains("tauri_extra_commands::tauri_search_inspect"));
  assert!(!lib_min.contains("tauri_extra_commands::tauri_search_rebuild"));
  for marker in [
    "tauri_search_inspect",
    "tauri_search_rebuild",
    "fn build_search_index",
    "fn scan_markdown_notes",
    "fn extract_wikilinks",
    "SEARCH_INDEX_FILE",
    "portable-markdown-index",
  ] {
    assert!(!core_commands.contains(marker), "optional Search implementation leaked: {marker}");
  }
  for action in [
    "search.inspect",
    "search.rebuild",
    "search.concepts",
    "search.initVault",
    "search.clear",
    "search.disable",
    "search.enable",
    "rag.chat",
  ] {
    assert!(!compatibility.contains(action), "optional Search compatibility call leaked: {action}");
    assert!(!api_contracts.contains(action), "optional Search core API contract leaked: {action}");
  }
}

#[test]
fn extracted_sync_runtime_is_physically_absent_from_core() {
  let root = repo_root();
  let lib_min = read_text("Elephant/backend/tauri/src/lib_min.rs");
  let core_commands = read_text("Elephant/backend/tauri/src/core_commands.rs");
  let compatibility = read_text("Elephant/frontend/app/services/elephantnoteClient/compatibilityCalls.js");
  let api_contracts = read_text("Elephant/shared/apiContracts.js");
  let vault_mod = read_text("Elephant/backend/tauri/src/vault/mod.rs");
  let cargo = read_text("Elephant/backend/tauri/Cargo.toml");
  let removed = [
    "Elephant/backend/tauri/src/sync_commands.rs",
    "Elephant/backend/tauri/src/sync_contract_tests.rs",
    "Elephant/backend/tauri/src/sync/mod.rs",
    "Elephant/backend/tauri/src/vault/sync.rs",
    "Elephant/backend/tauri/src/vault/sync_iroh/network.rs",
  ];

  for path in removed {
    assert!(!root.join(path).exists(), "legacy core Sync path must stay absent: {path}");
  }
  assert!(!lib_min.contains("mod sync_commands;"));
  assert!(!lib_min.contains("pub mod sync;"));
  assert!(!lib_min.contains("IrohSyncState"));
  assert!(!lib_min.contains("sync_commands::iroh_sync_"));
  assert!(!lib_min.contains("tauri_extra_commands::tauri_sync_plan"));
  assert!(!core_commands.contains("tauri_sync_plan"));
  assert!(!core_commands.contains("crate::vault::sync"));
  for action in ["sync.status", "sync.plan", "sync.enqueue", "sync.run"] {
    assert!(!compatibility.contains(action), "optional Sync compatibility call leaked: {action}");
    assert!(!api_contracts.contains(action), "optional Sync core API contract leaked: {action}");
  }
  assert!(!vault_mod.contains("pub mod sync;"));
  assert!(!cargo.contains("iroh ="));
  assert!(!cargo.contains("iroh-mdns-address-lookup"));
}
