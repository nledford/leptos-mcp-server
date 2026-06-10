# Installing the Leptos MCP Server skill

This repository includes an Agent Skill at:

```text
.agents/skills/leptos-mcp-server/SKILL.md
```

## Install with the `skills` npm package

Install directly from GitHub:

```bash
npx skills add https://github.com/nledford/leptos-mcp-server --skill leptos-mcp-server
```

For a local checkout, install from the repository root:

```bash
npx skills add ./.agents/skills/leptos-mcp-server
```

The `skills` CLI expects a skill directory containing `SKILL.md` with YAML frontmatter. This skill follows that format, and `--skill leptos-mcp-server` selects this nested skill directory when installing from the GitHub repository.

## Manual install

Copy the skill directory into an agent skill directory:

```bash
mkdir -p ~/.agents/skills
cp -R .agents/skills/leptos-mcp-server ~/.agents/skills/leptos-mcp-server
```

## Npm publishing status

This repository is a Rust/Cargo package, not an npm package, and it has no `package.json`. No npm package metadata was added; the GitHub install command above uses the repository as a `skills` source rather than an npm package.

If this repository later becomes an npm package, include `.agents/skills/**` in the package `files` metadata so the skill is present in the published tarball.
