# Data Model: Custom Slash Commands

**Date**: 2025-01-16
**Feature**: Custom Slash Commands for Codex CLI

## Core Entities

### Command
Represents a single custom slash command loaded from a Markdown template file.

**Fields**:
- `name`: String - Base command name (filename without .md extension)
- `scope`: CommandScope - Whether command is from user or project directory
- `namespaces`: Vec<String> - Hierarchical path from subdirectory structure
- `description`: Option<String> - User-friendly description from YAML frontmatter
- `argument_hint`: Option<String> - Usage hint for arguments from YAML frontmatter
- `model`: Option<String> - Model override for this command from YAML frontmatter
- `body`: String - Markdown content after frontmatter for interpolation
- `source_path`: PathBuf - Original file path for debugging/reload

**Validation Rules**:
- `name` must be valid ASCII identifier (alphanumeric, hyphen, underscore)
- `body` must be valid UTF-8 string
- `model` if present must be valid model identifier
- `source_path` must exist and be readable at load time

**State Transitions**:
- Created: When file is discovered and parsed successfully
- Cached: When added to command registry
- Executed: When invoked with arguments (temporary interpolation state)
- Reloaded: When file is re-read during cache refresh

### CommandScope
Enumeration indicating the origin directory of a command.

**Values**:
- `User`: Command from `~/.codex/commands` (or CODEX_SLASH_COMMANDS_DIR_USER)
- `Project`: Command from `.codex/commands` (or CODEX_SLASH_COMMANDS_DIR_PROJECT)

**Validation Rules**:
- Must be one of the two defined values
- Determines command precedence in conflict resolution

### CommandRegistry
In-memory index of all discovered commands providing fast lookup and metadata.

**Fields**:
- `commands`: HashMap<String, Command> - Fully qualified name to command mapping
- `user_commands`: HashMap<String, String> - Unqualified name to qualified name mapping for user scope
- `project_commands`: HashMap<String, String> - Unqualified name to qualified name mapping for project scope
- `last_loaded`: Option<SystemTime> - Timestamp of last registry refresh

**Validation Rules**:
- Fully qualified names must be unique across all scopes
- Unqualified names may conflict between scopes (disambiguation required)
- All commands must pass individual validation

**State Transitions**:
- Empty: Initial state before any command discovery
- Loading: During directory scanning and file parsing
- Ready: Commands loaded and available for lookup
- Reloading: During cache refresh operation

### InterpolationContext
Runtime data for template variable replacement during command execution.

**Fields**:
- `all_arguments`: String - Complete argument string after command name
- `positional_arguments`: Vec<String> - Individual arguments split by whitespace
- `command`: &Command - Reference to command being executed

**Validation Rules**:
- `all_arguments` preserves original spacing and quoting
- `positional_arguments` respects shell-like argument parsing
- No validation of argument content (user-provided data)

**State Transitions**:
- Created: When command is invoked with specific arguments
- Applied: During template interpolation process
- Consumed: After interpolated result is generated

### CommandNamespace
Hierarchical path structure derived from directory organization.

**Fields**:
- `components`: Vec<String> - Directory names from root to command file
- `scope`: CommandScope - Whether namespace is in user or project scope

**Validation Rules**:
- Directory names must be valid filesystem identifiers
- Maximum depth of 5 levels to prevent excessive nesting
- Each component must be valid ASCII identifier

**State Transitions**:
- Parsed: When directory structure is analyzed
- Qualified: When combined with scope and command name
- Resolved: When used for command lookup

### FrontmatterMetadata
Parsed YAML frontmatter data from command template files.

**Fields**:
- `description`: Option<String> - Human-readable command description
- `argument_hint`: Option<String> - Usage example or argument format
- `model`: Option<String> - Model name for single-turn override
- `allowed_tools`: Option<Vec<String>> - Parsed but ignored (security constraint)

**Validation Rules**:
- YAML must be valid if frontmatter block exists
- Unknown fields are ignored (forward compatibility)
- `allowed_tools` field has no effect (security requirement)
- Malformed YAML results in empty metadata (graceful degradation)

**State Transitions**:
- Raw: Frontmatter block extracted from file
- Parsed: YAML successfully deserialized
- Applied: Metadata used in Command creation

## Entity Relationships

### Command ↔ CommandScope
- Each Command has exactly one CommandScope
- CommandScope determines conflict resolution and display labeling

### Command ↔ CommandNamespace
- Each Command may have zero or more namespace components
- Namespace determines fully qualified command name

### CommandRegistry ↔ Command
- Registry contains all loaded Commands
- Provides multiple index structures for efficient lookup

### InterpolationContext ↔ Command
- Context is created for specific Command execution
- Temporary relationship during template processing

### Command ↔ FrontmatterMetadata
- Each Command may have associated metadata from file frontmatter
- Metadata enhances command with description, hints, and model override

## Data Flow

1. **Discovery**: Directory scanning creates Command entities from .md files
2. **Parsing**: FrontmatterMetadata extracted and validated for each Command
3. **Registration**: Commands added to CommandRegistry with computed qualified names
4. **Invocation**: User input creates InterpolationContext for specific Command
5. **Interpolation**: Context applied to Command body for template variable replacement
6. **Execution**: Interpolated result injected as user message in conversation

## Storage Strategy

- **Persistent**: Template files on filesystem (.codex/commands directories)
- **Memory**: CommandRegistry cache for fast access during session
- **Temporary**: InterpolationContext exists only during command execution
- **Configuration**: Directory paths via environment variables or defaults

## Error Handling

- **File Not Found**: Skip missing files during discovery, log warnings
- **Parse Errors**: Invalid YAML frontmatter ignored, use empty metadata
- **Invalid Names**: Skip commands with invalid identifiers, log warnings
- **Conflicts**: Ambiguous unqualified names require user disambiguation
- **Missing Arguments**: Positional placeholders become empty strings

## Performance Characteristics

- **Command Lookup**: O(1) via HashMap in CommandRegistry
- **Argument Interpolation**: O(n) where n is template body length
- **Directory Scanning**: O(m) where m is number of .md files
- **Memory Usage**: Proportional to number of commands and average template size