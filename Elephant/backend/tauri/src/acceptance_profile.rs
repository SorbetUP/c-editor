use std::path::PathBuf;

use tauri::{AppHandle, Manager};

const ACCEPTANCE_PORT_ENV: &str = "ELEPHANT_ACCEPTANCE_TAURI_PORT";
const PROFILE_ROOT_ENV: &str = "ELEPHANT_ACCEPTANCE_PROFILE_DIR";

fn profile_root_from_values(acceptance_port: Option<&str>, profile_root: Option<&str>) -> Result<Option<PathBuf>, String> {
  if acceptance_port.is_none() {
    return Ok(None);
  }
  let Some(profile_root) = profile_root.filter(|value| !value.is_empty()) else {
    return Ok(None);
  };
  let path = PathBuf::from(profile_root);
  if !path.is_absolute() {
    return Err(format!("{PROFILE_ROOT_ENV} must be an absolute path"));
  }
  if path.components().any(|component| matches!(component, std::path::Component::ParentDir)) {
    return Err(format!("{PROFILE_ROOT_ENV} must not contain parent-directory traversal"));
  }
  Ok(Some(path))
}

fn acceptance_profile_root() -> Result<Option<PathBuf>, tauri::Error> {
  profile_root_from_values(
    std::env::var(ACCEPTANCE_PORT_ENV).ok().as_deref(),
    std::env::var(PROFILE_ROOT_ENV).ok().as_deref(),
  )
  .map_err(|error| tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, error)))
}

pub fn app_config_dir(app: &AppHandle) -> Result<PathBuf, tauri::Error> {
  match acceptance_profile_root()? {
    Some(root) => Ok(root.join("config")),
    None => app.path().app_config_dir(),
  }
}

pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf, tauri::Error> {
  match acceptance_profile_root()? {
    Some(root) => Ok(root.join("user-data")),
    None => app.path().app_data_dir(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn acceptance_profile_override_requires_the_existing_acceptance_contract() {
    assert_eq!(profile_root_from_values(None, Some("/tmp/profile")).unwrap(), None);
    assert_eq!(profile_root_from_values(Some("0"), Some("/tmp/profile")).unwrap(), Some(PathBuf::from("/tmp/profile")));
  }

  #[test]
  fn acceptance_profile_override_rejects_non_absolute_or_traversing_paths() {
    assert!(profile_root_from_values(Some("0"), Some("relative/profile")).is_err());
    assert!(profile_root_from_values(Some("0"), Some("/tmp/profile/../other")).is_err());
  }
}
