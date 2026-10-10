# fnm-shim

Small, dependency-free Unix shims for `node`, `npm`, `npx`, and `corepack`.
Selects the nearest `.node-version` or `.nvmrc`, resolves the highest matching
installed Node version, and replaces itself with the real command. Other version
requests fall back to `fnm exec`; without a version file, it uses the fnm default.

Requires [fnm](https://github.com/Schniz/fnm) and an installed Node version.
Set `FNM_DIR` to your fnm data directory; otherwise the shim uses
`$HOME/.local/share/fnm`.

## Install

A dedicated directory keeps the shims' PATH position independent of
`~/.local/bin`, which other installers often prepend. It also keeps the binaries,
symlinks, and Cargo installation metadata together for easy removal.

### From GitHub with Cargo

Requires Rust 1.99 or newer. No crates.io publication is needed. Install the
current `main` branch:

```sh
rustup update stable
cargo +stable install --git https://github.com/awe123343/fnm-shim \
  --locked --root "$HOME/.local/share/fnm-shim"
```

To install a specific release tag (replace `v0.0.5` with your chosen release):

```sh
cargo +stable install --git https://github.com/awe123343/fnm-shim \
  --tag v0.0.5 --locked --root "$HOME/.local/share/fnm-shim"
```

To pin an exact commit instead:

```sh
cargo +stable install --git https://github.com/awe123343/fnm-shim \
  --rev 006c374 --locked --root "$HOME/.local/share/fnm-shim"
```

Replace `006c374` with the commit you want (a full SHA is also accepted).
Development and release builds use the toolchain pinned in `rust-toolchain.toml`.

### Prebuilt binary

Download the matching archive and `SHA256SUMS` from
[GitHub Releases](https://github.com/awe123343/fnm-shim/releases).

| Platform | Target |
| --- | --- |
| Linux x86-64 (Ubuntu 24.04 or compatible glibc) | `x86_64-unknown-linux-gnu` |
| Linux ARM64 (Ubuntu 24.04 or compatible glibc) | `aarch64-unknown-linux-gnu` |
| macOS Intel | `x86_64-apple-darwin` |
| macOS Apple Silicon | `aarch64-apple-darwin` |

For example, download the latest Apple Silicon release using GitHub CLI
(replace the target for other platforms):

```sh
tag=$(gh release view --repo awe123343/fnm-shim --json tagName --jq .tagName)
gh release download "$tag" --repo awe123343/fnm-shim \
  --pattern '*-aarch64-apple-darwin.tar.gz' --pattern SHA256SUMS
```

In the download directory, replace `ARCHIVE` with the downloaded filename.
New archives use `fnm-shim-<tag>-<target>.tar.gz`:

```sh
archive=ARCHIVE
shasum -a 256 -c SHA256SUMS --ignore-missing
dir="$HOME/.local/share/fnm-shim/bin"
mkdir -p "$dir"
tar -xzf "$archive" -C "$dir" fnm-shim
```

### Enable the shims

After either installation method:

```sh
dir="$HOME/.local/share/fnm-shim/bin"
for tool in node npm npx corepack; do
  ln -sf fnm-shim "$dir/$tool"
done
```

For interactive shells, add the dedicated directory to PATH **before** evaluating `fnm env`, so
fnm's multishell directory takes precedence and `fnm use` keeps working. For zsh:

```sh
export PATH="$HOME/.local/share/fnm-shim/bin:$PATH"
eval "$(fnm env --use-on-cd --shell zsh)"
```

For GUI tools without a fnm multishell, include `~/.local/share/fnm-shim/bin` in the tool's
PATH ahead of other Node installations, and pass the correct `FNM_DIR`.
GUI apps do not necessarily inherit your shell configuration. The shim then
selects a version from the project or fnm's default alias.

Putting the shim ahead of the multishell instead forces project-file selection
on every invocation, overriding the version chosen by `fnm use`.
Invoke `node`, `npm`, `npx`, or `corepack`,
rather than invoking `fnm-shim` directly. `/usr/local/bin` also works for a
system-wide install, but may require administrator permissions and shares its
PATH position with other tools.

To uninstall the dedicated installation, remove its PATH entry and delete
`~/.local/share/fnm-shim` (this includes Cargo's installation metadata).

### macOS signing and Gatekeeper

Release binaries are **not Developer ID signed or notarized by Apple**.
Browser downloads may carry quarantine metadata and be blocked by Gatekeeper.
Prefer building locally with Cargo, or downloading with `gh release download`
and verifying `SHA256SUMS` before extraction.

## Releases

Pull requests build and test all four targets and upload Actions artifacts
named `fnm-shim-pr<number>-<sha7>-<target>` (the SHA identifies the tested merge
commit). These are preview builds, not published versions.
Pushes to `main` trigger builds and publish a GitHub Release only when `src/**`,
`Cargo.toml`, `Cargo.lock`, or `rust-toolchain.toml` changes. Releases include
binaries, `SHA256SUMS`, and generated release notes. Documentation, tests, and
workflow-only changes do not publish releases; their PRs still run CI.

Tags use `v0.0.<workflow run number>`, and point to the exact built commit.
Numbers may have gaps because PR runs also increment the counter. These are
build release identifiers, independent of the Cargo package version. No manual
tagging or crates.io publishing is required.

## License

[MIT](LICENSE). Release archives include the license text.
