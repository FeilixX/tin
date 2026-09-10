use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

fn init(root: &Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_tin"))
        .arg("init")
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn hook(root: &Path, event: &str, extra: Value) -> Value {
    let mut input = json!({"cwd": root, "session_id": "test-session"});
    input
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let mut child = Command::new(env!("CARGO_BIN_EXE_tin"))
        .args(["hook", event])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn configured_files_are_saved_by_agent_and_restored_through_hooks() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    init(root);
    let transcript = root.join("conversation.jsonl");
    let observation = json!({"payload":{"type":"token_count","info":{
        "last_token_usage":{"total_tokens":95}, "model_context_window":100
    }}})
    .to_string();
    fs::write(&transcript, &observation).unwrap();
    let input = json!({"transcript_path":transcript});
    let stop = hook(root, "stop", input.clone());
    assert_eq!(stop["decision"], "block");
    assert!(stop["reason"].as_str().unwrap().contains("context.md"));
    // This write is what the caller agent does after receiving the prompt.
    fs::write(
        root.join(".tin/context.md"),
        "Goal: ship the lightweight hooks.\nNext: local install.",
    )
    .unwrap();
    assert_eq!(
        hook(root, "stop", json!({"stop_hook_active":true})),
        json!({})
    );
    init(root);
    assert!(
        fs::read_to_string(root.join(".tin/context.md"))
            .unwrap()
            .contains("Goal:")
    );
    assert_eq!(hook(root, "stop", input.clone()), json!({}));
    assert_eq!(hook(root, "pre-compact", input.clone()), json!({}));
    // The path of the host's own record, not a copy of it.
    assert_eq!(
        fs::read_to_string(root.join(".tin/runtime/last-transcript-path.txt")).unwrap(),
        transcript.display().to_string()
    );
    for source in ["compact", "startup", "clear"] {
        let output = hook(root, "session-start", json!({"source":source}));
        let text = output["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap();
        // Paths and staleness, never contents: file bodies are unbounded and
        // the host silently truncates an oversized injection.
        assert!(text.contains(".tin/context.md"));
        assert!(text.contains("written "));
        assert!(!text.contains("Goal: ship the lightweight hooks."));
        assert!(text.contains("conversation.jsonl"));
    }
    assert_eq!(hook(root, "stop", input.clone())["decision"], "block");
    assert_eq!(hook(root, "stop", input.clone()), json!({}));
    let fallback = hook(root, "user-prompt-submit", json!({}));
    assert!(
        fallback["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap()
            .contains(".tin/context.md")
    );
    let config = root.join(".tin/config.toml");
    fs::write(
        &config,
        fs::read_to_string(&config).unwrap().replace("0.80", "0.98"),
    )
    .unwrap();
    assert_eq!(
        hook(
            root,
            "stop",
            json!({"transcript_path":transcript, "session_id":"below-threshold"})
        ),
        json!({})
    );
    // A directory would stat fine and be reported with an age, as if a note.
    fs::create_dir(root.join("notes")).unwrap();
    fs::write(
        &config,
        format!(
            "{}\n[[documents]]\npath = \"notes\"\nsave = true\n",
            fs::read_to_string(&config).unwrap()
        ),
    )
    .unwrap();
    let rejected = hook(root, "session-start", json!({"source":"startup"}));
    assert!(
        rejected["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap()
            .contains("Not a file: notes"),
        "{rejected}"
    );
}

#[test]
fn warning_reaches_agent_without_blocking() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    assert_eq!(hook(root, "session-start", json!({})), json!({}));
    init(root);
    let output = hook(
        root,
        "pre-compact",
        json!({"transcript_path":root.join("missing.jsonl")}),
    );
    assert!(
        output["systemMessage"]
            .as_str()
            .unwrap()
            .contains("tin warning:")
    );
    assert!(output.get("decision").is_none());
    let output = hook(root, "session-start", json!({}));
    assert!(
        output["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap()
            .contains("previous hook did not complete")
    );
    assert!(!root.join(".tin/runtime/warning.txt").exists());
}
