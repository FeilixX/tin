# Install tin — agent instructions

Install the tin executable, add the Codex plugin if the user works in Codex, then
configure tin for the user's target project. Resolve the environment yourself:
host platform, PATH and available build tools. The tin source directory and the
user's target project may be different.

## 1. Install the executable

Take the release archive matching this host from
https://github.com/FeilixX/tin/releases, extract it, and put the executable on
PATH. From a source checkout instead, `cargo xtask package` builds the same
archive under `dist/`. macOS and Windows are supported. Choose a PATH directory
the user already owns; ask before creating one or editing PATH.

Confirm with `tin --help`. If it does not resolve, stop and tell the user.

## 2. Codex only: install the plugin

The plugin adds hooks that run tin automatically. It carries no binary: its hook
commands name `tin` bare, so `tin` must resolve on PATH for every Codex session;
a plugin installed without it fails on every hook.

```text
codex plugin marketplace add FeilixX/tin --json
codex plugin add tin@tin --json
```

Check the response for the installed plugin and version. If tin is already
installed, consult `codex plugin --help` for the installed CLI's update procedure.
Preserve other plugins and Codex settings.

Other agents (Claude Code, Grok Build, pi and the rest) skip this step. They run
no tin hooks and use the CLI directly, as section 4 describes.

## 3. Configure the project

Resolve the target project from the user's working directory or explicit request:

```text
tin init --project <target-project>
```

Read `.tin/config.toml` and inspect the project for material needed to resume work:
goals, progress, decisions and their reasons, unfinished tasks, blockers and useful
file pointers. Prefer existing documents and preserve their format. Configure as
many files as the project needs; no authority manifest or particular content format
is required. Paths are relative to the project. Mark files to update with
`save = true` and read-only recovery references with `save = false`.

`tin init` creates no document. If the project has nothing suitable, write one
yourself at the suggested `.tin/context.md` or wherever the project prefers, and
point the configuration at it. Give it the current goal, progress, next steps,
key decisions and their reasons, unfinished work, blockers and necessary file
pointers. Keep it a description of the current state: revise what went stale
instead of appending a session log. Split it only when the project needs
separate documents.

Git is optional. If the project already uses Git, a commit hash and a short account
of uncommitted changes can help the next agent check the code state. They supplement
the context notes; they do not replace goals, reasoning or remaining work. Do not
initialize a repository or create commits solely to enable tin.

tin delivers these paths and how stale each one is, never the file contents, so
list documents worth opening rather than everything the project owns, and expect
to read them yourself. A configured path may not exist yet; one that exists but
is not a file is refused.

Adapt the save and restore instructions to these documents. Keep the supplied
threshold unless the user or project needs a different value; only the Codex
hooks use it.

## 4. Use tin

This is the protocol. It applies on every host.

- **At the start of a task**, run `tin status` in the project. It prints the
  configured paths, grouped by whether you should update them, each with how long
  ago it was written. Read the documents the task needs. A note older than the
  work it describes lags that work.
- **A request to save context**, in any language, means: update the documents
  marked `save = true` now. No save command, commit step, special phrase or
  threshold is required. Do not change read-only references. Tell the user once
  that they can ask you to save context at any time.
- Maintain the files as work progresses. File content is a working note, not
  proof of completion, and the user's current instructions take precedence.
- A `tin warning` means explain the issue to the user. Do not claim a failed save
  or restore succeeded.
- Defaults and instructions live in `.tin/config.toml` and apply on the next tin
  call.

### With the Codex plugin

The hooks do the same work without being asked, and two things the CLI cannot:

- **SessionStart** delivers the entry `tin status` prints, on startup, resume,
  clear and compact. **UserPromptSubmit** repeats the paths when startup delivery
  waits for the first input.
- **Stop** asks you once per compaction cycle to update the `save = true`
  documents, when context usage reaches `save.threshold_ratio` (80% by default,
  measured as the "Context N% used" Codex shows). When it asks, update them and
  tell the user the notes are current, so a fresh thread started then loses
  nothing. SessionStart after compaction re-enables it; `stop_hook_active`
  prevents a save loop.
- **PreCompact** records where Codex keeps the raw conversation, not a copy. The
  entry shows that path; if it is newer than the notes, it holds work they miss.

On other hosts there is no threshold reminder and no recorded conversation path.

## 5. Report the result

Tell the user which files you update when they ask to save, and which ones you
read at the start of a task. Use the actual configured paths, for example:

> I update A and B whenever you ask me to save context, in any language. At the
> start of a task `tin status` gives me A, B and C with how stale each one is,
> and I read the ones the task needs. You can change this selection at any time.

In Codex, add that at 80% context you update A and B yourself and tell them the
notes are current, and that the start-of-session list arrives on its own. Tell
them to restart Codex to load the plugin and approve hook trust when prompted,
and that the hooks need `tin` on PATH.

Elsewhere, tell them that in a new session the agent learns tin by running
`tin --help`, so mentioning tin is enough.

Explain this in the user's language. Distinguish completed installation from
hook behavior actually observed.
