use crate::{Result, config::Config, project};
use clap::Subcommand;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, ErrorKind},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

#[derive(Clone, Copy, Subcommand)]
pub enum Event {
    /// Ask the agent to update configured files above the context threshold.
    Stop,
    /// Record where the host keeps the raw conversation, before compaction.
    PreCompact,
    /// Hand back the configured paths on startup, resume, clear or compact.
    SessionStart,
    /// Repeat that entry when startup delivery waits for the first input.
    UserPromptSubmit,
}

impl Event {
    fn name(self) -> &'static str {
        match self {
            Self::Stop => "Stop",
            Self::PreCompact => "PreCompact",
            Self::SessionStart => "SessionStart",
            Self::UserPromptSubmit => "UserPromptSubmit",
        }
    }
    fn accepts_context(self) -> bool {
        matches!(self, Self::SessionStart | Self::UserPromptSubmit)
    }
}

#[derive(Deserialize)]
struct Input {
    cwd: PathBuf,
    session_id: Option<String>,
    source: Option<String>,
    transcript_path: Option<PathBuf>,
    #[serde(default)]
    stop_hook_active: bool,
}

/// Where PreCompact leaves the path of the raw conversation. tin keeps the
/// path, never the bytes: the host keeps that file for months, so copying it
/// would duplicate an unbounded amount of raw text into the project.
const POINTER: &str = "last-transcript-path.txt";

fn context(event: Event, text: String) -> Value {
    json!({"hookSpecificOutput": {"hookEventName": event.name(), "additionalContext": text}})
}

pub fn warning(event: Event, message: &str) -> Value {
    let text = format!(
        "tin warning: {message}\nTell the user briefly. Keep usable material, decide the next step, and do not report a failed save or restore as successful."
    );
    if event.accepts_context() {
        context(event, text)
    } else {
        json!({"systemMessage": text})
    }
}

pub fn run(event: Event, text: &str) -> Value {
    let input: Input = match serde_json::from_str(text) {
        Ok(input) => input,
        Err(e) => return warning(event, &format!("Invalid hook input: {e}")),
    };
    let root = match project::find(&input.cwd) {
        Ok(Some(root)) => root,
        Ok(None) => return json!({}),
        Err(e) => return warning(event, &e.to_string()),
    };
    let result = Config::load(&root).and_then(|config| handle(event, &input, &root, &config));
    match result {
        Ok(value) => value,
        Err(e) => {
            let mut message = e.to_string();
            if !event.accepts_context()
                && let Err(note_error) = write_runtime(&root, "warning.txt", message.as_bytes())
            {
                message.push_str(&format!(
                    "; Cannot retain warning for the next entry: {note_error}"
                ));
            }
            warning(event, &message)
        }
    }
}

