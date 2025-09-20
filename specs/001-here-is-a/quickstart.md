# Quickstart: Custom Slash Commands

**Version**: 1.0.0
**Date**: 2025-01-16

This guide demonstrates how to create, test, and use custom slash commands in Codex CLI.

## Prerequisites

- Codex CLI built with `slash_commands` feature enabled
- Access to filesystem for creating template files
- Basic understanding of Markdown and YAML

## Quick Start (5 minutes)

### 1. Create Your First Command

Create the user commands directory:
```bash
mkdir -p ~/.codex/commands
```

Create a simple command template:
```bash
cat > ~/.codex/commands/explain.md << 'EOF'
---
description: "Explain a concept in simple terms"
argument-hint: "<concept>"
---

Please explain $1 in simple terms, as if you were teaching it to someone who is new to the topic. Use examples and analogies to make it clear.

Additional context: $ARGUMENTS
EOF
```

### 2. Test Command Discovery

Launch Codex CLI and check that your command is discovered:
```bash
codex
```

In the REPL:
```
/help
```

You should see your custom command listed:
- `/explain` (user) - Explain a concept in simple terms

### 3. Use Your Command

Execute your command with arguments:
```
/explain quantum computing
```

The system will inject the interpolated template as your message:
```
Please explain quantum computing in simple terms, as if you were teaching it to someone who is new to the topic. Use examples and analogies to make it clear.

Additional context: quantum computing
```

## Project Commands (10 minutes)

### 1. Create Project-Specific Commands

Create project commands directory in your current project:
```bash
mkdir -p .codex/commands
```

Create a project-specific command:
```bash
cat > .codex/commands/review.md << 'EOF'
---
description: "Code review with focus areas"
argument-hint: "<file-path> [focus-area]"
model: "gpt-4o"
---

Please review the code in $1 with special attention to:
- Code quality and best practices
- Security considerations
- Performance implications
- $2

Focus area: $2
EOF
```

### 2. Test Project Command

In Codex CLI:
```
/help
```

You should now see both commands:
- `/explain` (user) - Explain a concept in simple terms
- `/review` (project) - Code review with focus areas

### 3. Use Project Command with Model Override

```
/review src/main.rs error handling
```

This command will:
1. Use `gpt-4o` model for this turn only
2. Inject the interpolated template
3. Revert to your previous model after the response

## Advanced Features (15 minutes)

### 1. Namespaced Commands

Create organized command structure:
```bash
mkdir -p ~/.codex/commands/dev/git
mkdir -p ~/.codex/commands/writing
```

Create namespaced commands:
```bash
cat > ~/.codex/commands/dev/git/commit.md << 'EOF'
---
description: "Generate conventional commit message"
argument-hint: "<type> <description>"
---

Generate a conventional commit message for:
Type: $1
Description: $2

Please follow the conventional commits specification and include:
- Appropriate scope if relevant
- Clear description under 50 characters
- Body with explanation if needed

Changes to describe: $ARGUMENTS
EOF

cat > ~/.codex/commands/writing/blog.md << 'EOF'
---
description: "Blog post outline generator"
argument-hint: "<topic>"
---

Create a comprehensive blog post outline for: $1

Include:
- Compelling headline options
- Introduction hook
- Main sections with key points
- Conclusion strategy
- SEO considerations

Target audience and additional context: $ARGUMENTS
EOF
```

### 2. Test Namespaced Commands

In Codex CLI:
```
/help
```

You should see:
- `/user:dev:git:commit` (user) - Generate conventional commit message
- `/user:writing:blog` (user) - Blog post outline generator

Use short form if unambiguous:
```
/commit fix "resolve memory leak in parser"
```

Or use fully qualified names:
```
/user:dev:git:commit feat "add slash commands feature"
```

### 3. Handle Command Conflicts

Create conflicting command names:
```bash
cat > ~/.codex/commands/test.md << 'EOF'
---
description: "User test command"
---
This is a user test command.
EOF

cat > .codex/commands/test.md << 'EOF'
---
description: "Project test command"
---
This is a project test command.
EOF
```

