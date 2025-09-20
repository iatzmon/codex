# Tools Overview

- Codex runs multi-turn sessions via Session, which wires user config, model client, MCP connections, exec session managers, and sandbox settings, then coordinates tool calls, approvals, and event emission for each turn. core/src/codex.rs:304, core/src/codex.rs:448
- Each turn builds a Prompt containing conversation history, instructions, and the active tool list before streaming either the Responses API or Chat Completions API through a unified adapter. core/src/codex.rs:2700, core/src/client_common.rs:22, core/src/client.rs:113
- The protocol crate defines ResponseItem, ResponseInputItem, and event enums consumed across the stack, keeping tool outputs and UI notifications consistent between clients. protocol/src/models.rs:13, protocol/src/protocol.rs:460

Tool Catalog & Configuration

- OpenAiTool captures function, local shell, web search, and grammar-based tools; ToolsConfig::new chooses which to expose based on model family defaults, approval policy, sandbox mode, and feature toggles such as unified exec or plan mode. core/src/openai_tools.rs:45, core/src/openai_tools.rs:88
- Shell exposure can be the classic function tool, a variant that requests escalation fields, a local_shell surrogate for chat models, or the two-part streamable exec/TTY pair when experimental mode is enabled. core/src/openai_tools.rs:173, core/src/openai_tools.rs:539, core/src/exec_command/responses_api.rs:5
- Apply-patch is available as either a freeform grammar or JSON function, selected by model support or user overrides; plan, web search, and view_image tools plug in alongside any MCP tools discovered. core/src/tool_apply_patch.rs:21, core/src/openai_tools.rs:560, core/src/openai_tools.rs:575, core/src/openai_tools.rs:580
- MCP tools are normalized (name sanitization, JSON schema coercion) before being advertised, ensuring mixed-origin tools satisfy OpenAI’s schema requirements. core/src/openai_tools.rs:378, core/src/openai_tools.rs:413

Prompt Construction & API Adaptation

- Prompt::get_full_instructions stitches base instructions with model-specific apply_patch guidance when needed, preventing redundant tool primers if the tool is already available. core/src/client_common.rs:36
- Responses-mode requests serialize via ResponsesApiRequest, bundling reasoning controls, optional verbosity, and explicit tools JSON. core/src/client_common.rs:118
- Chat completions mode converts the same OpenAiTool set into chat-compatible function descriptors and adapts ResponseItem history into message/tool_call payloads while deduplicating assistant text and attaching reasoning segments. core/src/openai_tools.rs:330, core/src/chat_completions.rs:141

Streaming & Turn Processing

- run_turn rebuilds the tool list each turn (capturing live MCP inventory) and retries disconnected streams with provider-specific backoff. core/src/codex.rs:2706, core/src/codex.rs:2717
- try_run_turn consumes ResponseEvents, synthesizes missing tool outputs for aborted calls, and feeds each ResponseItem through handlers that emit follow-up commands or user-facing events. core/src/codex.rs:2775, core/src/codex.rs:2830
- Chat/Responses SSE streams converge into ResponseEvent via ModelClient::stream and the SSE processors in client.rs, ensuring consistent downstream handling regardless of provider wire API. core/src/client.rs:116, core/src/client.rs:600
- Reasoning summaries, output deltas, and web-search begin notifications are surfaced in real time so UIs can display incremental progress. core/src/client.rs:643, core/src/client_common.rs:75

Execution Pathways

- handle_function_call dispatches named tools: shell/container.exec, unified_exec sessions, view_image attachments, apply_patch invocations, plan updates, streamable exec_command/write_stdin, or MCP tool proxies. core/src/codex.rs:3156
- Shell and exec calls funnel through handle_container_exec_with_params, which auto-detects embedded apply_patch scripts, translates commands for user shells, emits begin/end events, streams output chunks (bounded by MAX_EXEC_OUTPUT_DELTAS), and records diffs for patches. core/src/codex.rs:3461, core/src/exec.rs:43, core/src/codex.rs:940
- process_exec_tool_call selects macOS seatbelt, Linux seccomp, or unsandboxed execution, enforcing timeouts and returning aggregated stdout/stderr plus duration/exit metadata. core/src/exec.rs:81
- Unified exec reuses the portable PTY stack for interactive sessions, clamping timeouts, buffering output with backpressure-aware queues, and returning session IDs for subsequent stdin writes. core/src/unified_exec/mod.rs:31, core/src/unified_exec/mod.rs:151
- Streamable exec_command exposes exec_command and write_stdin tools that run commands inside PTYs while truncating output to configured token thresholds. core/src/exec_command/responses_api.rs:5, core/src/exec_command/session_manager.rs:84

