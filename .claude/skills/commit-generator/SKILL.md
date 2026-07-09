---
name: commit-generator
description: Generating intelligent, context-aware commit messages for the Backbone Framework codebase. You specialize in analyzing code changes and following conventional commit standards while maintaining high-quality commit history. commit the changes, grouping by its functionality use one-line short message. dont create big files in one commit. and dont add any signature. NEVER add Co-Authored-By or any other signature.
---

## Core Responsibilities

### 🎯 Intelligent Commit Analysis
- Analyze code changes to understand impact and scope
- Detect breaking changes, new features, and bug fixes automatically
- Identify affected modules, files, and cross-dependencies
- Assess performance impact and technical complexity

### 🔧 Conventional Commit Standards
- Follow established conventional commit format (type/scope: description)
- Generate meaningful commit subjects that clearly communicate changes
- Include issue/ticket references when available
- Mark breaking changes and deprecations appropriately

### 🚀 Integration and Automation
- Generate changelog entries and release notes automatically
- Link commits to GitHub issues, JIRA tickets, or documentation
- Maintain consistent commit standards across development team
- Automate release preparation and version tagging processes

## Verified Environment

### Backbone Framework Commit Standards
- **Repository Structure**: apps/backbone/, libs/modules/, libs/crates/, docs/
- **Module Boundaries**: libs/modules/{module}/ with clear boundaries
- **Code Generation**: Backbone schema changes affecting multiple files
- **Issue Tracking**: GitHub issues, JIRA tickets for project management

## Commit Analysis Capabilities

### File Impact Analysis
```bash
# Analyze changed files and their impact
git diff --name-only
git diff --stat
git log --oneline -5

# Determine change types
- Modified schema files → schema changes
- Generated Rust code → generation artifacts
- Test files → test updates
- Documentation → documentation changes
```

### Change Type Detection
```yaml
commit_categories:
  feature:
    description: "New functionality or capabilities"
    prefixes: ["feat:", "feature:"]
    examples: ["Add user authentication", "Implement payment processing"]

  fix:
    description: "Bug fixes and corrections"
    prefixes: ["fix:", "bugfix:"]
    examples: ["Fix validation error", "Correct database migration"]

  chore:
    description: "Maintenance, refactoring, build changes"
    prefixes: ["chore:", "refactor:", "build:"]
    examples: ["Update dependencies", "Refactor user entity", "Add tests"]

  docs:
    description: "Documentation updates and improvements"
    prefixes: ["docs:", "documentation:"]
    examples: ["Update README", "Add API documentation"]

  perf:
    description: "Performance improvements and optimizations"
    prefixes: ["perf:", "performance:", "optimize:"]
    examples: ["Optimize database queries", "Improve response time"]

  breaking:
    description: "Breaking changes that require migration"
    prefixes: ["feat!:", "fix!:", "BREAKING CHANGE:"]
    examples: ["Change user entity interface", "Remove deprecated API"]
```

## Commit Generation Workflows

### 1. Feature Development Commit
```bash
# Analyze changes
git status
git diff --name-only

# Generate commit message with context
# Command: `git commit -m "$(generate_commit_message)"`
```

### 2. Bug Fix Commit
```bash
# Analyze bug fix changes
git diff --stat
git log --grep "fix" --oneline -3

# Generate bug fix commit with issue reference
# Command: `git commit -m "fix(auth): Resolve login validation error for edge cases"`
```

### 3. Refactoring Commit
```bash
# Analyze refactoring impact
git diff --stat
git show --name-status HEAD~1

# Generate refactoring commit with scope
# Command: `git commit -m "refactor(user): Simplify user entity validation logic"`
```

## Commit Message Generation Standards

### One-Line Short Messages
```bash
# Format: type(scope): brief-description

# Examples:
feat(sapiens): Add phone verification to user entity
fix(auth): Resolve login validation error for edge cases
refactor(user): Simplify user entity validation logic
docs(api): Update authentication endpoint documentation
chore(deps): Update actix-web to version 4.5
perf(database): Optimize user query performance
```

### Grouping by Functionality
```bash
# Schema changes:
feat(sapiens): Add user profile and preferences
feat(bucket): Implement file sharing with permissions
feat(postman): Add email template variables

# Bug fixes:
fix(auth): Resolve login validation error
fix(database): Fix migration rollback issue
fix(api): Correct response status codes

# Refactoring:
refactor(user): Extract validation to service layer
refactor(schema): Simplify model inheritance
refactor(tests): Consolidate test utilities

# Documentation:
docs(readme): Update setup instructions
docs(api): Add authentication examples
docs(migration): Guide for user entity changes

# Maintenance:
chore(deps): Update rust dependencies
chore(build): Improve docker build speed
chore(test): Add e2e test coverage
```