Try to use the ambiguous command:
```
/test
```

You'll get a disambiguation prompt:
```
Ambiguous command 'test'. Please specify:
- /user:test (user) - User test command
- /project:test (project) - Project test command
```

Use qualified names:
```
/user:test
/project:test
```

### 4. Test Command Reload

After creating new commands, reload without restarting:
```
/help reload
```

This refreshes the command registry from the filesystem.

## Environment Customization

### 1. Custom Directory Locations

Set environment variables for custom command directories:
```bash
export CODEX_SLASH_COMMANDS_DIR_USER="/path/to/my/commands"
export CODEX_SLASH_COMMANDS_DIR_PROJECT="/path/to/project/commands"
```

### 2. Share Commands Across Projects

Create a shared commands directory:
```bash
mkdir -p ~/shared-commands
export CODEX_SLASH_COMMANDS_DIR_USER="~/shared-commands"
```

## Testing and Validation

### 1. Test Command Discovery
```bash
# Check command directories exist
ls -la ~/.codex/commands
ls -la .codex/commands

# Verify files are readable
cat ~/.codex/commands/explain.md
```

### 2. Test Template Syntax

Create a test command with various features:
```bash
cat > ~/.codex/commands/test-syntax.md << 'EOF'
---
description: "Test all template features"
argument-hint: "<arg1> <arg2> [optional]"
model: "gpt-4o"
---

Testing template interpolation:
- All arguments: $ARGUMENTS
- First argument: $1
- Second argument: $2
- Third argument: $3 (may be empty)

Special syntax (should be literal):
- Shell command: !echo "hello"
- File reference: @README.md

This tests all interpolation features.
EOF
```

Test the command:
```
/test-syntax hello world
```

Verify output shows:
- `$ARGUMENTS` replaced with "hello world"
- `$1` replaced with "hello"
- `$2` replaced with "world"
- `$3` is empty
- `!echo "hello"` appears literally
- `@README.md` appears literally

### 3. Test Error Handling

Test various error conditions:
```bash
# Create invalid frontmatter
cat > ~/.codex/commands/invalid.md << 'EOF'
---
invalid: yaml: content
---
This should work despite invalid YAML.
EOF

# Create empty file
touch ~/.codex/commands/empty.md

# Create file with no frontmatter
echo "Just markdown content" > ~/.codex/commands/plain.md
```

Reload and verify commands still work:
```
/help reload
/help
```

## Troubleshooting

### Commands Not Appearing

1. Check directory paths:
   ```bash
   echo $CODEX_SLASH_COMMANDS_DIR_USER
   echo $CODEX_SLASH_COMMANDS_DIR_PROJECT
   ```

2. Verify file permissions:
   ```bash
   ls -la ~/.codex/commands/
   ls -la .codex/commands/
   ```

3. Reload command registry:
   ```
   /help reload
   ```

### Template Not Interpolating

1. Verify syntax uses `$ARGUMENTS` and `$1`, `$2`, etc.
2. Check for typos in variable names
3. Test with simple template first

### Model Override Not Working

1. Verify model name in frontmatter is valid
2. Check that feature is enabled in your Codex build
3. Monitor session for model change and revert

### Command Conflicts

1. Use fully qualified names: `/user:name` or `/project:name`
2. Rename conflicting commands to be more specific
3. Organize with namespaces to avoid conflicts

## Next Steps

- Explore community command libraries compatible with Claude Code
- Create project-specific command collections for common workflows
- Share useful commands with your team using version control
- Integrate commands into your development workflow for maximum productivity

## File Organization Best Practices

```
~/.codex/commands/
├── dev/
│   ├── git/
│   │   ├── commit.md
│   │   └── branch.md
│   └── docker/
│       ├── build.md
│       └── deploy.md
├── writing/
│   ├── blog.md
│   ├── email.md
│   └── documentation.md
└── general/
    ├── explain.md
    └── summarize.md
```

This organization creates clear namespaces and prevents command naming conflicts.