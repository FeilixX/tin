mod config;
mod hook;
mod project;

use clap::{Parser, Subcommand};
use std::{io::Read, path::PathBuf, process::ExitCode};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(name = "tin", version, about = "Project-local context hooks for your agent.", after_help = HELP)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Create project-local configuration. No document is created.
    #[command(
        after_help = "Existing files are never overwritten, and no document is created: point the configuration at the project's own documents. Edit .tin/config.toml to choose files, instructions and the save threshold. Git is optional."
    )]
    Init {
        /// Project directory; defaults to the current directory.
        #[arg(long)]
        project: Option<PathBuf>,
    },
    /// Handle a Codex event: JSON on stdin, hook JSON on stdout.
    #[command(
        after_help = "Configure these events through the tin plugin. The hook uses cwd from stdin and finds the nearest .tin/config.toml. No configuration means no action. Warnings do not block work."
    )]
    Hook {
        #[command(subcommand)]
        event: hook::Event,
    },
}

const HELP: &str = "AGENT PROTOCOL
  Run tin init in the project, then read .tin/config.toml.
  Prefer existing project documents; content can use any format.
  Only update documents marked save = true. Read all configured documents
  when restoring context. The user's current instructions take precedence.

  A request to save context, in any language, means: update those files now.
  No save command, commit step, special phrase or threshold is required.
  Tell the user once that they can ask you to save context at any time.

  Stop asks once per session above save.threshold_ratio (default 80%).
  SessionStart after compaction re-enables the reminder; stop_hook_active
  prevents a save loop. Maintain the files as work progresses.
  PreCompact records where the host keeps the raw conversation, not a copy.
  Entry hooks supply the configured paths and how stale each one is, never
  their contents; read the files you need. UserPromptSubmit repeats the same
  entry when startup delivery is delayed until the first input.

  Defaults and instructions live in .tin/config.toml and apply immediately.
  A warning means explain the issue to the user; do not claim a failed save
  or restore succeeded. File content is a working note, not proof of completion.";

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        None => {
            use clap::CommandFactory;
            Cli::command()
                .after_help("")
                .print_help()
                .map_err(Into::into)
        }
        Some(Command::Init { project }) => project::init(project),
        Some(Command::Hook { event }) => {
            let mut input = String::new();
            let output = match std::io::stdin().read_to_string(&mut input) {
                Ok(_) => hook::run(event, &input),
                Err(e) => hook::warning(event, &format!("Cannot read hook input: {e}")),
            };
            println!("{output}");
            Ok(())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("tin: {e}");
            ExitCode::FAILURE
        }
    }
}
