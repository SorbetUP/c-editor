use std::path::{Path, PathBuf};

pub const HIDDEN_ROOT: &str = ".elephantnote";
pub const HIDDEN_ASSETS_ROOT: &str = ".assets";
pub const HIDDEN_CONFIG_ROOT: &str = ".config";
pub const CONFIG_DIR: &str = "config";
pub const WIKI_DIR: &str = "wiki";
pub const MODELS_DIR: &str = "models";
pub const SYNC_DIR: &str = "sync";
pub const INDEX_DIR: &str = "index";
pub const CACHE_DIR: &str = "cache";
pub const STATE_DIR: &str = "state";
pub const TRASH_DIR: &str = "trash";
pub const ADDONS_DIR: &str = "addons";
pub const ASSETS_DIR: &str = HIDDEN_ASSETS_ROOT;

pub const WORKSPACE_FILE: &str = "workspace.json";
pub const VAULT_FILE: &str = "vault.json";
pub const CALENDAR_FILE: &str = "calendar.json";
pub const SOURCES_FILE: &str = "sources.json";
pub const WIKI_FILE: &str = "wiki.json";
pub const MODELS_FILE: &str = "models.json";
pub const SYNC_FILE: &str = "sync.json";
pub const INDEX_FILE: &str = "index.json";

pub fn hidden_root(vault_root: impl AsRef<Path>) -> PathBuf {
  vault_root.as_ref().join(HIDDEN_ROOT)
}

pub fn hidden_config_root(vault_root: impl AsRef<Path>) -> PathBuf {
  vault_root.as_ref().join(HIDDEN_CONFIG_ROOT)
}

pub fn hidden_dir(vault_root: impl AsRef<Path>, name: &str) -> PathBuf {
  hidden_root(vault_root).join(name)
}

pub fn config_file(vault_root: impl AsRef<Path>, file: &str) -> PathBuf {
  hidden_dir(vault_root, CONFIG_DIR).join(file)
}

pub fn portable_config_file(vault_root: impl AsRef<Path>, category: &str, file: &str) -> PathBuf {
  hidden_config_root(vault_root).join(category).join(file)
}

pub fn wiki_file(vault_root: impl AsRef<Path>, file: &str) -> PathBuf {
  hidden_dir(vault_root, WIKI_DIR).join(file)
}

pub fn models_file(vault_root: impl AsRef<Path>, file: &str) -> PathBuf {
  hidden_dir(vault_root, MODELS_DIR).join(file)
}

pub fn sync_file(vault_root: impl AsRef<Path>, file: &str) -> PathBuf {
  hidden_dir(vault_root, SYNC_DIR).join(file)
}

pub fn index_file(vault_root: impl AsRef<Path>, file: &str) -> PathBuf {
  hidden_dir(vault_root, INDEX_DIR).join(file)
}

pub fn addons_dir(vault_root: impl AsRef<Path>) -> PathBuf {
  hidden_dir(vault_root, ADDONS_DIR)
}

pub fn assets_dir(vault_root: impl AsRef<Path>) -> PathBuf {
  vault_root.as_ref().join(HIDDEN_ASSETS_ROOT)
}

pub fn required_hidden_dirs() -> [&'static str; 10] {
  [
    HIDDEN_CONFIG_ROOT,
    HIDDEN_ASSETS_ROOT,
    CONFIG_DIR,
    MODELS_DIR,
    SYNC_DIR,
    INDEX_DIR,
    CACHE_DIR,
    STATE_DIR,
    TRASH_DIR,
    ADDONS_DIR,
  ]
}

pub fn is_hidden_vault_path(relative_path: &str) -> bool {
  relative_path
    .replace('\\', "/")
    .split('/')
    .next()
    .is_some_and(|first| first == HIDDEN_ROOT || first == HIDDEN_ASSETS_ROOT || first == HIDDEN_CONFIG_ROOT || first.starts_with('.'))
}

pub fn is_visible_vault_path(relative_path: &str) -> bool {
  !is_hidden_vault_path(relative_path)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn builds_hidden_root_path() {
    assert!(hidden_root("vault").ends_with("vault/.elephantnote"));
  }

  #[test]
  fn builds_canonical_metadata_paths() {
    assert!(config_file("vault", WORKSPACE_FILE).ends_with("vault/.elephantnote/config/workspace.json"));
    assert!(portable_config_file("vault", "provider", "provider.json").ends_with("vault/.config/provider/provider.json"));
    assert!(wiki_file("vault", WIKI_FILE).ends_with("vault/.elephantnote/wiki/wiki.json"));
    assert!(models_file("vault", MODELS_FILE).ends_with("vault/.elephantnote/models/models.json"));
    assert!(sync_file("vault", SYNC_FILE).ends_with("vault/.elephantnote/sync/sync.json"));
    assert!(index_file("vault", INDEX_FILE).ends_with("vault/.elephantnote/index/index.json"));
    assert!(addons_dir("vault").ends_with("vault/.elephantnote/addons"));
    assert!(assets_dir("vault").ends_with("vault/.assets"));
  }

  #[test]
  fn exposes_required_hidden_dirs_without_wiki() {
    let dirs = required_hidden_dirs();
    assert!(dirs.contains(&HIDDEN_CONFIG_ROOT));
    assert!(dirs.contains(&HIDDEN_ASSETS_ROOT));
    assert!(dirs.contains(&CONFIG_DIR));
    assert!(!dirs.contains(&WIKI_DIR));
    assert!(dirs.contains(&MODELS_DIR));
    assert!(dirs.contains(&SYNC_DIR));
    assert!(dirs.contains(&ADDONS_DIR));
  }

  #[test]
  fn distinguishes_hidden_and_visible_paths() {
    assert!(is_hidden_vault_path(".elephantnote/config/workspace.json"));
    assert!(is_hidden_vault_path(".elephantnote/addons/registry.json"));
    assert!(is_hidden_vault_path(".config/provider/provider.json"));
    assert!(is_hidden_vault_path(".assets/excalidraw.png"));
    assert!(is_hidden_vault_path(".git/config"));
    assert!(is_visible_vault_path("Projects/note.md"));
  }
}
