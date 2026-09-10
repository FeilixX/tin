use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

/// Codex hands each hook command to the host shell, and the shells disagree
/// about what a quoted path means, so a wrong command line can fail without an
/// error. The shipped commands name `tin` bare and rely on PATH, which both
/// /bin/sh and PowerShell resolve the same way. Prove it here: put the built
/// executable on PATH under the name the template uses, run every shipped
/// command through this host's shell, and require hook JSON back.
#[test]
fn every_shipped_hook_command_runs_on_this_platform() {
    let hooks: Value = serde_json::from_str(
        &fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("plugin/hooks/hooks.json"))
            .unwrap(),
    )
    .unwrap();

    let bin = tempfile::tempdir().unwrap();
    let installed = bin
        .path()
        .join(if cfg!(windows) { "tin.exe" } else { "tin" });
    fs::copy(env!("CARGO_BIN_EXE_tin"), &installed).unwrap();
    let path = std::env::join_paths(
        std::iter::once(bin.path().to_owned())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();

    let project = tempfile::tempdir().unwrap();
    let init = Command::new(env!("CARGO_BIN_EXE_tin"))
        .arg("init")
        .current_dir(project.path())
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    fs::write(project.path().join(".tin/context.md"), "notes").unwrap();

    let events = hooks["hooks"].as_object().expect("hook events");
    assert_eq!(events.len(), 4, "the template should configure four events");
    for (event, entries) in events {
        let command = entries[0]["hooks"][0]["command"].as_str().unwrap();
        let output = run_through_host_shell(command, &path, project.path(), event);
        if event == "SessionStart" {
            assert!(
                output["hookSpecificOutput"]["additionalContext"]
                    .as_str()
                    .is_some_and(|context| context.contains(".tin/context.md")),
                "{event} did not deliver the context pointer: {output}"
            );
        }
    }
}

fn run_through_host_shell(
    command: &str,
    path: &std::ffi::OsStr,
    project: &Path,
    event: &str,
) -> Value {
    let mut shell = if cfg!(windows) {
        let mut shell = Command::new("powershell.exe");
        shell.args(["-NoProfile", "-Command", command]);
        shell
    } else {
        let mut shell = Command::new("sh");
        shell.args(["-c", command]);
        shell
    };
    let mut child = shell
        .env("PATH", path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let input = json!({
        "cwd": project,
        "session_id": "packaging-test",
        "source": "startup",
        "hook_event_name": event,
    });
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{event} command failed: {command}\n{stdout}"
    );
    serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("{event} produced no hook JSON: {e}\ncommand: {command}\nstdout: {stdout}")
    })
}
