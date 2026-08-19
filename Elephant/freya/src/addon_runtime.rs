//! Freya-owned lifecycle for installed external JavaScript add-ons.

use crate::{
    addon_adapter::{self, InstalledAddon},
    addon_host,
    addon_worker::{AddonWorker, AddonWorkerConfig},
    resource_locator,
    vault_layout,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

pub struct RuntimeManager {
    workers: BTreeMap<String, AddonWorker>,
    vault_root: Option<PathBuf>,
    node_executable: PathBuf,
    next_request_id: u64,
}

impl Default for RuntimeManager {
    fn default() -> Self {
        // Development builds may deliberately fall back to system Node through
        // ResourceLocator. Release lookup is bundle-first and fails visibly if
        // a runtime is missing.
        let node_executable = resource_locator::node_runtime()
            .unwrap_or_else(|error| PathBuf::from(format!("__elephant_missing_node__:{error}")));
        Self::new(node_executable)
    }
}

impl RuntimeManager {
    pub fn new(node_executable: impl Into<PathBuf>) -> Self {
        Self {
            workers: BTreeMap::new(),
            vault_root: None,
            node_executable: node_executable.into(),
            next_request_id: 1,
        }
    }

    pub fn set_enabled(
        &mut self,
        root: &Path,
        addon_id: &str,
        enabled: bool,
    ) -> Result<(), String> {
        let request_id = self.request_id();
        let started = Instant::now();
        self.select_vault(root, request_id)?;
        let addon = installed_addon(root, addon_id)?;
        log(request_id, addon_id, "set-enabled:start", None);

        let result = if !uses_worker(&addon) {
            addon_adapter::set_enabled(root, addon_id, enabled).map(|_| ())
        } else if enabled {
            self.enable_worker(root, &addon, request_id).and_then(|()| {
                addon_adapter::set_enabled(root, addon_id, true)
                    .map(|_| ())
                    .map_err(|error| {
                        let _ = self.stop_worker(addon_id, request_id);
                        error
                    })
            })
        } else {
            let shutdown = self.stop_worker(addon_id, request_id);
            let persisted = addon_adapter::set_enabled(root, addon_id, false).map(|_| ());
            combine(shutdown, persisted)
        };

        log_result(request_id, addon_id, "set-enabled", started, &result);
        result
    }

    pub fn uninstall(&mut self, root: &Path, addon_id: &str) -> Result<(), String> {
        let request_id = self.request_id();
        let started = Instant::now();
        self.select_vault(root, request_id)?;
        log(request_id, addon_id, "uninstall:start", None);
        let shutdown = self.stop_worker(addon_id, request_id);
        let removed = addon_adapter::uninstall(root, addon_id);
        let result = combine(shutdown, removed);
        log_result(request_id, addon_id, "uninstall", started, &result);
        result
    }

    pub fn reconcile_enabled(&mut self, root: &Path) -> Result<(), String> {
        let request_id = self.request_id();
        let started = Instant::now();
        self.select_vault(root, request_id)?;
        let addons = addon_adapter::list(root)?;
        let desired = addons
            .iter()
            .filter(|addon| addon.enabled && uses_worker(addon))
            .map(|addon| addon.manifest.id.clone())
            .collect::<BTreeSet<_>>();
        let mut errors = Vec::new();

        for addon_id in self.workers.keys().cloned().collect::<Vec<_>>() {
            if !desired.contains(&addon_id) {
                if let Err(error) = self.stop_worker(&addon_id, request_id) {
                    errors.push(error);
                }
            }
        }
        for addon in addons
            .iter()
            .filter(|addon| desired.contains(&addon.manifest.id))
        {
            if let Err(error) = self.enable_worker(root, addon, request_id) {
                let rollback = addon_adapter::set_enabled(root, &addon.manifest.id, false)
                    .map(|_| ())
                    .map_err(|rollback| format!("{error}; disable rollback failed: {rollback}"));
                errors.push(rollback.err().unwrap_or(error));
            }
        }

        let result = errors_result(errors);
        log_result(request_id, "registry", "reconcile", started, &result);
        result
    }

    pub fn active_pid(&self, addon_id: &str) -> Option<u32> {
        self.workers.get(addon_id).map(AddonWorker::pid)
    }

    pub fn is_active(&mut self, addon_id: &str) -> Result<bool, String> {
        let Some(worker) = self.workers.get_mut(addon_id) else {
            return Ok(false);
        };
        let running = worker.is_running().map_err(|error| error.to_string())?;
        if !running {
            self.workers.remove(addon_id);
        }
        Ok(running)
    }

    fn enable_worker(
        &mut self,
        root: &Path,
        addon: &InstalledAddon,
        request_id: u64,
    ) -> Result<(), String> {
        if self.is_active(&addon.manifest.id)? {
            return Ok(());
        }
        let entry = runtime_entry(root, addon)?;
        let manifest = serde_json::to_value(&addon.manifest).map_err(|error| error.to_string())?;
        let mut worker = AddonWorker::spawn(AddonWorkerConfig::new(
            &self.node_executable,
            entry,
            &addon.manifest.id,
            manifest,
        ))
        .map_err(|error| visible_error("Start", &addon.manifest.id, &error.to_string()))?;
        let root = root.to_path_buf();
        let addon_for_broker = addon.clone();
        let mut broker = move |method: &str, params: &Value| {
            addon_host::handle(&root, &addon_for_broker, method, params)
        };
        if let Err(error) = worker.activate_with_broker(&mut broker) {
            let message = visible_error("Activate", &addon.manifest.id, &error.to_string());
            log(
                request_id,
                &addon.manifest.id,
                "activate:error",
                Some(&message),
            );
            let _ = worker.deactivate_with_broker(&mut broker);
            return Err(message);
        }
        log(
            request_id,
            &addon.manifest.id,
            "activate:done",
            Some(&format!("pid={}", worker.pid())),
        );
        self.workers.insert(addon.manifest.id.clone(), worker);
        Ok(())
    }

    fn stop_worker(&mut self, addon_id: &str, request_id: u64) -> Result<(), String> {
        let Some(mut worker) = self.workers.remove(addon_id) else {
            return Ok(());
        };
        let pid = worker.pid();
        let root = self.vault_root.clone();
        let addon = root
            .as_deref()
            .and_then(|root| installed_addon(root, addon_id).ok());
        let mut broker = move |method: &str, params: &Value| {
            match (root.as_deref(), addon.as_ref()) {
                (Some(root), Some(addon)) => addon_host::handle(root, addon, method, params),
                _ if method == "app.info" => Ok(serde_json::json!({
                    "name": "ElephantNote",
                    "runtime": "freya",
                    "version": env!("CARGO_PKG_VERSION"),
                    "addonApiVersion": addon_adapter::ADDON_API_VERSION
                })),
                _ => Err(format!("Addon host context is unavailable during shutdown: {method}")),
            }
        };
        let result = worker
            .deactivate_with_broker(&mut broker)
            .map(|_| ())
            .map_err(|error| visible_error("Deactivate", addon_id, &error.to_string()));
        log(
            request_id,
            addon_id,
            if result.is_ok() {
                "deactivate:done"
            } else {
                "deactivate:error"
            },
            Some(&format!("pid={pid}")),
        );
        result
    }

    fn select_vault(&mut self, root: &Path, request_id: u64) -> Result<(), String> {
        let root = fs::canonicalize(root).map_err(|error| format!("Open vault: {error}"))?;
        if self
            .vault_root
            .as_ref()
            .is_some_and(|active| active != &root)
        {
            let ids = self.workers.keys().cloned().collect::<Vec<_>>();
            let mut errors = Vec::new();
            for id in ids {
                if let Err(error) = self.stop_worker(&id, request_id) {
                    errors.push(error);
                }
            }
            errors_result(errors)?;
        }
        self.vault_root = Some(root);
        Ok(())
    }

    fn request_id(&mut self) -> u64 {
        let id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        id
    }
}

impl Drop for RuntimeManager {
    fn drop(&mut self) {
        let ids = self.workers.keys().cloned().collect::<Vec<_>>();
        for id in ids {
            let _ = self.stop_worker(&id, 0);
        }
    }
}

#[derive(Clone)]
pub(crate) struct RuntimeOwner(Arc<Mutex<RuntimeManager>>);

impl Default for RuntimeOwner {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(RuntimeManager::default())))
    }
}

