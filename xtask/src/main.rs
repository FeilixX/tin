use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(e) = run() {
        eprintln!("xtask: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("package") => {
            let mut target = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--target" => target = Some(args.next().ok_or("--target needs a triple")?),
                    other => return Err(format!("Unexpected argument: {other}").into()),
                }
            }
            package(target)
        }
        _ => Err("Usage: cargo xtask package [--target <triple>]".into()),
    }
}

/// Build one release asset. The plugin itself is text in this repository and
/// ships through the marketplace; what a user downloads is the executable.
fn package(target: Option<String>) -> Result<()> {
    let root = project_root();
    let triple = match target {
        Some(triple) => triple,
        None => host_triple()?,
    };
    let executable = if triple.contains("windows") {
        "tin.exe"
    } else {
        "tin"
    };

    check_version(&root)?;

    let mut build = Command::new(cargo());
    build.current_dir(&root).args([
        "build",
        "--release",
        "--package",
        "tin",
        "--target",
        &triple,
    ]);
    status(build, "cargo build")?;

    let built = target_dir(&root)?
        .join(&triple)
        .join("release")
        .join(executable);
    if !built.exists() {
        return Err(format!("Built executable is missing: {}", built.display()).into());
    }

    let label = label(&triple);
    let dist = root.join("dist");
    let package = dist.join(format!("tin-{label}"));
    if package.exists() {
        fs::remove_dir_all(&package)?;
    }
    fs::create_dir_all(&package)?;
    fs::copy(&built, package.join(executable))?;
    for document in ["README.md", "INSTALL.md", "LICENSE"] {
        let from = root.join(document);
        fs::copy(&from, package.join(document)).map_err(|e| format!("{}: {e}", from.display()))?;
    }

    let archive = archive(&dist, &format!("tin-{label}"))?;
    println!("{}", package.display());
    println!("{}", archive.display());
    Ok(())
}

/// The plugin manifest repeats the crate version; a mismatch publishes a
/// release whose declared version is not the one the marketplace serves.
fn check_version(root: &Path) -> Result<()> {
    let manifest: toml::Value = toml::from_str(&fs::read_to_string(root.join("Cargo.toml"))?)?;
    let crate_version = manifest
        .get("package")
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .ok_or("Cargo.toml has no package version")?
        .to_owned();
    let plugin: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        root.join("plugin/.codex-plugin/plugin.json"),
    )?)?;
    let plugin_version = plugin
        .get("version")
        .and_then(serde_json::Value::as_str)
        .ok_or("plugin.json has no version")?;
    if plugin_version != crate_version {
        return Err(format!(
            "Version mismatch: Cargo.toml {crate_version}, plugin.json {plugin_version}"
        )
        .into());
    }
    Ok(())
}

/// GNU tar has no zip writer and no .zip entry in its --auto-compress table,
/// so asking it for a zip silently yields an uncompressed tar. Ask tar which
/// one it is rather than assuming from the host.
fn archive(dist: &Path, name: &str) -> Result<PathBuf> {
    let version = Command::new("tar").arg("--version").output()?;
    let bsdtar = String::from_utf8_lossy(&version.stdout).contains("bsdtar");
    let (path, flags) = if bsdtar {
        (dist.join(format!("{name}.zip")), ["-a", "-c", "-f"])
    } else {
        (dist.join(format!("{name}.tar.gz")), ["-z", "-c", "-f"])
    };
    if path.exists() {
        fs::remove_file(&path)?;
    }
    let mut tar = Command::new("tar");
    tar.current_dir(dist).args(flags).arg(&path).arg(name);
    status(tar, "tar")?;
    Ok(path)
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the project")
        .to_owned()
}

/// Cargo knows where it writes; CARGO_TARGET_DIR is only one of the ways to
/// move it, and guessing produces a "missing executable" error for the wrong
/// reason.
fn target_dir(root: &Path) -> Result<PathBuf> {
    let output = Command::new(cargo())
        .current_dir(root)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()?;
    if !output.status.success() {
        return Err("cargo metadata failed".into());
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    metadata["target_directory"]
        .as_str()
        .map(PathBuf::from)
        .ok_or_else(|| "cargo metadata reported no target directory".into())
}

fn cargo() -> String {
    env::var("CARGO").unwrap_or_else(|_| "cargo".into())
}

fn host_triple() -> Result<String> {
    let output = Command::new("rustc").arg("-vV").output()?;
    if !output.status.success() {
        return Err("rustc -vV failed".into());
    }
    String::from_utf8(output.stdout)?
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(str::to_owned))
        .ok_or_else(|| "rustc -vV reported no host triple".into())
}

/// Asset names people recognise; unknown triples keep their triple.
fn label(triple: &str) -> String {
    match triple {
        "x86_64-pc-windows-msvc" => "windows-x64",
        "aarch64-pc-windows-msvc" => "windows-arm64",
        "aarch64-apple-darwin" => "macos-arm64",
        "x86_64-apple-darwin" => "macos-x64",
        other => other,
    }
    .to_owned()
}

fn status(mut command: Command, what: &str) -> Result<()> {
    let status = command
        .status()
        .map_err(|e| format!("Cannot run {what}: {e}"))?;
    if !status.success() {
        return Err(format!("{what} failed with {status}").into());
    }
    Ok(())
}
