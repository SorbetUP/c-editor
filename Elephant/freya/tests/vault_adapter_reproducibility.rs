use elephant_freya::vault_adapter::VaultAdapter;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new(label: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-vault-repro-{label}-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create fixture vault");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn descriptor_and_path_boundaries_match_the_committed_backend() {
    let fixture = FixtureVault::new("boundaries");
    let adapter = VaultAdapter::open(fixture.path()).expect("open committed descriptor boundary");
    let descriptor = serde_json::to_value(adapter.descriptor()).expect("serialize descriptor");

    assert_eq!(
        descriptor["path"],
        fs::canonicalize(fixture.path())
            .expect("canonical fixture")
            .to_string_lossy()
            .as_ref()
    );
    assert!(adapter.list_directory("../outside").is_err());
    assert!(adapter.list_directory("/absolute").is_err());
    assert!(adapter
        .create_note(
            Some("notes/../../outside".to_string()),
            Some("Escape.md".to_string()),
            Some("Escape".to_string()),
        )
        .is_err());
    assert!(!fixture.path().parent().unwrap().join("Escape.md").exists());
}

#[test]
fn rebuild_uses_the_committed_fts_index_and_removes_stale_documents() {
    let fixture = FixtureVault::new("fts");
    fs::write(
        fixture.path().join("Dense.md"),
        "# Dense\n\nneedle needle needle needle\n",
    )
    .expect("write dense note");
    fs::write(fixture.path().join("Sparse.md"), "# Sparse\n\nneedle\n").expect("write sparse note");
    fs::create_dir_all(fixture.path().join("Nested")).expect("create nested directory");
    fs::write(
        fixture.path().join("Nested/Deep.md"),
        "# Deep\n\nneedle elsewhere\n",
    )
    .expect("write nested note");
    fs::write(fixture.path().join(".Hidden.md"), "# Hidden\n\nneedle\n")
        .expect("write hidden note");
    let adapter = VaultAdapter::open(fixture.path()).expect("open adapter");

    let first = adapter
        .rebuild_search_index()
        .expect("rebuild committed FTS index");
    assert_eq!(first.status, "complete");
    assert_eq!(first.scanned, 3);
    assert!(first.failed.is_empty());
    assert!(fixture
        .path()
        .join(".elephantnote/index/notes.sqlite")
        .is_file());
    let hits = adapter.search_index("needle", 20).expect("search real FTS");
    assert_eq!(hits.len(), 3);
    assert_eq!(hits[0].path, "Dense.md");
    assert!(hits[0].score < hits[1].score);
    assert!(hits.iter().any(|hit| hit.path == "Nested/Deep.md"));
    assert!(!hits.iter().any(|hit| hit.path == ".Hidden.md"));

    fs::remove_file(fixture.path().join("Dense.md")).expect("remove indexed source");
    let second = adapter
        .rebuild_search_index()
        .expect("rebuild after source removal");
    assert_eq!(second.removed, 1);
    let hits = adapter
        .search_index("needle", 20)
        .expect("search rebuilt FTS");
    assert_eq!(hits.len(), 2);
    assert!(hits.iter().all(|hit| hit.path != "Dense.md"));
}

#[test]
fn trash_boundary_moves_lists_restores_and_empties_real_files() {
    let fixture = FixtureVault::new("trash");
    let adapter = VaultAdapter::open(fixture.path()).expect("open adapter");
    let note = adapter
        .create_note(
            None,
            Some("Recover.md".to_string()),
            Some("Recover".to_string()),
        )
        .expect("create note");

    let deleted = adapter.delete(&note.path).expect("move note to trash");
    assert!(!fixture.path().join("Recover.md").exists());
    assert!(fixture.path().join(&deleted.trash_path).is_dir());
    let entries = adapter.list_trash().expect("list real trash manifests");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].original_path, "Recover.md");

    let restored = adapter
        .restore_trash(&entries[0].trash_path)
        .expect("restore note");
    assert_eq!(restored.path, "Recover.md");
    assert!(fixture.path().join("Recover.md").is_file());
    assert!(adapter.list_trash().expect("list empty trash").is_empty());

    adapter.delete("Recover.md").expect("trash note again");
    let emptied = adapter.empty_trash().expect("empty trash");
    assert_eq!(emptied.count, 1);
    assert!(adapter.list_trash().expect("list emptied trash").is_empty());
}

#[test]
fn corrupt_trash_manifest_is_an_explicit_error() {
    let fixture = FixtureVault::new("corrupt-trash");
    let adapter = VaultAdapter::open(fixture.path()).expect("open adapter");
    adapter
        .create_note(
            None,
            Some("Broken.md".to_string()),
            Some("Broken".to_string()),
        )
        .expect("create note");
    let deleted = adapter.delete("Broken.md").expect("trash note");
    fs::write(
        fixture
            .path()
            .join(deleted.trash_path)
            .join("manifest.json"),
        "{not-json",
    )
    .expect("corrupt manifest");

    let error = adapter
        .list_trash()
        .expect_err("corruption must be visible");
    assert!(error.to_string().contains("Invalid trash manifest"));
}
