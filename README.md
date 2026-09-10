<p align="center">
  <img src="https://raw.githubusercontent.com/FeilixX/tin/main/assets/hero.png"
       alt="tin: keep the why across sessions. Project-owned context for Codex."
       width="880">
</p>

# tin

Codex compaction and new sessions lose the reasoning behind a project. The files
survive; why they look that way does not.

tin is a Codex plugin that asks your agent to write that down before the context
fills up, and hands the paths back when work resumes. macOS and Windows.

## The split

```
tin      the mechanical part: when to ask, which paths, how stale each one is
agent    the semantic part: what the notes actually say
config   the difference between one project and the next
```

tin never reads or writes the content of your documents. That line is what makes
the rest of it trustworthy.

## What it does

**At a context threshold** (80% by default, configurable) the Stop hook asks the
agent to update the documents you marked writable, and to tell you the notes are
current as of that moment, so a fresh thread started then loses nothing.

**At session start** it hands back the configured paths, grouped by whether the
agent should write them, each with how long ago it was written. Not the
contents: a document has no size limit, injected context does, and the host
truncates an oversized injection without saying so. An age is also the one thing
reading cannot recover cheaply. A note older than the raw conversation means the
notes lag the work.

**Before compaction** it records where Codex keeps that raw conversation, so a
later session can dig for work the notes never received.

## What it does not do

**Store content.** `tin init` writes `.tin/config.toml` and a gitignore. Nothing
else. Its runtime state is an empty marker file, one line of diagnostics and one
recorded path.

**Define a format.** Your documents keep whatever shape they already have. tin
records paths, not schemas.

**Judge your notes.** It can say a note is old. It cannot say it is good. The
reminder is a request, and tin reports that it asked, never that the save
succeeded.

**Run anything.** Each hook reads the configuration, does one mechanical thing
and exits. No daemon, no global directory, no database. Git is optional and
never required.

## Install

Copy this to your agent. The agent's manual is [INSTALL.md](INSTALL.md).

```text
Install tin from https://github.com/FeilixX/tin and configure it for my current
project. Read INSTALL.md there and follow it. Prefer my existing context
documents and preserve existing content. Use tin --help for the usage protocol.
Tell me when setup is complete and whether I need to restart Codex or approve
hook trust.
```

By hand instead: put the `tin` executable for your platform from the
[releases page](https://github.com/FeilixX/tin/releases) on your PATH, then

```text
codex plugin marketplace add FeilixX/tin
codex plugin add tin@tin
```

Restart Codex and approve hook trust when prompted. The hooks name `tin` bare,
so it has to be on PATH.

## Configure

`tin init` in the project writes `.tin/config.toml`. Point it at documents the
project already has; create one only if it has none.

```toml
[save]
threshold_ratio = 0.80
transcript_pointer = true
instructions = "What the agent should write when the threshold is reached."

[restore]
instructions = "What the agent should do with these paths at session start."

# One block per document. These two names are placeholders: the paths belong to
# your project, and tin expects nothing in particular to exist.
[[documents]]
path = "docs/PLAN.md"
save = true          # the agent updates this one

[[documents]]
path = "ARCHITECTURE.md"
save = false         # read-only reference
```

`tin --help` is the protocol the agent follows. Changes to the configuration
apply on the next hook call.

## License

[MIT](LICENSE)
