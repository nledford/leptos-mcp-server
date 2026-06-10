# Installing the Leptos MCP Server skill

This repository includes an Agent Skill at:

```text
.agents/skills/leptos-mcp-server/SKILL.md
```

## Install with the `skills` npm package

From the repository root:

```bash
npx skills add ./.agents/skills/leptos-mcp-server
```

The `skills` CLI expects a skill directory containing `SKILL.md` with YAML frontmatter. This skill follows that format.

## Manual install

Copy the skill directory into an agent skill directory:

```bash
mkdir -p ~/.agents/skills
cp -R .agents/skills/leptos-mcp-server ~/.agents/skills/leptos-mcp-server
```

## Npm publishing status

This repository is a Rust/Cargo package, not an npm package, and it has no `package.json`. No npm package metadata was added.

Current `skills` package documentation verifies local path installs and SKILL.md layout. It does not document stable `npx skills add <npm-package-name>` support for arbitrary npm packages. If this repository later becomes an npm package, include `.agents/skills/**` in the package `files` metadata so the skill is present in the published tarball.
