mod config;
mod hook;
mod project;

use clap::{Parser, Subcommand};
use std::{io::Read, path::PathBuf, process::ExitCode};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(name = "tin", version, about = "Keeps a project's working context across compaction and new sessions: a CLI any agent can run, with optional Codex hooks.", after_help = HELP)]
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
    /// Print the configured paths and how stale each one is. Run it at the start of a task.
    #[command(
        after_help = "Finds the nearest .tin/config.toml. Prints paths, never file contents: read the documents the task needs. A warning left by a Codex hook is shown once."
    )]
    Status {
        /// Directory to start from; defaults to the current directory.
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

const HELP: &str = concat!(
    "AGENT PROTOCOL
  tin init      Once per project. Point .tin/config.toml at the project's own
                documents; tin creates none.
  tin status    At the start of a task. Read the listed documents the task needs.
  Save context  When the user asks, in any language: update the documents marked
                save = true. No command is needed.

  Configuration, the Codex hooks and the full protocol:
  https://github.com/FeilixX/tin/blob/v",
    env!("CARGO_PKG_VERSION"),
    "/INSTALL.md"
);

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
        Some(Command::Status { project }) => hook::status(project),
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
