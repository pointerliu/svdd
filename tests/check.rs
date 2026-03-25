use std::path::Path;

use tempfile::tempdir;

use svdd::check::{CheckOutcome, ScriptChecker};

#[test]
fn maps_script_exit_codes_to_property_outcomes() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("check.sh");
    std::fs::write(
        &script,
        "#!/bin/sh\nif grep -q keep \"$1\"; then exit 1; else exit 0; fi\n",
    )
    .unwrap();
    make_executable(&script);

    let checker = ScriptChecker::new(script.clone());
    let kept_input = dir.path().join("kept.sv");
    let lost_input = dir.path().join("lost.sv");
    std::fs::write(&kept_input, "keep\n").unwrap();
    std::fs::write(&lost_input, "drop\n").unwrap();

    let kept = checker.run_file(&kept_input).unwrap();
    let lost = checker.run_file(&lost_input).unwrap();

    assert_eq!(kept, CheckOutcome::Kept);
    assert_eq!(lost, CheckOutcome::Lost);
}

fn make_executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }
}
