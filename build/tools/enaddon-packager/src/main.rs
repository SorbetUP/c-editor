use serde_json::Value;
use std::{
  env,
  fs,
  io::{Read, Write},
  path::{Component, Path, PathBuf},
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

type R<T> = Result<T, String>;

fn collect_files(root: &Path, current: &Path, output: &mut Vec<PathBuf>) -> R<()> {
  let mut entries = fs::read_dir(current)
    .map_err(|error| format!("Failed to read {}: {error}", current.display()))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| error.to_string())?;
  entries.sort_by_key(|entry| entry.file_name());

  for entry in entries {
    let path = entry.path();
    let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
      return Err(format!("Symbolic links are not allowed in addon packages: {}", path.display()));
    }
    if metadata.is_dir() {
      collect_files(root, &path, output)?;
    } else if metadata.is_file() {
      let relative = path
        .strip_prefix(root)
        .map_err(|error| error.to_string())?
        .to_path_buf();
      output.push(relative);
    }
  }
  Ok(())
}

fn safe_runtime_entry(value: &str) -> R<PathBuf> {
  let path = Path::new(value);
  if value.trim().is_empty() || path.is_absolute() {
    return Err("Addon manifest runtime.entry must be a safe relative path".to_string());
  }
  let mut normalized = PathBuf::new();
  for component in path.components() {
    match component {
      Component::Normal(part) => normalized.push(part),
      Component::CurDir => {}
      Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
        return Err("Addon manifest runtime.entry must stay inside the package".to_string());
      }
    }
  }
  if normalized.as_os_str().is_empty() {
    return Err("Addon manifest runtime.entry must not be empty".to_string());
  }
  Ok(normalized)
}

fn declared_runtime_entry(source: &Path) -> R<PathBuf> {
  let manifest_path = source.join("manifest.json");
  if !manifest_path.is_file() {
    return Err("Addon staging directory must contain manifest.json".to_string());
  }
  let manifest_bytes = fs::read(&manifest_path)
    .map_err(|error| format!("Failed to read {}: {error}", manifest_path.display()))?;
  let manifest: Value = serde_json::from_slice(&manifest_bytes)
    .map_err(|error| format!("Invalid addon manifest: {error}"))?;
  let entry = manifest
    .pointer("/runtime/entry")
    .and_then(Value::as_str)
    .ok_or_else(|| "Addon manifest must declare runtime.entry".to_string())?;
  safe_runtime_entry(entry)
}

fn package(source: &Path, output: &Path) -> R<(u64, String)> {
  if !source.is_dir() {
    return Err(format!("Addon staging directory does not exist: {}", source.display()));
  }
  let runtime_entry = declared_runtime_entry(source)?;
  if !source.join(&runtime_entry).is_file() {
    return Err(format!(
      "Addon staging directory does not contain manifest runtime entry {}",
      runtime_entry.display()
    ));
  }
  if let Some(parent) = output.parent() {
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }

  let mut files = Vec::new();
  collect_files(source, source, &mut files)?;
  files.sort();

  let file = fs::File::create(output).map_err(|error| error.to_string())?;
  let mut archive = ZipWriter::new(file);
  let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .unix_permissions(0o644)
    .last_modified_time(zip::DateTime::default());

  for relative in files {
    let name = relative.to_string_lossy().replace('\\', "/");
    let path = source.join(&relative);
    let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
    let file_options = if name.starts_with("native/") {
      options.unix_permissions(0o755)
    } else {
      options
    };
    archive.start_file(name, file_options).map_err(|error| error.to_string())?;
    let mut input = fs::File::open(&path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    input.read_to_end(&mut bytes).map_err(|error| error.to_string())?;
    archive.write_all(&bytes).map_err(|error| error.to_string())?;
  }
  archive.finish().map_err(|error| error.to_string())?;

  let bytes = fs::read(output).map_err(|error| error.to_string())?;
  Ok((bytes.len() as u64, blake3::hash(&bytes).to_hex().to_string()))
}

fn main() {
  let args = env::args().skip(1).collect::<Vec<_>>();
  if args.len() != 2 {
    eprintln!("usage: elephant-enaddon-packager <staging-dir> <output.enaddon>");
    std::process::exit(2);
  }

  let source = PathBuf::from(&args[0]);
  let output = PathBuf::from(&args[1]);
  match package(&source, &output) {
    Ok((bytes, blake3)) => {
      println!(
        "{}",
        serde_json::json!({
          "path": output,
          "bytes": bytes,
          "blake3": blake3
        })
      );
    }
    Err(error) => {
      eprintln!("{error}");
      std::process::exit(1);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_non_default_runtime_entries() {
    assert_eq!(safe_runtime_entry("main.mobile.js").unwrap(), PathBuf::from("main.mobile.js"));
    assert_eq!(safe_runtime_entry("runtime/main.js").unwrap(), PathBuf::from("runtime/main.js"));
  }

  #[test]
  fn rejects_runtime_entry_traversal() {
    assert!(safe_runtime_entry("../main.js").is_err());
    assert!(safe_runtime_entry("/main.js").is_err());
    assert!(safe_runtime_entry("").is_err());
  }
}
