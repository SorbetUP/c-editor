//! Native adapter for the persisted addon registry.
//!
//! The registry format is the production Tauri addon contract.  Freya owns
//! the settings surface, but does not invent an in-memory catalogue or claim
//! that a JavaScript worker is running when it is not.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::navigation_contract::WorkspaceView;
use crate::vault_layout;

type Result<T> = std::result::Result<T, String>;

pub(crate) const ADDON_API_VERSION: u32 = 1;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AddonManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default = "default_api_version")]
    pub api_version: u32,
    #[serde(default)]
    pub min_app_version: String,
    #[serde(default)]
    pub runtime: AddonRuntime,
    #[serde(default)]
    pub contributes: Value,
    #[serde(default)]
    pub activation_events: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AddonRuntime {
    #[serde(rename = "type")]
    pub kind: String,
    pub entry: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstalledAddon {
    pub manifest: AddonManifest,
    pub enabled: bool,
    #[serde(default)]
    pub package_hash: String,
    #[serde(default)]
    pub installed_at: String,
    #[serde(default)]
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeView {
    pub addon_id: String,
    pub title: String,
    pub view: WorkspaceView,
    pub icon: NativeViewIcon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeViewIcon {
    Book,
    Calendar,
    Chat,
    Dashboard,
    Graph,
    Models,
    Sync,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Registry {
    #[serde(default)]
    version: u32,
    #[serde(default)]
    addons: BTreeMap<String, InstalledAddon>,
}

pub fn list(root: &Path) -> Result<Vec<InstalledAddon>> {
    Ok(read(root)?.addons.into_values().collect())
}

pub fn install(root: &Path, package_path: &Path) -> Result<InstalledAddon> {
    let existing = read(root)?;
    let package = crate::addon_packages::install(root, package_path)?;
    let enabled = existing
        .addons
        .get(&package.manifest.id)
        .map(|addon| addon.enabled)
        .unwrap_or(false);
    let record = InstalledAddon {
        manifest: package.manifest,
        enabled,
        package_hash: package.package_hash,
        installed_at: now(),
        source: "external".to_owned(),
    };
    let mut registry = existing;
    registry
        .addons
        .insert(record.manifest.id.clone(), record.clone());
    if let Err(error) = write(root, &registry) {
        let _ = fs::remove_dir_all(&package.target);
        if let Some(backup) = package.backup {
            let _ = fs::rename(backup, package.target);
        }
        return Err(error);
    }
    if let Some(backup) = package.backup {
        let _ = fs::remove_dir_all(backup);
    }
    eprintln!(
        "[freya][addons] action=install-complete id={} hash={}",
        record.manifest.id, record.package_hash
    );
    Ok(record)
}

/// Returns only addon views that have a real native Freya implementation.
/// Unknown JavaScript contributions stay out of the rail until their runtime
/// bridge exists; showing them as clickable chrome would be misleading.
pub fn native_views(root: &Path) -> Result<Vec<NativeView>> {
    let mut views = Vec::new();
    for addon in list(root)? {
        if !addon.enabled {
            continue;
        }
        let Some(view) = native_view_for(&addon) else {
            continue;
        };
        views.push(view);
    }
    Ok(views)
}

fn native_view_for(addon: &InstalledAddon) -> Option<NativeView> {
    let (title, view, icon) = match addon.manifest.id.as_str() {
        "elephant.graph" => ("Graph", WorkspaceView::Graph, NativeViewIcon::Graph),
        "elephant.wiki" => ("Wiki", WorkspaceView::Wiki, NativeViewIcon::Book),
        "elephant.calendar" => (
            "Calendar",
            WorkspaceView::Calendar,
            NativeViewIcon::Calendar,
        ),
        "elephant.ai-chat" => ("Chat", WorkspaceView::Chat, NativeViewIcon::Chat),
        "elephant.open-models" => ("Models", WorkspaceView::Models, NativeViewIcon::Models),
        "elephant.sync" => ("Sync", WorkspaceView::Sync, NativeViewIcon::Sync),
        "elephant.dashboard" => (
            "Dashboard",
            WorkspaceView::Dashboard,
            NativeViewIcon::Dashboard,
        ),
        _ => return None,
    };
    Some(NativeView {
        addon_id: addon.manifest.id.clone(),
        title: title.to_owned(),
        view,
        icon,
    })
}

pub(crate) fn has_native_view(addon_id: &str) -> bool {
    matches!(
        addon_id,
        "elephant.graph"
            | "elephant.wiki"
            | "elephant.calendar"
            | "elephant.ai-chat"
            | "elephant.open-models"
            | "elephant.sync"
            | "elephant.dashboard"
    )
}

pub fn set_enabled(root: &Path, addon_id: &str, enabled: bool) -> Result<InstalledAddon> {
    let mut registry = read(root)?;
    let addon = registry
        .addons
        .get_mut(addon_id)
        .ok_or_else(|| format!("Unknown addon: {addon_id}"))?;
    addon.enabled = enabled;
    let result = addon.clone();
    write(root, &registry)?;
    eprintln!("[freya][addons] action:set-enabled id={addon_id} enabled={enabled}");
    Ok(result)
}

pub fn uninstall(root: &Path, addon_id: &str) -> Result<()> {
    let mut registry = read(root)?;
    if registry.addons.remove(addon_id).is_none() {
        return Err(format!("Unknown addon: {addon_id}"));
    }
    write(root, &registry)?;
    let package = vault_layout::addons_dir(root)
        .join("packages")
        .join(addon_id);
    if package.exists() {
        fs::remove_dir_all(&package)
            .map_err(|error| format!("Remove addon package {}: {error}", package.display()))?;
    }
    let data = vault_layout::addons_dir(root).join("data").join(addon_id);
    if data.exists() {
        fs::remove_dir_all(&data)
            .map_err(|error| format!("Remove addon data {}: {error}", data.display()))?;
    }
    eprintln!("[freya][addons] action:uninstall-complete id={addon_id}");
    Ok(())
}

fn default_api_version() -> u32 {
    ADDON_API_VERSION
}

fn now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

fn read(root: &Path) -> Result<Registry> {
    let path = registry_path(root);
    if !path.exists() {
        return Ok(Registry {
            version: 1,
            ..Registry::default()
        });
    }
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Read addon registry {}: {error}", path.display()))?;
    serde_json::from_str(&raw)
        .map_err(|error| format!("Parse addon registry {}: {error}", path.display()))
}

fn write(root: &Path, registry: &Registry) -> Result<()> {
    let path = registry_path(root);
    let parent = path
        .parent()
        .ok_or_else(|| "Addon registry has no parent directory".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "Create addon registry directory {}: {error}",
            parent.display()
        )
    })?;
    let temporary = path.with_extension("json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(registry).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("Write addon registry {}: {error}", temporary.display()))?;
    fs::rename(&temporary, &path)
        .map_err(|error| format!("Replace addon registry {}: {error}", path.display()))
}

fn registry_path(root: &Path) -> PathBuf {
    vault_layout::addons_dir(root).join("registry.json")
}
