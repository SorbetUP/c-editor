use elephant_draw::LibraryFile;
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

const ELEPHANT_DIR: &str = ".elephantnote";
const LIBRARY_FILE: &str = "drawing-library.excalidrawlib";

pub(super) fn load(root: &Path) -> Result<LibraryFile, String> {
    let path = library_path(root, false)?;
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(LibraryFile::empty("elephant"));
        }
        Err(error) => {
            return Err(format!(
                "Drawing library unavailable at {}: {error}",
                path.display()
            ));
        }
    };
    let library: LibraryFile = serde_json::from_str(&raw)
        .map_err(|error| format!("Drawing library invalid at {}: {error}", path.display()))?;
    if library.file_type != "excalidrawlib" || library.version != 2 {
        return Err(format!(
            "Drawing library invalid at {}: expected excalidrawlib v2",
            path.display()
        ));
    }
    Ok(library)
}

pub(super) fn save(root: &Path, library: &LibraryFile) -> Result<(), String> {
    let path = library_path(root, true)?;
    let raw = serde_json::to_vec_pretty(library).map_err(|error| error.to_string())?;
    let temporary = path.with_extension("excalidrawlib.tmp");
    fs::write(&temporary, raw).map_err(|error| {
        format!(
            "Unable to persist drawing library {}: {error}",
            temporary.display()
        )
    })?;
    fs::rename(&temporary, &path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("Unable to replace drawing library {}: {error}", path.display())
    })
}

fn library_path(root: &Path, create: bool) -> Result<PathBuf, String> {
    let root = fs::canonicalize(root).map_err(|error| error.to_string())?;
    let directory = root.join(ELEPHANT_DIR);
    if fs::symlink_metadata(&directory)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(format!(
            "Refusing symlinked drawing library directory: {}",
            directory.display()
        ));
    }
    if create {
        fs::create_dir_all(&directory).map_err(|error| {
            format!(
                "Unable to create drawing library directory {}: {error}",
                directory.display()
            )
        })?;
    }
    Ok(directory.join(LIBRARY_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use elephant_draw::LibraryItemStatus;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-draw-library-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn absent_library_loads_empty_and_roundtrips_excalidrawlib_v2() {
        let root = root();
        let mut library = load(&root).unwrap();
        assert_eq!(library.file_type, "excalidrawlib");
        assert_eq!(library.version, 2);
        library.library_items.push(elephant_draw::LibraryItem {
            id: "one".to_owned(),
            status: LibraryItemStatus::Unpublished,
            created: 42,
            elements: Vec::new(),
            name: Some("One".to_owned()),
            files: serde_json::json!({}),
        });
        save(&root, &library).unwrap();
        assert_eq!(load(&root).unwrap(), library);
        let _ = fs::remove_dir_all(root);
    }
}
