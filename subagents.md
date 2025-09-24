<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# *pm create-brownfield-prd for this Codex CLI fork (https://github.com/itamux/codex): for adding a Subagents feature to a Codex CLI fork that matches the capabilities of this feature in Claude Code. The implementation should be as surgical and isolated as possible and touch as little of Codex’s original code as possible so that upstream merges will remain simple

Do not include timelines or phases, just the feature specifications, testing and acceptance criteria. References to the .claude folder should be replaced with the .codex

Here is a brownfield PRD to add a Subagents feature to the itamux/codex CLI fork that mirrors Claude Code’s subagents while keeping changes isolated and minimal; all Claude-specific paths are mapped to .codex as specified.[^1][^2]
The scope focuses strictly on feature specification plus testing and acceptance criteria without timelines or phases, and it leverages Codex’s existing configuration and MCP support to minimize churn and keep upstream merges simple.[^3][^1]

### Purpose and goals

Add a “Subagents” capability to Codex CLI that matches Claude Code’s behavior: define task-specialized subagents with their own instructions, tool permissions, and optional model selection, discoverable at project and user scopes.[^2][^3]
Ensure subagents run in isolated contexts and can be explicitly invoked or auto-suggested based on descriptions, following Claude Code patterns for parity where practical.[^4][^2]
Keep implementation surgical by encapsulating new logic in a standalone module and integrating via minimal wiring, preserving easy upstream sync with the original Codex codebase.[^1][^3]

### Out of scope

No UI beyond CLI/REPL additions is included, and no changes to remote cloud services or model backends are proposed beyond selecting models already supported by Codex or configured locally.[^3][^1]
No new MCP servers are introduced; the feature only consumes MCP tools already configured in Codex or available in the current session.[^1][^3]
No timelines, phases, pricing, or distribution mechanics are covered in this PRD per the request to focus solely on specifications and verification.[^2][^1]

### Definitions

Subagent: a specialized assistant with its own instructions, tool access, optional model, and isolated context that can be invoked to handle specific task types, analogous to Claude Code subagents.[^5][^2]
Project subagents: subagents defined within the project and taking precedence over global definitions, mirroring Claude’s .claude/agents structure but under .codex/agents for this fork.[^2][^1]
User subagents: subagents defined at the user level for reuse across projects, placed under ~/.codex/agents analogous to ~/.claude/agents in Claude Code.[^1][^2]

### Comparison: Claude vs Codex mapping

| Aspect | Claude Code | Codex CLI fork |
| :-- | :-- | :-- |
| Project subagents path | .claude/agents/ [^2] | .codex/agents/ [^2][^1] |
| User subagents path | ~/.claude/agents/ [^2] | ~/.codex/agents/ [^2][^1] |
| Config file (global) | ~/.claude/settings.json [^3] | ~/.codex/config.toml [^1] |
| Manager command | /agents in REPL [^2] | agents subcommand or REPL command in Codex [^2][^1] |

### User stories

As a developer, define a “code-reviewer” subagent in .codex/agents to review diffs with limited tools and invoke it explicitly in a session using natural language or a manager command, similar to Claude Code usage.[^2][^1]
As a developer, place shared subagents in ~/.codex/agents and have project-local definitions override them when names collide, matching Claude precedence semantics.[^3][^2]
As a developer, let subagents inherit available tools by default but restrict specific subagents to a curated tool list to increase safety and predictability, mirroring Claude’s configuration.[^3][^2]

### Functional requirements

Codex must load subagent definitions from .codex/agents (project) and ~/.codex/agents (user), resolving conflicts in favor of project definitions as in Claude Code.[^2][^3]
Each subagent file must use Markdown with YAML frontmatter supporting fields name, description, tools (optional), and model (optional), where omitted tools inherit all session tools and omitted model uses a default selection.[^3][^2]
Codex must support explicit invocation by subagent name in prompts and expose a CLI/REPL “agents” manager for listing, inspecting, and creating subagents, analogous to Claude’s /agents.[^1][^2]

### Configuration and files

Add a new directory convention: .codex/agents within a repo and ~/.codex/agents globally for discoverability and sharing, following the Claude layout but mapped to Codex paths.[^1][^2]
Keep subagent definitions as Markdown with YAML frontmatter so they are diffable, code-reviewable, and portable across projects, mirroring Claude Code’s approach.[^2][^3]
Use ~/.codex/config.toml for feature switches and defaults, introducing subagents.enabled, subagents.default_model, and subagents.discovery settings to avoid scattering config and to align with Codex’s TOML-based configuration.[^3][^1]

