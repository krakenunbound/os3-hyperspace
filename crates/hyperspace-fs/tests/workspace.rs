use std::path::PathBuf;

use hyperspace_core::HyperspaceState;
use tempfile::TempDir;

use hyperspace_fs::JsonWorkspaceStore;

#[test]
fn roundtrip_workspace_json() {
    let temp = TempDir::new().expect("temp dir");
    let path = temp.path().join("workspace.json");
    let store = JsonWorkspaceStore::new(path);

    let original = HyperspaceState::with_demo_content();
    store.save(&original).expect("save");

    let loaded = store.load().expect("load");
    assert_eq!(loaded.dimensions.len(), original.dimensions.len());
    assert_eq!(loaded.active_dimension, original.active_dimension);
    assert_eq!(
        loaded.dimensions[0].objects.len(),
        original.dimensions[0].objects.len()
    );
}

#[test]
fn load_or_default_creates_fresh_state_when_missing() {
    let path = PathBuf::from("definitely-not-a-real-workspace-file.json");
    let store = JsonWorkspaceStore::new(path);
    let state = store.load_or_default();
    assert_eq!(state.dimensions.len(), 2);
}
