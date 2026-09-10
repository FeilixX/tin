# tin

Codex compaction and new sessions can lose the intent, decisions and unfinished work behind a project. tin uses lightweight hooks to ask your agent to save that context in project files and read it when work resumes. You choose the files and instructions; your agent handles their content.

tin is a small executable plus a Codex plugin that carries the hooks. macOS and Windows are supported.

Copy this prompt to your agent to install and configure it. The agent's installation manual is [INSTALL.md](INSTALL.md).

```text
Install tin from https://github.com/FeilixX/tin and configure it for my current
project. Read INSTALL.md there and follow it. Prefer my existing context
documents and preserve existing content. Use tin --help for the usage protocol.
Tell me when setup is complete and whether I need to restart Codex or approve
hook trust.
```

To do it by hand instead: put the `tin` executable for your platform from the [releases page](https://github.com/FeilixX/tin/releases) on your PATH, then

```text
codex plugin marketplace add FeilixX/tin
codex plugin add tin@tin
```

and ask your agent to run `tin init` in the project and choose which documents to save and restore.

## License

[MIT](LICENSE)
