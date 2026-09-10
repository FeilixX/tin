use crate::{Result, config};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Component, Path, PathBuf},
};

/// canonicalize hands back Windows verbatim paths (\\?\C:\...). They work, but
/// the agent copies what it reads here into its own tools and into prose for
/// the user, so show the ordinary form. UNC verbatim paths are left alone
/// because stripping the prefix would change what they point at.
pub fn shown(path: &Path) -> String {
    let text = path.display().to_string();
    match text.strip_prefix(r"\\?\") {
        Some(drive) if drive.as_bytes().get(1) == Some(&b':') => drive.to_owned(),
        _ => text,
    }
}

pub fn find(cwd: &Path) -> Result<Option<PathBuf>> {
    let cwd = cwd.canonicalize()?;
    for root in cwd.ancestors() {
        match fs::metadata(root.join(".tin/config.toml")) {
            Ok(_) => return Ok(Some(root.to_owned())),
            Err(e) if e.kind() == ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(None)
}

pub fn local_path(root: &Path, name: &str) -> Result<PathBuf> {
    let path = Path::new(name);
    if name.trim().is_empty()
        || path.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("Expected a project-relative path: {name}").into());
    }
    let joined = root.join(path);
    for ancestor in joined.ancestors() {
        match ancestor.canonicalize() {
            Ok(resolved) => {
                if !resolved.starts_with(root) {
                    return Err(format!("Path leaves the project: {name}").into());
                }
                break;
            }
            Err(e) if e.kind() == ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(joined)
}

pub fn init(directory: Option<PathBuf>) -> Result<()> {
    let root = directory
        .unwrap_or(std::env::current_dir()?)
        .canonicalize()?;
    let dir = local_path(&root, ".tin")?;
    fs::create_dir_all(dir)?;
    // No document is created here. tin writes configuration and runtime
    // metadata; the content of a context file is the agent's to write, and a
    // template tin does not track is one more thing claiming to be content.
    for (name, content) in [
        (".tin/config.toml", config::DEFAULT_CONFIG),
        (".tin/.gitignore", "runtime/\ncontext.md\n"),
    ] {
        let path = local_path(&root, name)?;
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                file.write_all(content.as_bytes())?;
                println!("Created {name}");
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => println!("Kept {name}"),
            Err(e) => return Err(format!("{}: {e}", shown(&path)).into()),
        }
    }
    config::Config::load(&root)?;
    println!(
        "Read .tin/config.toml and point it at the documents this project already has. Run tin --help for the protocol."
    );
    Ok(())
}