impl RuntimeOwner {
    pub(crate) fn set_enabled(
        &self,
        root: &Path,
        addon_id: &str,
        enabled: bool,
    ) -> Result<(), String> {
        self.lock()?.set_enabled(root, addon_id, enabled)
    }

    pub(crate) fn uninstall(&self, root: &Path, addon_id: &str) -> Result<(), String> {
        self.lock()?.uninstall(root, addon_id)
    }

    pub(crate) fn reconcile_enabled(&self, root: &Path) -> Result<(), String> {
        self.lock()?.reconcile_enabled(root)
    }

    pub(crate) fn is_active(&self, addon_id: &str) -> bool {
        self.lock()
            .and_then(|mut manager| manager.is_active(addon_id))
            .unwrap_or(false)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, RuntimeManager>, String> {
        self.0
            .lock()
            .map_err(|_| "Addon runtime manager lock is poisoned".to_owned())
    }
}

impl fmt::Debug for RuntimeOwner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RuntimeOwner")
    }
}

impl PartialEq for RuntimeOwner {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

fn installed_addon(root: &Path, addon_id: &str) -> Result<InstalledAddon, String> {
    addon_adapter::list(root)?
        .into_iter()
        .find(|addon| addon.manifest.id == addon_id)
        .ok_or_else(|| format!("Unknown addon: {addon_id}"))
}

fn uses_worker(addon: &InstalledAddon) -> bool {
    addon.source == "external"
        && addon.manifest.runtime.kind == "javascript-worker"
        && !addon_adapter::has_native_view(&addon.manifest.id)
}

fn runtime_entry(root: &Path, addon: &InstalledAddon) -> Result<PathBuf, String> {
    let mut relative = PathBuf::new();
    for component in Path::new(&addon.manifest.runtime.entry).components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            _ => return Err("Addon runtime entry must be a safe relative path".to_owned()),
        }
    }
    if relative.extension().and_then(|value| value.to_str()) != Some("js") {
        return Err("Addon runtime entry must be a JavaScript file".to_owned());
    }
    let package = fs::canonicalize(
        vault_layout::addons_dir(root)
            .join("packages")
            .join(&addon.manifest.id),
    )
    .map_err(|error| format!("Open addon package: {error}"))?;
    let entry = fs::canonicalize(package.join(relative))
        .map_err(|error| format!("Open addon runtime entry: {error}"))?;
    if !entry.starts_with(&package) || !entry.is_file() {
        return Err("Addon runtime entry escapes its package directory".to_owned());
    }
    Ok(entry)
}