### No Claude Signatures
```bash
# ❌ WRONG - Include signatures
git commit -m "feat(sapiens): Add user profile

🤖 Generated with Claude Code
Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"

# ✅ CORRECT - Clean commit messages
git commit -m "feat(sapiens): Add user profile with preferences and avatar"

# ✅ CORRECT - Simple issue reference
git commit -m "fix(auth): Resolve login validation edge case

Fixes #456"
```

## Integration Patterns

### ✅ Encouraged Behaviors
- Generate one-line commit messages with clear scope and brief description
- Group related changes by functionality (schema, fixes, refactoring, docs, maintenance)
- Use conventional commit format: type(scope): description
- Link to issues with simple references (Fixes #123, Closes #456)
- Focus on what changed, not how or why (save details for PR description)

### ❌ Avoid Behaviors
- Adding Claude signatures or attribution in commit messages
- Writing long, multi-line commit messages with extensive details
- Including explanations of "how" the change was implemented
- Adding personal commentary or development notes to commits
- Using generic messages without analyzing the actual changes

## Automation and Tools

### Rust Commit Analyzer Tool
```bash
# Build the Rust binary
cd .claude/skills/commit-generator/scripts
./build.sh

# Use the analyzer
./commit-analyzer                    # Show suggested commit message
./commit-analyzer --verbose          # Show detailed analysis
./commit-analyzer --quiet            # Show only commit message

# From repo root
./commit-analyzer --verbose --repo-path .
```

### Git Hooks Integration
```bash
# .git/hooks/prepare-commit-msg
#!/bin/sh

# Use Rust commit analyzer for improved messages
COMMIT_ANALYZER="$(git rev-parse --show-toplevel)/commit-analyzer"

if [ -f "$COMMIT_ANALYZER" ]; then
    echo "Using Backbone commit analyzer..."
    MESSAGE=$($COMMIT_ANALYZER --quiet)
    if [ -n "$MESSAGE" ] && [ "$MESSAGE" != "No staged changes found" ]; then
        echo "$MESSAGE" > "$1"
    fi
else
    echo "Commit analyzer not found, using standard validation"
fi
```

### VS Code Integration
```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Analyze Commit Changes",
      "type": "shell",
      "command": "./commit-analyzer",
      "args": ["--verbose"],
      "group": "build",
      "presentation": {
        "reveal": "always",
        "panel": "new"
      }
    },
    {
      "label": "Generate Commit Message",
      "type": "shell",
      "command": "./commit-analyzer",
      "args": ["--quiet"],
      "group": "build"
    }
  ]
}
```

## Examples

### Example 1: Schema Change Commit
**User**: "I've updated the User schema to add profile information"

**Generated commit**:
```bash
git commit -m "feat(sapiens): Add user profile with preferences and avatar"
```

### Example 2: Bug Fix with Issue Reference
**User**: "I fixed the login validation error users were reporting"

**Generated commit**:
```bash
git commit -m "fix(auth): Resolve login validation edge case

Fixes #456"
```

### Example 3: Multiple Related Changes
**Changes**: Updated user entity, added validation, and wrote tests

**Generated commit**:
```bash
git commit -m "feat(sapiens): Add user validation with comprehensive tests"
```

### Example 4: Documentation Update
**Changes**: Updated README and added API examples

**Generated commit**:
```bash
git commit -m "docs(readme): Update setup instructions and API examples"
```

## Guidelines

- **ONE-LINE FOCUS**: Keep commit messages concise and focused on what changed
- **CONVENTIONAL FORMAT**: Follow type(scope): description format consistently
- **FUNCTIONAL GROUPING**: Group related changes by their primary functionality
- **SIMPLE REFERENCES**: Use simple issue references (Fixes #123) without attribution
- **NO SIGNATURES**: NEVER add Claude signatures, Co-Authored-By, or any attribution to commits
- **CLEAR SCOPE**: Use descriptive scope that accurately reflects change boundaries

## Integration

Works closely with:
- **All Development Team Members**: Provides high-quality commit messages for their changes
- **Schema Maintainer**: Provides context for schema-related commits
- **Project Managers**: Links commits to tickets and milestones
- **Release Managers**: Generates changelog entries and release notes
- **Code Reviewers**: Provides context for understanding commit impact and scope