fn handle(event: Event, input: &Input, root: &Path, config: &Config) -> Result<Value> {
    match event {
        Event::Stop => {
            if input.stop_hook_active {
                return Ok(json!({}));
            }
            let transcript = input.transcript_path.as_deref().ok_or(
                "No transcript path; context usage is unknown. Save manually when needed.",
            )?;
            let ratio = usage_ratio(transcript)?;
            if ratio < config.save.threshold_ratio {
                return Ok(json!({}));
            }
            let files: Vec<_> = config
                .documents
                .iter()
                .filter(|doc| doc.save)
                .map(|doc| doc.path.as_str())
                .collect();
            if files.is_empty() {
                return Ok(json!({}));
            }
            let marker = reminder_path(root, input)?;
            fs::create_dir_all(marker.parent().unwrap())?;
            match fs::File::create_new(&marker) {
                Ok(_) => (),
                Err(e) if e.kind() == ErrorKind::AlreadyExists => return Ok(json!({})),
                Err(e) => return Err(format!("Cannot record save reminder: {e}").into()),
            }
            Ok(json!({"decision": "block", "reason": format!(
                "tin: Update these files in {} now, then finish. Do not call another save command.\n{}\n{}\nDo not change read-only reference files. If saving fails, tell the user; do not keep retrying.\nThen tell the user the notes are current as of this moment, so starting a fresh thread now loses nothing. Work after this point is at risk until the next save, and an automatic compaction can arrive without passing through this reminder. Name the host's own command for it; tin does not know the interface. Help: tin --help",
                project::shown(root), files.join("\n"), config.save.instructions
            )}))
        }
        Event::PreCompact => {
            if config.save.transcript_pointer {
                let path = input.transcript_path.as_deref().ok_or(
                    "No transcript path; cannot record where the raw conversation is kept",
                )?;
                // Recording a pointer that does not resolve is the kind of
                // quiet half-success this tool exists to avoid.
                fs::metadata(path).map_err(|e| {
                    format!(
                        "Cannot reach the transcript at {}: {e}",
                        project::shown(path)
                    )
                })?;
                write_runtime(root, POINTER, project::shown(path).as_bytes())?;
            }
            Ok(json!({}))
        }
        Event::SessionStart | Event::UserPromptSubmit => {
            if matches!(event, Event::SessionStart) && input.source.as_deref() == Some("compact") {
                match fs::remove_file(reminder_path(root, input)?) {
                    Ok(()) => (),
                    Err(e) if e.kind() == ErrorKind::NotFound => (),
                    Err(e) => return Err(format!("Cannot reset save reminder: {e}").into()),
                }
            }
            // SessionStart happens once, UserPromptSubmit on every prompt, so
            // only the first can afford prose. Both carry the paths.
            let mut text = if matches!(event, Event::SessionStart) {
                format!(
                    "tin context for {}, paths relative to it.\n{}\ntin records paths, not contents: read what this task needs, because a copy here would only be a snapshot. You can ask to save context at any time, in any language; tell the user that once. Help: tin --help\n",
                    project::shown(root),
                    config.restore.instructions,
                )
            } else {
                format!(
                    "tin context for {}, paths relative to it. Read what this task needs if it is not loaded.\n",
                    project::shown(root)
                )
            };
            let note = project::local_path(root, ".tin/runtime/warning.txt")?;
            match fs::read_to_string(&note) {
                Ok(message) => {
                    text.push_str(&format!("\ntin warning: {message}\nTell the user; the previous hook did not complete.\n"));
                    if let Err(e) = fs::remove_file(&note) {
                        text.push_str(&format!(
                            "tin warning: Cannot remove {}: {e}\n",
                            project::shown(&note)
                        ));
                    }
                }
                Err(e) if e.kind() == ErrorKind::NotFound => (),
                Err(e) => text.push_str(&format!(
                    "tin warning: Cannot read {}: {e}. Tell the user.\n",
                    project::shown(&note)
                )),
            }
            let (mut writable, mut reference) = (Vec::new(), Vec::new());
            for doc in &config.documents {
                project::local_path(root, &doc.path)?;
                let line = describe(root, &doc.path);
                if doc.save {
                    &mut writable
                } else {
                    &mut reference
                }
                .push(line);
            }
            for (heading, paths) in [
                ("\nUpdate when saving:\n", writable),
                ("\nRead-only reference:\n", reference),
            ] {
                if !paths.is_empty() {
                    text.push_str(heading);
                    for line in paths {
                        text.push_str(&format!("  {line}\n"));
                    }
                }
            }
            if config.save.transcript_pointer {
                match fs::read_to_string(project::local_path(
                    root,
                    &format!(".tin/runtime/{POINTER}"),
                )?) {
                    Ok(recorded) => {
                        let recorded = recorded.trim();
                        text.push_str(&format!(
                            "\nRaw conversation: {}\n",
                            detail(recorded, Path::new(recorded), "gone from the host")
                        ));
                        // Explaining it every turn is prose UserPromptSubmit
                        // cannot afford; the path itself is the useful part.
                        if matches!(event, Event::SessionStart) {
                            text.push_str("The host's own record of the session before its last compaction, not a summary. If it is newer than the notes above, it holds work they are missing.\n");
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::NotFound => (),
                    Err(e) => text.push_str(&format!(
                        "tin warning: cannot read the recorded transcript path: {e}\n"
                    )),
                }
            }
            Ok(context(event, text))
        }
    }
}

/// One line per configured path: where it is and how stale it is. The age is
/// the cheap part of this: comparing a note's age with the raw transcript's
/// tells the agent its notes lag the actual work without reading anything.
fn describe(root: &Path, relative: &str) -> String {
    detail(relative, &root.join(relative), "not created yet")
}

/// `absent` distinguishes a document the agent has not written from a recorded
/// transcript the host no longer has; both are worth saying, differently.
fn detail(label: &str, path: &Path, absent: &str) -> String {
    match fs::metadata(path) {
        Ok(metadata) if metadata.len() == 0 => {
            format!("{label}  tin warning: empty, it holds nothing")
        }
        Ok(metadata) => match metadata
            .modified()
            .map(|at| SystemTime::now().duration_since(at))
        {
            Ok(Ok(elapsed)) => format!("{label}  written {}", age(elapsed)),
            _ => format!("{label}  write time unavailable"),
        },
        Err(e) if e.kind() == ErrorKind::NotFound => format!("{label}  tin warning: {absent}"),
        Err(e) => format!("{label}  tin warning: cannot inspect it: {e}"),
    }
}

fn age(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs();
    let (count, unit) = match seconds {
        0..=89 => (seconds, "second"),
        90..=5399 => (seconds / 60, "minute"),
        5400..=129_599 => (seconds / 3600, "hour"),
        _ => (seconds / 86400, "day"),
    };
    format!("{count} {unit}{} ago", if count == 1 { "" } else { "s" })
}

fn reminder_path(root: &Path, input: &Input) -> Result<PathBuf> {
    let session = input
        .session_id
        .as_deref()
        .filter(|id| !id.is_empty())
        .ok_or("No session ID; cannot deduplicate save reminders. Save manually when needed.")?;
    let key: String = session.bytes().map(|byte| format!("{byte:02x}")).collect();
    project::local_path(root, &format!(".tin/runtime/reminded-{key}"))
}

fn write_runtime(root: &Path, name: &str, bytes: &[u8]) -> Result<()> {
    let directory = project::local_path(root, ".tin/runtime")?;
    fs::create_dir_all(directory)?;
    let path = project::local_path(root, &format!(".tin/runtime/{name}"))?;
    fs::write(&path, bytes).map_err(|e| format!("Cannot write {}: {e}", project::shown(&path)))?;
    Ok(())
}

fn usage_ratio(path: &Path) -> Result<f64> {
    let file =
        fs::File::open(path).map_err(|e| format!("Cannot read {}: {e}", project::shown(path)))?;
    let mut ratio = None;
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line)?;
        if value["payload"]["type"] == "token_count" {
            let info = &value["payload"]["info"];
            ratio = match (
                info["last_token_usage"]["total_tokens"].as_u64(),
                info["model_context_window"].as_u64(),
            ) {
                (Some(used), Some(window)) if window > 0 => Some(used as f64 / window as f64),
                _ => None,
            };
        }
    }
    ratio.ok_or_else(|| "Context usage is unknown; no usable token_count observation. Save manually when needed.".into())
}