fn combine(first: Result<(), String>, second: Result<(), String>) -> Result<(), String> {
    match (first, second) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(first), Err(second)) => Err(format!("{first}; {second}")),
    }
}

fn errors_result(errors: Vec<String>) -> Result<(), String> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn visible_error(action: &str, addon_id: &str, error: &str) -> String {
    format!("{action} addon {addon_id}: {}", clean_message(error))
}

fn clean_message(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .take(512)
        .collect::<String>()
}

fn log(request_id: u64, addon_id: &str, action: &str, detail: Option<&str>) {
    let detail = detail.map(clean_log_detail).unwrap_or_default();
    eprintln!(
        "[freya][addons] request={request_id} id={} action={action}{}",
        clean_message(addon_id),
        if detail.is_empty() {
            String::new()
        } else {
            format!(" detail={detail}")
        }
    );
}

fn clean_log_detail(value: &str) -> String {
    clean_message(value)
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if part.contains('/')
                || part.contains('\\')
                || ["token", "secret", "password", "authorization"]
                    .iter()
                    .any(|sensitive| lower.contains(sensitive))
            {
                "<redacted>"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn log_result(
    request_id: u64,
    addon_id: &str,
    action: &str,
    started: Instant,
    result: &Result<(), String>,
) {
    let detail = format!(
        "duration_ms={}{}",
        started.elapsed().as_millis(),
        result
            .as_ref()
            .err()
            .map(|error| format!(" error={error}"))
            .unwrap_or_default()
    );
    log(
        request_id,
        addon_id,
        if result.is_ok() { "complete" } else { "error" },
        Some(&format!("operation={action} {detail}")),
    );
}
