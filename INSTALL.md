# Install tin — agent instructions

Install the tin executable, install the Codex plugin, then configure the plugin
for the user's target project. Resolve the environment yourself: host platform,
PATH and available build tools. The tin source directory and the user's target
project may be different.

## 1. Install the executable

The plugin carries no binary. Its hook commands name `tin` bare, so `tin` must
resolve on PATH for every Codex session; a plugin installed without it fails on
every hook.

Take the release archive matching this host from
https://github.com/FeilixX/tin/releases and put the executable on PATH, or build
from a source checkout with `cargo xtask package` and install the result from
`dist/`. macOS and Windows are supported. Choose a PATH directory the user
already owns; ask before creating a new one or editing PATH.

Confirm with `tin --help`. That output is also the agent usage protocol; read it.
If `tin --help` does not resolve, stop and tell the user rather than installing
the plugin.

## 2. Install the plugin in Codex

```text
codex plugin marketplace add FeilixX/tin --json
codex plugin add tin@tin --json
```

Check the response for the installed plugin and version. If tin is already
installed, consult `codex plugin --help` for the installed CLI's update procedure.
Preserve other plugins and Codex settings.

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

If no suitable documents exist, use the suggested `.tin/context.md`. Populate it
with the current goal, progress, next steps, key decisions and reasons, unfinished
work, blockers and necessary file pointers. Keep it a concise description of the
current state: revise outdated information instead of continually appending a
session log. Split it only when the project needs separate documents.

Git is optional. If the project already uses Git, a commit hash and a short account
of uncommitted changes can help the next agent check the code state. They supplement
the context notes; they do not replace goals, reasoning or remaining work. Do not
initialize a repository or create commits solely to enable tin.

Adapt the save and restore instructions to these documents. Keep the supplied
threshold unless the user or project needs a different value. The CLI help is the
save and restore protocol.

## 4. Report the result

Tell the user exactly which files will be updated before compaction and which
will be read when restoring context. Use the actual configured paths, for example:

> Before compaction, I will update A and B. When restoring context, I will read
> A, B and C. You can change this selection or ask me to save context at any time.

Explain this in the user's language. Tell them to restart Codex to load
the plugin and approve hook trust when prompted, and that the hooks need `tin` on
PATH. Distinguish completed installation from hook behavior actually observed.
