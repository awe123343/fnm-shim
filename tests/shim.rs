use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;
use std::process::{Command, Output};

fn script(path: &Path, body: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn selects_versions_and_forwards_arguments() {
    let root = std::env::temp_dir().join(format!("fnm-shim-test-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("remove test directory");
        }
    }
    let _cleanup = Cleanup(root.clone());
    let project = root.join("project");
    let cwd = project.join("nested");
    let fnm = root.join("fnm");
    let shim = root.join("node");
    fs::create_dir_all(&cwd).unwrap();
    symlink(env!("CARGO_BIN_EXE_fnm-shim"), &shim).unwrap();
    for version in ["22.9.0", "22.10.0", "24.0.0"] {
        script(
            &fnm.join(format!("node-versions/v{version}/installation/bin/node")),
            &format!("printf '%s\\n' '{version}' \"$@\"\nprintf '%s\\n' \"${{PATH%%:*}}\""),
        );
    }
    script(&root.join("fallback/fnm"), "printf '%s\\n' \"$@\"; exit 42");
    let run = || -> Output {
        Command::new(&shim)
            .current_dir(&cwd)
            .env("FNM_DIR", &fnm)
            .env("PATH", root.join("fallback"))
            .args(["argument with spaces", ""])
            .output()
            .unwrap()
    };
    // A local file isolates the test from version files above the temp directory.
    fs::write(project.join(".nvmrc"), "22\n").unwrap();
    let output = run();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "22.10.0\nargument with spaces\n\n{}\n",
            fnm.join("node-versions/v22.10.0/installation/bin")
                .display()
        )
    );
    fs::write(project.join(".node-version"), "v22.9.0").unwrap();
    assert!(run().stdout.starts_with(b"22.9.0\n"));
    fs::write(cwd.join(".nvmrc"), "24").unwrap();
    assert!(run().stdout.starts_with(b"24.0.0\n"));
    fs::write(cwd.join(".nvmrc"), "lts/*").unwrap();
    let output = run();
    assert_eq!(output.status.code(), Some(42));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "exec\n--using={}\n--\nnode\nargument with spaces\n\n",
            cwd.join(".nvmrc").display()
        )
    );
}
