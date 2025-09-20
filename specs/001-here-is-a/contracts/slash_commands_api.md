# Slash Commands API Contract

**Version**: 1.0.0
**Date**: 2025-01-16

## Overview

This document defines the internal API contract for the custom slash commands feature. Since this is a CLI extension rather than a web service, these are function signatures and data contracts for the Rust implementation.

## Core API Functions

### CommandRegistry API

#### `fn discover_commands() -> Result<CommandRegistry, DiscoveryError>`
**Purpose**: Scan configured directories and build command registry
**Input**: None (uses environment variables and defaults)
**Output**:
- `Ok(CommandRegistry)`: Successfully loaded command registry
- `Err(DiscoveryError)`: Directory access or parsing failures

**Contract**:
- MUST scan both user and project command directories
- MUST handle missing directories gracefully
- MUST skip invalid files and continue processing
- MUST build fully qualified name mappings

#### `fn reload_commands(&mut self) -> Result<usize, ReloadError>`
**Purpose**: Refresh command registry from filesystem
**Input**: Mutable reference to existing registry
**Output**:
- `Ok(usize)`: Number of commands loaded
- `Err(ReloadError)`: Directory access failures

**Contract**:
- MUST clear existing registry before reload
- MUST preserve timestamp of reload operation
- MUST handle concurrent access safely

#### `fn lookup_command(&self, name: &str) -> CommandLookupResult`
**Purpose**: Find command by name (qualified or unqualified)
**Input**: Command name string
**Output**: Enum with lookup result

**Contract**:
- MUST return exact match for fully qualified names
- MUST detect ambiguous unqualified names
- MUST suggest alternatives for unknown commands

### Command Parsing API

#### `fn parse_command_file(path: &Path) -> Result<Command, ParseError>`
**Purpose**: Parse individual command template file
**Input**: File path to .md template
**Output**:
- `Ok(Command)`: Successfully parsed command
- `Err(ParseError)`: File access or format errors

**Contract**:
- MUST extract YAML frontmatter if present
- MUST handle malformed frontmatter gracefully
- MUST preserve original template body
- MUST compute namespaces from file path

#### `fn extract_frontmatter(content: &str) -> (Option<FrontmatterMetadata>, String)`
**Purpose**: Separate YAML frontmatter from markdown body
**Input**: Raw file content
**Output**: Tuple of optional metadata and body content

**Contract**:
- MUST detect frontmatter delimited by `---` lines
- MUST return None for malformed YAML
- MUST preserve all content after frontmatter as body

### Interpolation API

#### `fn interpolate_template(command: &Command, args: &[String]) -> String`
**Purpose**: Replace template variables with argument values
**Input**: Command template and argument list
**Output**: Interpolated template string

**Contract**:
- MUST replace `$ARGUMENTS` with full argument string
- MUST replace `$1`, `$2`, etc. with positional arguments
- MUST use empty string for missing positional arguments
- MUST preserve all other content unchanged

#### `fn parse_command_line(input: &str) -> Option<(String, Vec<String>)>`
**Purpose**: Parse user input into command name and arguments
**Input**: Raw command line input
**Output**: Optional tuple of command name and arguments

**Contract**:
- MUST recognize slash-prefixed commands
- MUST handle quoted arguments correctly
- MUST return None for non-command input

### Integration API

#### `fn execute_slash_command(input: &str, registry: &CommandRegistry) -> CommandExecutionResult`
**Purpose**: Main entry point for slash command processing
**Input**: User input string and command registry
**Output**: Execution result enum

**Contract**:
- MUST check if input is a slash command
- MUST perform command lookup and validation
- MUST handle model override if specified
- MUST return interpolated content for injection

#### `fn extend_help_output(commands: &CommandRegistry) -> Vec<HelpEntry>`
**Purpose**: Generate help entries for custom commands
**Input**: Command registry reference
**Output**: List of help entries with descriptions

**Contract**:
- MUST include scope labels for all commands
- MUST sort commands alphabetically within scopes
- MUST include argument hints when available

## Data Type Contracts

### CommandLookupResult
```rust
enum CommandLookupResult {
    Found(Command),
    Ambiguous { user: Option<Command>, project: Option<Command> },
    NotFound { suggestions: Vec<String> },
}
```

### CommandExecutionResult
```rust
enum CommandExecutionResult {
    Success {
        content: String,
        model_override: Option<String>
    },
    NotACommand,
    CommandNotFound {
        name: String,
        suggestions: Vec<String>
    },
    AmbiguousCommand {
        name: String,
        qualified_options: Vec<String>
    },
    ExecutionError(String),
}
```

### DiscoveryError
```rust
enum DiscoveryError {
    DirectoryAccess { path: PathBuf, error: io::Error },
    PermissionDenied { path: PathBuf },
    TooManyCommands { limit: usize },
}
```

### ParseError
```rust
enum ParseError {
    FileNotFound(PathBuf),
    InvalidUtf8(PathBuf),
    InvalidCommandName(String),
    NamespaceTooDeep { path: PathBuf, depth: usize },
}
```

## Environment Integration

### Environment Variables
- `CODEX_SLASH_COMMANDS_DIR_USER`: Override default user commands directory
- `CODEX_SLASH_COMMANDS_DIR_PROJECT`: Override default project commands directory

### Directory Defaults
- User: `~/.codex/commands`
- Project: `.codex/commands` (relative to current working directory)

## Error Handling Requirements

### File System Errors
- Missing directories: Log warning, continue with empty registry
- Permission denied: Log error, skip inaccessible directories
- Corrupt files: Log warning, skip individual files

### Parsing Errors
- Invalid YAML frontmatter: Use empty metadata, preserve body
- Invalid command names: Skip file, log warning with path
- Namespace too deep: Skip file, log warning about limit

### Runtime Errors
- Command not found: Provide helpful suggestions
- Ambiguous commands: List qualified options for disambiguation
- Model override failures: Log warning, continue without override

## Performance Requirements

### Response Time
- Command lookup: < 1ms for registry of 1000 commands
- Template interpolation: < 10ms for templates up to 10KB
- Directory scanning: < 100ms for directories with 100 files

### Memory Usage
- Registry overhead: < 1MB for 1000 commands
- Template caching: Proportional to file sizes
- No memory leaks during reload operations

### Concurrency
- Registry reads: Thread-safe immutable access
- Registry reloads: Exclusive write access with minimal blocking
- No shared mutable state between command executions

## Integration Points

### TUI Integration
- Help system extension for custom command display
- Tab completion integration for command name suggestions
- Error message display for failed command execution

### Core Integration
- Model override integration with session management
- Message injection into conversation flow
- Preservation of existing REPL functionality

### Configuration Integration
- Environment variable reading with precedence rules
- Graceful fallback to defaults when overrides invalid
- No persistent configuration storage required