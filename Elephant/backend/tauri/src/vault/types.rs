use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultConfig {
  pub vaults: Vec<VaultDescriptor>,
  pub active_vault_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultDescriptor {
  pub id: String,
  pub name: String,
  pub path: String,
  pub icon: String,
  pub last_opened_at: String,
}

pub fn slug_id(value: &str) -> String {
  let mut out = String::new();
  let mut last_dash = false;

  for ch in value.trim().to_lowercase().chars() {
    if ch.is_ascii_alphanumeric() {
      out.push(ch);
      last_dash = false;
    } else if !last_dash {
      out.push('-');
      last_dash = true;
    }
  }

  let out = out.trim_matches('-').to_string();
  if out.is_empty() { "vault".to_string() } else { out }
}

pub fn next_vault_id(existing: &[VaultDescriptor], name: &str) -> String {
  let base = slug_id(name);
  if !existing.iter().any(|vault| vault.id == base) {
    return base;
  }

  let mut suffix = 2;
  loop {
    let candidate = format!("{}-{}", base, suffix);
    if !existing.iter().any(|vault| vault.id == candidate) {
      return candidate;
    }
    suffix += 1;
  }
}

pub fn active_vault(config: &VaultConfig) -> Option<VaultDescriptor> {
  let active_id = config.active_vault_id.as_ref()?;
  config.vaults.iter().find(|vault| &vault.id == active_id).cloned()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn creates_slug_ids() {
    assert_eq!(slug_id("My Vault"), "my-vault");
    assert_eq!(slug_id("!!!"), "vault");
  }

  #[test]
  fn creates_unique_ids() {
    let existing = vec![VaultDescriptor {
      id: "work".to_string(),
      name: "Work".to_string(),
      path: "Work".to_string(),
      icon: String::new(),
      last_opened_at: "0".to_string(),
    }];
    assert_eq!(next_vault_id(&existing, "Work"), "work-2");
  }

  #[test]
  fn resolves_active_vault() {
    let config = VaultConfig {
      vaults: vec![VaultDescriptor {
        id: "a".to_string(),
        name: "A".to_string(),
        path: "A".to_string(),
        icon: String::new(),
        last_opened_at: "0".to_string(),
      }],
      active_vault_id: Some("a".to_string()),
    };
    assert_eq!(active_vault(&config).unwrap().name, "A");
  }
}
