# Backbone CLI Command Reference

## Schema Commands

### backbone schema validate
```bash
# Validate module schema
backbone schema validate <module>

# With verbose output
backbone schema validate <module> --verbose

# With warnings
backbone schema validate <module> --warnings

# Examples:
backbone schema validate sapiens
backbone schema validate postman --verbose
```

### backbone schema generate
```bash
# Generate all targets
backbone schema generate --target all <module>

# Generate specific targets
backbone schema generate --target rust,repository <module>

# Force overwrite
backbone schema generate --target all --force <module>

# Available targets:
# rust, sql, repository, repository-trait, grpc, handler, openapi
# service, domain-service, auth, permission, cqrs, events
# state-machine, workflow, trigger, validator, specification
# test, benchmark, config, module, value-object, computed, cli

# Examples:
backbone schema generate --target rust,repository,grpc sapiens
backbone schema generate --target all postman --force
```

### backbone schema watch
```bash
# Watch for schema changes
backbone schema watch <module>

# Examples:
backbone schema watch sapiens
# (Make changes in libs/modules/sapiens/schema/)
# (Auto-regeneration happens)
# Press Ctrl+C to stop
```

### backbone schema migration
```bash
# Create migration from schema changes
backbone schema migration <module>

# Force migration creation
backbone schema migration <module> --force

# Examples:
backbone schema migration sapiens
backbone schema migration bucket --force
```

## Module Commands

### backbone module create
```bash
# Create new module
backbone module create <module-name>

# Examples:
backbone module create payments
backbone module create analytics
```

### backbone module validate
```bash
# Validate module structure
backbone module validate <module>

# Examples:
backbone module validate sapiens
backbone module validate postman
```

### backbone module list
```bash
# List all modules
backbone module list

# With details
backbone module list --verbose
```

### backbone module info
```bash
# Show module information
backbone module info <module>

# Examples:
backbone module info sapiens
backbone module info postman
```

### backbone module deps
```bash
# Show module dependencies
backbone module deps <module>

# Show dependency graph
backbone module deps graph

# Examples:
backbone module deps sapiens
backbone module deps graph
```

## Framework Commands

### backbone version
```bash
# Show Backbone version
backbone version

# Detailed version info
backbone version --verbose
```

### backbone doctor
```bash
# Check framework health
backbone doctor

# Detailed check
backbone doctor --verbose
```

### backbone config
```bash
# List all configuration
backbone config --list

# Get specific configuration
backbone config get <key>

# Set configuration
backbone config set <key> <value>

# Examples:
backbone config get default.database
backbone config set default.database postgresql
```

## Generation Targets Reference

### Core Targets
- `rust` - Rust domain entities and value objects
- `sql` - Database table creation and migrations
- `repository` - PostgreSQL repository implementations
- `repository-trait` - Repository trait definitions
- `grpc` - gRPC service implementations
- `handler` - REST API handlers
- `openapi` - OpenAPI/Swagger specifications

### Service Layer Targets
- `service` - Application service implementations
- `domain-service` - Domain service interfaces
- `auth` - Authentication handlers
- `permission` - Authorization and permissions

### Advanced Pattern Targets
- `cqrs` - CQRS command/query separation
- `events` - Domain events and event sourcing
- `state-machine` - State machine implementations
- `workflow` - Business process orchestrations
- `trigger` - Database triggers and hooks

### Quality and Security Targets
- `validator` - Input validation logic
- `specification` - Business rule specifications
- `test` - Test code generation
- `benchmark` - Performance benchmarks

### Integration Targets
- `config` - Configuration files
- `module` - Module structure and lib.rs
- `value-object` - Value object implementations
- `computed` - Computed field implementations
- `cli` - CLI command implementations

## Common Flags and Options

### Global Options
```bash
--help, -h          # Show help
--verbose, -v       # Verbose output
--quiet, -q         # Quiet output
--version           # Show version
```

### Generate Command Options
```bash
--target <targets>  # Comma-separated list of targets
--force             # Force overwrite existing files
--dry-run           # Show what would be generated
--output <dir>      # Output directory
```

### Validate Command Options
```bash
--verbose, -v       # Detailed validation output
--warnings          # Show warnings as well as errors
--strict            # Strict validation mode
```

## Error Codes

### Schema Validation Errors
- `E001` - YAML syntax error
- `E002` - Invalid field type
- `E003` - Missing required field
- `E004` - Circular reference detected
- `E005` - Invalid relationship

### Generation Errors
- `G001` - Template not found
- `G002` - Permission denied
- `G003` - Invalid target
- `G004` - Compilation error in generated code

### Module Errors
- `M001` - Module already exists
- `M002` - Invalid module name
- `M003` - Module not found
- `M004` - Invalid module structure