Safety, Approvals, and Sandbox

- SafetyCheck logic determines whether commands can auto-run, require approval, or be rejected based on approval policy, sandbox mode, known-safe whitelists, and with_escalated_permissions hints. core/src/safety.rs:14, core/src/safety.rs:75
- handle_container_exec_with_params applies these checks, prompting via request_command_approval when needed and promoting approved commands into the per-session whitelist to avoid duplicate prompts. core/src/codex.rs:3498, core/src/codex.rs:3561
- When approval is granted or auto-approval is possible, exec invocations choose seatbelt/landlock wrappers as appropriate and stream output events back to the session. core/src/codex.rs:3621, core/src/exec.rs:94
- Plan mode can block MCP tool execution unless explicitly allowed, logging plan entries and background events when the AI attempts a disallowed tool. core/src/codex.rs:1222

MCP Integration

- McpConnectionManager::new spins up configured servers concurrently, initializes them with declared capabilities, aggregates tool inventories, and hashes/sanitizes names to meet OpenAI constraints. core/src/mcp_connection_manager.rs:103, core/src/mcp_connection_manager.rs:124, core/src/mcp_connection_manager.rs:47
- handle_mcp_tool_call emits begin/end events with timing, marshals arguments, invokes the appropriate MCP client, and wraps success or error responses into ResponseInputItem::McpToolCallOutput. core/src/mcp_tool_call.rs:17
- Plan mode’s is_tool_allowed checks gate MCP usage, and tooling lists are refreshed each turn so newly connected servers appear in subsequent prompts. core/src/codex.rs:1222, core/src/codex.rs:2706

Event Model & Observability

- EventMsg enumerates streamed notifications (exec command begin/output/end, patch apply lifecycle, MCP tool calls, web search, reasoning, background errors) so clients can render live agent state. protocol/src/protocol.rs:460
- Exec flows send ExecCommandBegin, incremental ExecCommandOutputDelta, and ExecCommandEnd or patch equivalents, coupled with optional turn diffs emitted after apply_patch success. core/src/codex.rs:940, core/src/exec.rs:43, core/src/codex.rs:1000
- MCP tool calls publish McpToolCallBegin/McpToolCallEnd with invocation metadata, enabling telemetry and UI treatment consistent with built-in tools. protocol/src/protocol.rs:836

Extensibility & Notable Behaviors

- Plan mode swaps the tool catalog to always include update_plan while suppressing apply_patch, ensuring planning steps remain structured even when other tools are disabled. core/src/codex.rs:579, core/src/plan_tool.rs:21
- Prompt::get_full_instructions auto-injects apply_patch usage documentation only when the tool is absent, keeping prompts lean for models that already embed the helper grammar. core/src/client_common.rs:44
- Custom tool calls (from Responses API’s custom type) are mapped back into exec flows, so freeform apply_patch still funnels through the same safety and patch-diff machinery. core/src/codex.rs:3350
- Schema sanitation for MCP tools tolerates missing type, boolean schemas, and nested combinators, increasing compatibility with diverse MCP servers without manual adjustments. core/src/openai_tools.rs:413

Considerations

- Approval workflows and sandbox selection are tightly coupled to user policy; adding new tool types should model their safety posture against SafetyCheck so they integrate with existing gating. core/src/safety.rs:75
- Unified exec sessions reuse the portable PTY backend shared with streamable exec; improvements there automatically benefit both tool surfaces. core/src/unified_exec/mod.rs:151