### Subagent schema

Implement the following schema in each Markdown file’s YAML frontmatter, consistent with Claude Code concepts and fields: name (required), description (required), tools (optional), model (optional), with semantics matching Claude’s inheritance and selection rules where feasible.[^2][^3]
A typical example should parse and load identically to Claude with renamed paths, enabling parity while keeping Codex configuration cohesive and minimal in changes.[^1][^2]
Codex should validate uniqueness of subagent names after kebab-case normalization and report conflicts with clear error messages to maintain predictable routing.[^3][^2]

```
---
name: code-reviewer
description: Expert code review specialist; use immediately after writing or modifying code.
tools: Read, Grep, Glob, Bash
model: inherit
---
You are a senior code reviewer ensuring high standards of code quality and security.
When invoked:
1. Run git diff to see recent changes
2. Focus on modified files
3. Begin review immediately
Provide feedback organized by priority (Critical, Warnings, Suggestions) with concrete fixes.
```


### Invocation and routing

Support explicit invocation via natural language phrases like “Use the code-reviewer subagent to check my recent changes,” which Claude documents and encourages for clarity.[^4][^2]
Provide an “agents” manager for listing, viewing, and creating subagents, analogous to Claude’s /agents interface but implemented as a Codex command to fit the CLI environment.[^1][^2]
Allow omission of the tools field to inherit all session tools (including MCP tools) and allow omission of model to inherit a default subagent model or the main session model when set to inherit, mirroring Claude.[^2][^3]

### Tool permissions and model selection

Respect tool whitelists per subagent, constraining access to only the named tools if tools is present; otherwise inherit all configured tools to match Claude semantics.[^3][^2]
Permit a model field that accepts alias-like values or inherit to use the main conversation’s model, aligning with Claude’s documented behavior.[^2][^3]
Document a Codex-level default subagent model via ~/.codex/config.toml (subagents.default_model), providing a single place to control defaults, analogous to Claude’s settings and environment patterns.[^1][^3]

### MCP integration

When tools are inherited, include tools exposed by enabled MCP servers so subagents can use the same capabilities as the parent context, similar to Claude’s inheritance behavior.[^3][^1]
Respect Codex’s mcp_servers configuration in ~/.codex/config.toml to avoid duplicating server definitions, ensuring subagents see the same tool registry as the main session unless restricted.[^1][^3]
Surface MCP tool names in the agents manager for selection and debugging so subagent tool restrictions are auditable and easily adjusted.[^3][^1]

### Context isolation and execution

Each subagent runs with an isolated context window to prevent conversation cross-contamination, aligning with Claude’s documented subagent model and benefits.[^5][^2]
The parent session should pass only a minimal task directive and any explicitly provided context, keeping most of the subagent’s reasoning separate to preserve the main conversation budget as in Claude best practices.[^5][^4]
Return the subagent’s final artifact or summary to the parent thread without injecting the full subagent transcript by default, with an option to expand details on demand via the manager for auditability.[^4][^2]

### CLI/REPL UX

Add codex agents to open a TUI-like manager listing project and user subagents, showing name, tools, and model with actions to view, create, or edit entries, analogous to Claude’s /agents.[^2][^1]
Support codex agents create --project and codex agents create --user to scaffold Markdown templates in the correct location, mirroring Claude’s guidance while matching Codex conventions.[^1][^2]
Allow inline explanations and examples in the manager to guide authors toward action-oriented descriptions, as Claude recommends in its subagent docs.[^2][^3]

### Security, approvals, and permissions

Honor existing sandbox and approvals flows so that subagents cannot bypass user approvals for privileged actions, aligning with Codex’s sandbox and permission models.[^3][^1]
Encourage fine-grained tool restrictions per subagent and provide clear denied tool feedback when a subagent tries to exceed its permissions, matching Claude’s permission structure.[^2][^3]
Ensure ignore patterns and deny lists continue to prevent access to sensitive files regardless of subagent configuration, preserving defense-in-depth.[^1][^3]

### Backward compatibility and isolation

