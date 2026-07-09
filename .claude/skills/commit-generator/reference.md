# Reference Documentation for Commit Generator

## Project Documentation References

### Core Framework Documentation
- **[FRAMEWORK.md](../../docs/technical/FRAMEWORK.md)** - Complete framework guide and architecture
- **[QUICKSTART.md](../../docs/technical/QUICKSTART.md)** - Quick start guide for development
- **[API_GUIDELINES.md](../../docs/technical/API_GUIDELINES.md)** - API design standards and patterns

### Commit Standards
- **[ARCHITECTURE_PATTERNS.md](../../docs/technical/ARCHITECTURE_PATTERNS.md)** - Development patterns and conventions
- **[SELF_DOCUMENTATION_STANDARDS.md](../../docs/technical/SELF_DOCUMENTATION_STANDARDS.md)** - Documentation practices
- **[MANUAL_LOGIC_GUIDE.md](../../docs/technical/MANUAL_LOGIC_GUIDE.md)** - Code generation and manual logic patterns

### Module Structure
- **[MODULE_ECOSYSTEM.md](../../docs/technical/MODULE_ECOSYSTEM.md)** - Module architecture and boundaries
- **[DDD_BOUNDED_CONTEXTS.md](../../docs/technical/DDD_BOUNDED_CONTEXTS.md)** - Domain-driven design patterns
- **[MODULAR_MONOLITH_GUIDE.md](../../docs/MODULAR_MONOLITH_GUIDE.md)** - Modular monolith architecture

### Schema and Code Generation
- **[SCHEMA/WORKFLOW.md](../../docs/schema/WORKFLOW.md)** - Schema development workflow
- **[SCHEMA/GENERATION.md](../../docs/schema/GENERATION.md)** - Code generation patterns
- **[SCHEMA/MIGRATION_GUIDE.md](../../docs/schema/MIGRATION_GUIDE.md)** - Migration practices

## Commit Types Reference

### Conventional Commit Specification
Based on [Conventional Commits v1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)

#### Format
```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

#### Types Used in Backbone Framework
- **feat**: New functionality or features
- **fix**: Bug fixes or corrections
- **docs**: Documentation changes
- **style**: Code style changes (formatting, etc.)
- **refactor**: Code refactoring without functional changes
- **perf**: Performance improvements
- **test**: Adding or updating tests
- **chore**: Maintenance tasks, dependency updates
- **build**: Build system or dependency changes

#### Scopes in Backbone Framework
- **sapiens**: User management module
- **postman**: Email/notification module
- **bucket**: File storage module
- **backbone**: Core framework changes
- **schema**: Schema-related changes
- **cli**: Command-line interface changes
- **docs**: Documentation updates
- **deps**: Dependency updates

## File Type to Commit Type Mapping

### Schema Changes (.model.yaml, .hook.yaml, .workflow.yaml)
```bash
# New entities or major schema changes
feat(sapiens): Add user profile entity
feat(bucket): Implement file sharing workflow

# Schema validation or fixes
fix(schema): Resolve user entity validation error
fix(bucket): Fix file upload schema constraint
```

### Generated Code (src/generated/, repository files)
```bash
# Usually part of schema changes - avoid separate commits
# If manual changes to generated code:
refactor(user): Enhance generated repository with custom queries
perf(database): Optimize generated user queries with indexes
```

### Application Code (src/application/, src/domain/, src/infrastructure/)
```bash
# New business logic
feat(sapiens): Implement user authentication service
feat(postman): Add email template rendering engine

# Bug fixes
fix(auth): Resolve session timeout issue
fix(email): Fix SMTP connection timeout handling

# Refactoring
refactor(user): Extract validation to dedicated service
refactor(payment): Simplify transaction handling logic
```

### Test Files (tests/, *_test.rs)
```bash
# New test coverage
test(sapiens): Add comprehensive user authentication tests
test(api): Add integration tests for payment endpoints

# Test fixes or improvements
test(auth): Fix flaky login test under concurrent load
refactor(tests): Consolidate user test utilities
```

### Configuration (.yml, .toml, .env)
```bash
# New features requiring config
feat(config): Add Redis configuration for caching
feat(deployment): Configure production environment variables

# Configuration fixes
fix(config): Resolve database connection timeout
chore(config): Update default development settings
```

### Documentation (docs/, README.md, *.md)
```bash
# New documentation
docs(readme): Add quick start guide for developers
docs(api): Document authentication endpoints

# Documentation updates
docs(api): Update authentication examples
docs(schema): Document new user entity fields
```

## Issue Reference Patterns

### GitHub Issues
```bash
Fixes #123
Closes #456
Resolves #789
```

### JIRA Tickets (if used)
```bash
BACK-123
PROJ-456
```

### Breaking Changes
```bash
feat!(sapiens): Redesign user authentication system

BREAKING CHANGE: User authentication flow has been redesigned.
Previous token-based authentication is deprecated in favor of JWT.
Migration guide available in docs/migrations/auth-v2.md
```

## Module-Specific Patterns

### Sapiens Module (User Management)
```bash
feat(sapiens): Add multi-factor authentication
fix(sapiens): Resolve password reset email delivery
refactor(sapiens): Simplify user role assignment logic
docs(sapiens): Document MFA setup process
```

### Postman Module (Email/Notifications)
```bash
feat(postman): Add email template variables
fix(postman): Resolve SMTP authentication failure
perf(postman): Optimize email queue processing
docs(postman): Document template engine usage
```

### Bucket Module (File Storage)
```bash
feat(bucket): Implement file versioning
fix(bucket): Resolve large file upload timeout
perf(bucket): Optimize image thumbnail generation
docs(bucket): Document file sharing permissions
```

### Backbone Core Framework
```bash
feat(backbone): Add generic caching layer
fix(backbone): Resolve repository transaction handling
refactor(backbone): Simplify module registration API
docs(backbone): Update framework migration guide
```

## Generated vs Manual Code Patterns

### Generated Code (from schema)
```bash
# These should typically be part of schema commits
feat(sapiens): Add user profile with validation
# (Includes generated entities, repositories, handlers)

# Avoid separate commits like:
# feat(sapiens): Generate user repository (WRONG)
# chore(sapiens): Update generated user entity (WRONG)
```

### Manual Code Logic
```bash
# Custom business logic added to generated code
feat(sapiens): Add user profile image processing
feat(auth): Implement session validation middleware

# Performance optimizations
perf(user): Optimize user query with database indexes
perf(email): Batch email sending for notifications
```