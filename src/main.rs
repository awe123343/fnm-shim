//! node/npm/npx/corepack shim for fnm: picks the Node version from the nearest
//! .nvmrc / .node-version above cwd (fnm default if none) and execs the real binary.
//! Fast path resolves plain versions ("22", "v22.22.2") against installed dirs;
//! anything else (lts/*, not installed) is handed to `fnm exec`, which errors clearly.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

const VERSION_FILES: [&str; 2] = [".node-version", ".nvmrc"];

fn fnm_dir() -> PathBuf {
    env::var_os("FNM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env::var_os("HOME").expect("HOME unset")).join(".local/share/fnm")
        })
}

fn find_version_file() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    cwd.ancestors()
        .flat_map(|d| VERSION_FILES.iter().map(move |f| d.join(f)))
        .find(|p| p.is_file())
}

fn parse(v: &str) -> Option<Vec<u64>> {
    v.trim_start_matches('v')
        .split('.')
        .map(|p| p.parse().ok())
        .collect()
}

/// Highest installed version whose leading components equal the request.
fn resolve(fnm: &Path, request: &str) -> Option<PathBuf> {
    let want = parse(request)?;
    fs::read_dir(fnm.join("node-versions"))
        .ok()?
        .filter_map(|e| {
            let name = e.ok()?.file_name().into_string().ok()?;
            let have = parse(&name)?;
            (have.len() >= want.len() && have[..want.len()] == want[..]).then_some((have, name))
        })
        .max()
        .map(|(_, name)| fnm.join("node-versions").join(name).join("installation"))
}

fn main() {
    let mut args = env::args_os();
    let argv0 = PathBuf::from(args.next().unwrap_or_default());
    let tool = argv0.file_name().unwrap_or_default().to_owned();
    let fnm = fnm_dir();

    let install = match find_version_file() {
        None => fnm.join("aliases/default"),
        Some(file) => {
            let request = fs::read_to_string(&file).unwrap_or_default();
            match resolve(&fnm, request.trim()) {
                Some(dir) => dir,
                None => {
                    let err = Command::new("fnm")
                        .arg("exec")
                        .arg(format!("--using={}", file.display()))
                        .arg("--")
                        .arg(&tool)
                        .args(args)
                        .exec();
                    eprintln!(
                        "fnm-shim: cannot resolve {} and failed to run fnm: {err}",
                        file.display()
                    );
                    exit(127);
                }
            }
        }
    };

    let bin = install.join("bin");
    // Put the resolved bin first so child processes (npm's `#!/usr/bin/env node`, scripts) skip the shim.
    let mut path = OsString::from(&bin);
    if let Some(old) = env::var_os("PATH") {
        path.push(":");
        path.push(old);
    }
    let err = Command::new(bin.join(&tool))
        .args(args)
        .env("PATH", path)
        .exec();
    eprintln!(
        "fnm-shim: failed to exec {}: {err}",
        bin.join(&tool).display()
    );
    exit(127);
}
