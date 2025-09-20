# Research: Custom Slash Commands Implementation

**Date**: 2025-01-16
**Feature**: Custom Slash Commands for Codex CLI

## Research Overview

Since the Technical Context had no NEEDS CLARIFICATION items, this research focuses on existing codebase patterns and best practices for the surgical implementation approach.

## Existing Codebase Integration Points

### Decision: REPL Input Interception
**Rationale**: Minimal intervention strategy requires single hook point in existing input processing
**Research Findings**:
- Codex CLI already supports slash commands (built-in commands)
- Input processing likely in `tui` or `core` crates
- Need to identify existing slash command handling mechanism
**Alternatives Considered**:
- Modify multiple input points: Rejected (violates surgical approach)
- Replace existing command system: Rejected (upstream merge incompatible)

### Decision: Rust Workspace Crate Structure
**Rationale**: Follows existing `codex-*` naming convention and isolation principles
**Research Findings**:
- Existing crates: `codex-core`, `codex-tui`, `codex-cli`, etc.
- Each crate has clear purpose and dependencies
- Cargo workspace allows independent testing
**Alternatives Considered**:
- Add to existing `core` crate: Rejected (violates single purpose principle)
- Standalone binary: Rejected (integration complexity)

### Decision: YAML Frontmatter Parsing
**Rationale**: Matches Claude Code format for community compatibility
**Research Findings**:
- `serde_yaml` crate is standard for YAML parsing in Rust
- Frontmatter extraction pattern: split on `---` delimiters
- Error handling: malformed YAML should be ignored per requirements
**Alternatives Considered**:
- TOML frontmatter: Rejected (incompatible with Claude Code)
- JSON frontmatter: Rejected (less readable for users)

### Decision: Feature Flag Implementation
**Rationale**: Keeps default binary unchanged for upstream compatibility
**Research Findings**:
- Cargo features are standard Rust approach
- Conditional compilation with `#[cfg(feature = "slash_commands")]`
- Default features should exclude this for upstream safety
**Alternatives Considered**:
- Runtime configuration: Rejected (always adds binary size)
- Separate binary: Rejected (user experience complexity)

### Decision: File System Directory Scanning
**Rationale**: Cross-platform compatibility with environment variable overrides
**Research Findings**:
- Standard library `std::fs::read_dir` for recursive scanning
- `dirs` crate for home directory resolution
- Environment variable precedence pattern
**Alternatives Considered**:
- File watching: Rejected (complexity, resource usage)
- Database storage: Rejected (adds dependency, violates simplicity)

### Decision: In-Memory Command Registry
**Rationale**: Fast lookup performance with manual reload capability
**Research Findings**:
- `HashMap<String, Command>` for O(1) lookup
- Load on first use pattern to avoid startup impact
- `/help reload` command for manual cache invalidation
**Alternatives Considered**:
- File-based caching: Rejected (adds complexity)
- Automatic file watching: Rejected (resource usage)

### Decision: Argument Interpolation Strategy
**Rationale**: Simple string replacement with deterministic rules
**Research Findings**:
- Regex-based replacement for `$ARGUMENTS` and `$1`, `$2`, etc.
- Handle missing positional arguments gracefully (empty string)
- Preserve escaped sequences if needed
**Alternatives Considered**:
- Template engine (handlebars, tera): Rejected (adds complexity)
- Custom parser: Rejected (regex sufficient for simple case)

### Decision: Model Override Integration
**Rationale**: Temporary session state change with automatic revert
**Research Findings**:
- Need to integrate with existing model management in `codex-core`
- Session state should track previous model for revert
- One-turn scope requires careful integration with conversation flow
**Alternatives Considered**:
- Persistent model change: Rejected (user experience issue)
- No model override: Rejected (requirement from spec)

### Decision: TUI Integration Approach
**Rationale**: Minimal changes to existing TUI while adding functionality
**Research Findings**:
- Existing help system in TUI needs extension
- Tab completion system needs command registry integration
- Maintain existing styling and interaction patterns
**Alternatives Considered**:
- Separate TUI for commands: Rejected (user experience fragmentation)
- No TUI changes: Rejected (missing help/completion requirements)

## Security and Safety Research

### Decision: No Shell Execution Path
**Rationale**: Explicit security requirement to prevent template-based code execution
**Research Findings**:
- `!command` syntax must be treated as literal text
- No access to `std::process::Command` or similar
- Template processing must be pure string manipulation
**Alternatives Considered**: None - this is a hard security requirement

### Decision: No File Inclusion Path
**Rationale**: Prevents large context injection and unauthorized file access
**Research Findings**:
- `@file` syntax must be treated as literal text
- No file reading beyond initial template discovery
- Template content is self-contained
**Alternatives Considered**: None - this is a hard security requirement

## Performance Considerations

### Decision: Lazy Loading with Caching
**Rationale**: Minimize impact on CLI startup time while providing fast command access
**Research Findings**:
- Registry loading only when first slash command is used
- Memory overhead acceptable for hundreds of commands
- Cache invalidation via explicit reload command
**Alternatives Considered**:
- Eager loading: Rejected (startup impact)
- No caching: Rejected (performance impact)

## Cross-Platform Compatibility

### Decision: Standard Library + dirs Crate
**Rationale**: Minimal dependencies with proven cross-platform support
**Research Findings**:
- `dirs::home_dir()` handles platform differences
- Environment variable override provides flexibility
- Standard library filesystem APIs work on all target platforms
**Alternatives Considered**:
- Platform-specific implementations: Rejected (maintenance burden)
- Additional platform abstraction crates: Rejected (unnecessary dependency)

## Testing Strategy Research

### Decision: Unit + Integration + Snapshot Testing
**Rationale**: Comprehensive coverage following existing codebase patterns
**Research Findings**:
- Unit tests for command registry, parsing, interpolation
- Integration tests for end-to-end command execution
- Snapshot tests for TUI changes using `cargo insta`
**Alternatives Considered**:
- Manual testing only: Rejected (violates constitution)
- Unit tests only: Rejected (insufficient coverage)

## Conclusion

All research supports the surgical implementation approach with minimal codebase intervention. The proposed architecture maintains upstream merge compatibility while providing full feature functionality as specified.