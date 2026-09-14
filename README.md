<p align="center">
  <img src="https://raw.githubusercontent.com/FeilixX/tin/main/assets/hero.png"
       alt="tin: keep the why across sessions."
       width="880">
</p>

# tin

Compaction and new sessions lose the reasoning behind a project. The files
survive; why they look that way does not.

tin gets your agent to write that down, and hands the paths back when work
resumes. It is a CLI any coding agent can run, plus a Codex plugin that makes it
automatic. macOS and Windows.

## The split

```
tin      the mechanical part: which paths, how stale each one is, when to ask
agent    the semantic part: what the notes actually say
config   the difference between one project and the next
```

tin never reads or writes the content of your documents. That line is what makes
the rest of it trustworthy.

## What it does

**When work resumes** `tin status` lists the configured paths, grouped by whether
the agent should write them, each with how long ago it was written. Not the
contents: a document has no size limit, an agent's context does. An age is also
the one thing reading cannot recover cheaply. A note older than the work means the
notes lag it.

**When you ask** to save context, in any language, the agent updates the
documents you marked writable. No command.

**In Codex** the plugin's hooks do this on their own. At session start they hand
back the same list. At a context threshold (80% of what Codex shows as used,
configurable) they ask the agent to update the notes and tell you they are
current, so a fresh thread started then loses nothing. Before compaction they
record where Codex keeps the raw conversation, so a later session can dig for
work the notes never received.

Claude Code, Grok Build, pi and other agents use the CLI. They get no threshold
reminder and no recorded conversation path.

## What it does not do

**Store content.** `tin init` writes `.tin/config.toml` and a gitignore. Nothing
else. Its runtime state is an empty marker file, one line of diagnostics and one
recorded path.

**Define a format.** Your documents keep whatever shape they already have. tin
records paths, not schemas.

**Judge your notes.** It can say a note is old. It cannot say it is good. The
reminder is a request, and tin reports that it asked, never that the save
succeeded.

**Run anything.** Each call reads the configuration, does one mechanical thing
and exits. No daemon, no global directory, no database. Git is optional and
never required.

## Install

Copy this to your agent. The agent's manual is [INSTALL.md](INSTALL.md).

```text
Install tin from https://github.com/FeilixX/tin and configure it for my current
project. Read INSTALL.md there and follow it. Prefer my existing context
documents and preserve existing content. Tell me when setup is complete and
whether I need to do anything in my agent.
```

By hand instead: put the `tin` executable for your platform from the
[releases page](https://github.com/FeilixX/tin/releases) on your PATH and run
`tin init` in the project. For Codex, also

```text
codex plugin marketplace add FeilixX/tin
codex plugin add tin@tin
```

then restart Codex and approve hook trust when prompted. The hooks name `tin`
bare, so it has to be on PATH.

## Configure

`tin init` in the project writes `.tin/config.toml`. Point it at documents the
project already has; create one only if it has none.

```toml
[save]
threshold_ratio = 0.80    # Codex hooks only
transcript_pointer = true # Codex hooks only
instructions = "What the agent should write when saving."

[restore]
instructions = "What the agent should do with these paths when work resumes."

# One block per document. These two names are placeholders: the paths belong to
# your project, and tin expects nothing in particular to exist.
[[documents]]
path = "docs/PLAN.md"
save = true          # the agent updates this one

[[documents]]
path = "ARCHITECTURE.md"
save = false         # read-only reference
```

`tin --help` points the agent at the protocol. Changes to the configuration apply
on the next tin call.

## License

[MIT](LICENSE)
