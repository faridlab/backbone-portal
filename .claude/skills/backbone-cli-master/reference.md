# Reference Documentation for Backbone CLI Master

## Project Documentation References

### Core CLI Documentation
- **[FRAMEWORK.md](../../docs/technical/FRAMEWORK.md)** - Complete framework guide with CLI usage
- **[QUICKSTART.md](../../docs/technical/QUICKSTART.md)** - Quick start guide with CLI commands
- **[CLI/README.md](../../docs/cli/README.md)** - CLI tool documentation and commands
- **[CRATES_CLI_STATUS.md](../../docs/cli/CRATES_CLI_STATUS.md)** - CLI implementation status

### Schema and Code Generation
- **[SCHEMA/WORKFLOW.md](../../docs/schema/WORKFLOW.md)** - Schema development workflow
- **[SCHEMA/GENERATION.md](../../docs/schema/GENERATION.md)** - Code generation process
- **[SCHEMA/MIGRATION_GUIDE.md](../../docs/schema/MIGRATION_GUIDE.md)** - Migration workflows
- **[SCHEMA/IMPLEMENTATION_PLAN.md](../../docs/schema/IMPLEMENTATION_PLAN.md)** - Schema implementation details

### Module and Architecture
- **[MODULE_ECOSYSTEM.md](../../docs/technical/MODULE_ECOSYSTEM.md)** - Module structure and CLI patterns
- **[DDD_BOUNDED_CONTEXTS.md](../../docs/technical/DDD_BOUNDED_CONTEXTS.md)** - Module boundary definitions
- **[ARCHITECTURE_PATTERNS.md](../../docs/technical/ARCHITECTURE_PATTERNS.md)** - CLI architectural patterns

## CLI Command Categories

### Schema Commands (Core)
```bash
# Validation and Generation
backbone schema validate <module>
backbone schema generate --target <targets> <module>
backbone schema watch <module>
backbone schema parse <module>
backbone schema diff <module>

# Migration Management
backbone schema migration <module>
backbone schema migrate up|down <module>
backbone schema migration status <module>
```

### Module Management Commands
```bash
# Module Lifecycle
backbone module create <name>
backbone module validate <module>
backbone module list
backbone module info <module>
backbone module delete <module>

# Module Dependencies
backbone module deps <module>
backbone module deps graph
```

### Framework Commands
```bash
# Framework Information
backbone version
backbone doctor
backbone config --list
backbone config get <key>
backbone config set <key> <value>

# Framework Management
backbone init
backbone update
backbone clean
```

## Generation Targets Reference

### Core Targets (Essential)
```bash
--target rust              # Rust domain entities and value objects
--target sql              # Database table creation and migrations
--target repository        # PostgreSQL repository implementations
--target repository-trait  # Repository trait definitions
--target grpc             # gRPC service implementations
--target handler          # REST API handlers
--target openapi          # OpenAPI/Swagger specifications
```

### Service Layer Targets
```bash
--target service          # Application service implementations
--target domain-service   # Domain service interfaces
--target auth             # Authentication handlers
--target permission       # Authorization and permissions
```

### Advanced Pattern Targets
```bash
--target cqrs             # CQRS command/query separation
--target events           # Domain events and event sourcing
--target state-machine    # State machine implementations
--target workflow         # Business process orchestrations
--target trigger          # Database triggers and hooks
```

### Quality and Security Targets
```bash
--target validator        # Input validation logic
--target specification    # Business rule specifications
--target test             # Test code generation
--target benchmark        # Performance benchmarks
```

### Integration Targets
```bash
--target config           # Configuration files
--target module           # Module structure and lib.rs
--target value-object     # Value object implementations
--target computed         # Computed field implementations
--target cli              # CLI command implementations
```

## Workflow Patterns

### Schema Development Workflow
```bash
# 1. Validate current state
backbone schema validate <module>

# 2. Make schema changes (manual editing)
# Edit libs/modules/<module>/schema/models/*.yaml

# 3. Validate changes
backbone schema validate <module>

# 4. Generate code
backbone schema generate --target all <module>

# 5. Verify compilation
cargo check

# 6. Run tests
cargo test
```

### New Module Creation Workflow
```bash
# 1. Create module
backbone module create <module-name>

# 2. Validate structure
backbone module validate <module-name>

# 3. Create initial schemas
# Edit libs/modules/<module-name>/schema/

# 4. Generate initial code
backbone schema generate --target rust,repository <module-name>

# 5. Register module
# Edit apps/backbone/src/main.rs and Cargo.toml
```

### Production Deployment Workflow
```bash
# 1. Validate all modules
for module in $(ls libs/modules/); do
  backbone schema validate $module
done

# 2. Generate all code
for module in $(ls libs/modules/); do
  backbone schema generate --target all $module
done

# 3. Full build
cargo build --release

# 4. Run tests
cargo test
```

## Module-Specific Patterns

### Sapiens Module (User Management)
```bash
# Schema changes
backbone schema validate sapiens
backbone schema generate --target rust,repository,grpc sapiens

# Migration
backbone schema migration sapiens
backbone schema migrate up sapiens
```

### Postman Module (Email/Notifications)
```bash
# Schema changes
backbone schema validate postman
backbone schema generate --target service,handler postman

# Email templates
backbone schema generate --target template postman
```

### Bucket Module (File Storage)
```bash
# Schema changes
backbone schema validate bucket
backbone schema generate --target all bucket

# File operations
backbone schema generate --target storage,workflow bucket
```

## Error Handling and Recovery

### Validation Errors
```bash
# Check schema syntax
backbone schema validate <module> --verbose

# Parse schema for detailed errors
backbone schema parse <module>

# Check dependencies
backbone module deps <module>
```

### Generation Errors
```bash
# Clean generated files
rm -rf libs/modules/<module>/src/generated/

# Regenerate with force
backbone schema generate --target all --force <module>

# Check compilation errors
cargo check --message-format=short
```

### Migration Issues
```bash
# Check migration status
backbone schema migration status <module>

# Rollback migration
backbone schema migrate down <module>

# Create new migration
backbone schema migration <module> --force
```

## Integration with Development Tools

### VS Code Tasks
```json
{
  "tasks": [
    {
      "label": "Validate Schema",
      "type": "shell",
      "command": "backbone schema validate ${input:module}"
    },
    {
      "label": "Generate Code",
      "type": "shell",
      "command": "backbone schema generate --target all ${input:module}"
    },
    {
      "label": "Watch Development",
      "type": "shell",
      "command": "backbone schema watch ${input:module}"
    }
  ]
}
```

### Git Hooks
```bash
# Pre-commit validation
#!/bin/sh
for module in $(ls libs/modules/); do
  if [ -d "libs/modules/$module/schema" ]; then
    backbone schema validate $module
    if [ $? -ne 0 ]; then
      echo "Schema validation failed for $module"
      exit 1
    fi
  fi
done
```