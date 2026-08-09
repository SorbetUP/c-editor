use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn write_atomically(path: impl AsRef<Path>, bytes: &[u8]) -> std::io::Result<()> {
  let path = path.as_ref();
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  let tmp: PathBuf = path.with_extension(format!(
    "{}.tmp",
    path.extension().and_then(|e| e.to_str()).unwrap_or("")
  ));
  {
    let mut file = fs::File::create(&tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
  }
  fs::rename(&tmp, path)
}

pub fn write_json_atomically<T: serde::Serialize>(path: impl AsRef<Path>, value: &T) -> std::io::Result<()> {
  let bytes = serde_json::to_vec_pretty(value)
    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
  write_atomically(path, &bytes)
}

pub fn read_json_or<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>, fallback: T) -> T {
  match fs::read(path.as_ref()) {
    Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or(fallback),
    Err(_) => fallback,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde::{Deserialize, Serialize};
  use std::fs;

  #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
  struct Sample {
    name: String,
    n: u32,
  }

  #[test]
  fn atomic_write_roundtrips_json() {
    let dir = std::env::temp_dir().join(format!("elephantnote_test_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("sample.json");
    let value = Sample { name: "elephant".into(), n: 42 };
    write_json_atomically(&path, &value).unwrap();
    let result: Sample = read_json_or(&path, Sample { name: "fallback".into(), n: 0 });
    assert_eq!(result, value);
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn atomic_write_replaces_existing() {
    let dir = std::env::temp_dir().join(format!("elephantnote_test2_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("replace.json");
    write_json_atomically(&path, &Sample { name: "old".into(), n: 1 }).unwrap();
    write_json_atomically(&path, &Sample { name: "new".into(), n: 2 }).unwrap();
    let result: Sample = read_json_or(&path, Sample { name: "fallback".into(), n: 0 });
    assert_eq!(result.name, "new");
    assert_eq!(result.n, 2);
    assert!(!path.with_extension("json.tmp").exists());
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn read_json_or_falls_back_when_missing() {
    let path = Path::new("/nonexistent/elephantnote_test/missing.json");
    let v: Sample = read_json_or(path, Sample { name: "fallback".into(), n: 7 });
    assert_eq!(v.n, 7);
  }
}