Introduce all logic in a new isolated module (e.g., subagents/) with a single integration seam in the request routing layer, minimizing touch points with existing Codex code.[^3][^1]
Gate the feature behind subagents.enabled in ~/.codex/config.toml so existing users experience no behavior change unless they opt in.[^1][^3]
Adopt file-based discovery and TOML configuration only, avoiding schema or core architectural changes to keep upstream merges simple.[^3][^1]

### Testing strategy

Unit tests: frontmatter parser, precedence resolution (project over user), name normalization and collision detection, tools list inheritance and restriction, model selection rules including inherit.[^2][^3]
Integration tests: end-to-end explicit invocation flows, agents manager list/create/edit operations, MCP tool visibility and restriction enforcement in subagent runs.[^1][^3]
Behavior tests: verify isolated context behavior by asserting main thread tokens and messages do not inflate when subagents perform multi-step work, consistent with Claude patterns.[^5][^4]

### Acceptance criteria

- Codex loads subagents from .codex/agents and ~/.codex/agents with project-level precedence, and exposes them via codex agents list with correct tools and model metadata.[^2][^1]
- Subagent files with YAML frontmatter fields name and description are required; tools and model are optional with documented inheritance semantics, and validation errors are reported clearly on load.[^3][^2]
- Explicit invocation by name triggers the subagent, runs in an isolated context, and returns a summarized result to the parent session with an option to expand details in the manager.[^4][^2]
- Tools restrictions are enforced per subagent, inherited when omitted, and MCP tools follow the same visibility rules as the parent session unless restricted.[^1][^3]
- Model selection honors model in the subagent file, inherit for main-session parity, or subagents.default_model from ~/.codex/config.toml when omitted, without modifying unrelated Codex behavior.[^2][^1]
- Feature is off by default, fully controlled by subagents.enabled, and all new code is encapsulated in a new module with a single routed call path to keep diffs surgical.[^3][^1]


### References for parity and design alignment

Claude Code documents project and user subagents under .claude/agents and ~/.claude/agents with explicit invocation and a manager, which this PRD maps to .codex/agents and ~/.codex/agents with a Codex manager command for parity.[^2][^3]
Claude Code settings enumerate tools and the Task capability for subagent execution and recommend leveraging subagents early to preserve context, which informs the isolation and inheritance rules here.[^4][^3]
Codex CLI supports ~/.codex/config.toml configuration and MCP servers, enabling surgical integration via TOML keys and existing tool registries without invasive architectural change.[^1]
<span style="display:none">[^10][^11][^12][^13][^14][^15][^16][^17][^18][^19][^20][^21][^6][^7][^8][^9]</span>

<div style="text-align: center">⁂</div>

[^1]: https://github.com/itamux/codex

[^2]: https://docs.claude.com/en/docs/claude-code/sub-agents

[^3]: https://docs.claude.com/en/docs/claude-code/settings

[^4]: https://www.anthropic.com/engineering/claude-code-best-practices

[^5]: https://superprompt.com/blog/best-claude-code-agents-and-use-cases

[^6]: https://github.com/VoltAgent/awesome-claude-code-subagents

[^7]: https://www.reddit.com/r/ClaudeAI/comments/1mhrbzn/new_claude_code_features_microcompact_enhanced/

[^8]: https://www.youtube.com/watch?v=Phr7vBx9yFQ

[^9]: https://hexdocs.pm/claude/guide-subagents.html

[^10]: https://dev.to/oikon/enhancing-claude-code-with-mcp-servers-and-subagents-29dd

[^11]: https://zachwills.net/how-to-use-claude-code-subagents-to-parallelize-development/

[^12]: https://www.reddit.com/r/ClaudeAI/comments/1mdyc60/whats_your_best_way_to_use_subagents_in_claude/

[^13]: https://lobehub.com/mcp/razyone-sub-agent-continue

[^14]: https://github.com/wshobson/agents

[^15]: https://security.googlecloudcommunity.com/community-blog-42/claude-code-subagents-mcp-soc-runbooks-5641

[^16]: https://enting.org/mastering-claude-code-sub-agent/

[^17]: https://www.claudelog.com/mechanics/sub-agents/

[^18]: https://www.youtube.com/watch?v=cR1jEOuZUUI

[^19]: https://www.youtube.com/watch?v=nvYybDRQXLo

[^20]: https://www.reddit.com/r/ClaudeAI/comments/1l9ja9h/psa_dont_forget_you_can_invoke_subagents_in/

[^21]: https://www.pubnub.com/blog/best-practices-for-claude-code-sub-agents/