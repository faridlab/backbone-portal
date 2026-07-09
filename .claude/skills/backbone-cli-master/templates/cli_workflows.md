# Backbone CLI Workflow Templates

## Development Workflows

### Schema Development Workflow
```bash
# Template: Standard schema development
# Replace <module> with actual module name

# 1. Validation
backbone schema validate <module>

# 2. Make changes (edit schema files)
# vim libs/modules/<module>/schema/models/<entity>.model.yaml

# 3. Validate changes
backbone schema validate <module>

# 4. Generate code
backbone schema generate --target rust,repository <module>

# 5. Compile check
cargo check

# 6. Run tests
cargo test --package backbone-<module>
```

### Rapid Development Workflow
```bash
# Template: Quick iteration during development
# Use for rapid schema changes

backbone schema watch <module>
# (Make schema changes in another terminal)
# (Auto-regeneration happens)
# Ctrl+C to stop

# Final validation and generation
backbone schema validate <module>
backbone schema generate --target all <module>
```

### Full Feature Development Workflow
```bash
# Template: Complete feature development
# Use when adding new entities or major changes

# 1. Schema design and validation
backbone schema validate <module>

# 2. Generate all layers
backbone schema generate --target all <module>

# 3. Add custom business logic
# Edit generated files in libs/modules/<module>/src/

# 4. Build and test
cargo build
cargo test --package backbone-<module>

# 5. Integration testing
cargo run --bin backbone -- <module>-test
```

## Module Management Workflows

### New Module Creation
```bash
# Template: Create new module from scratch
# Replace <module-name> with desired module name

# 1. Create module structure
backbone module create <module-name>

# 2. Validate structure
backbone module validate <module-name>

# 3. Create initial schema
# mkdir -p libs/modules/<module-name>/schema/models/
# vim libs/modules/<module-name>/schema/models/<entity>.model.yaml

# 4. Validate schema
backbone schema validate <module-name>

# 5. Generate initial code
backbone schema generate --target rust,repository <module-name>

# 6. Register in main application
# Edit apps/backbone/Cargo.toml
# Edit apps/backbone/src/main.rs
```

### Module Enhancement Workflow
```bash
# Template: Enhance existing module
# Use for adding new features to existing modules

# 1. Analyze current state
backbone module info <module>
backbone schema diff <module>

# 2. Make schema changes
# Edit schema files

# 3. Validate and generate
backbone schema validate <module>
backbone schema generate --target all <module>

# 4. Update dependencies if needed
backbone module deps <module>

# 5. Test integration
cargo test --package backbone-<module>
```

## Production Workflows

### Production Release Workflow
```bash
# Template: Prepare for production release
# Ensures all modules are ready for deployment

# 1. Validate all schemas
for module in $(ls libs/modules/); do
  if [ -d "libs/modules/$module/schema" ]; then
    echo "Validating $module..."
    backbone schema validate $module || exit 1
  fi
done

# 2. Generate all code
for module in $(ls libs/modules/); do
  if [ -d "libs/modules/$module/schema" ]; then
    echo "Generating $module..."
    backbone schema generate --target all $module || exit 1
  fi
done

# 3. Full build
cargo build --release || exit 1

# 4. Run test suite
cargo test || exit 1

# 5. Check for issues
backbone doctor || exit 1

echo "Production preparation complete!"
```

### Migration Workflow
```bash
# Template: Database migration for production
# Use when applying schema changes to production database

# 1. Generate migration
backbone schema migration <module>

# 2. Review migration
cat libs/modules/<module>/migrations/*.sql

# 3. Test migration on staging
# DATABASE_URL="staging_db_url" backbone schema migrate up <module>

# 4. Backup production database
# pg_dump production_db > backup_$(date +%Y%m%d_%H%M%S).sql

# 5. Apply to production
# DATABASE_URL="production_db_url" backbone schema migrate up <module>

# 6. Verify migration
backbone schema migration status <module>
```

## Troubleshooting Workflows

### Schema Validation Issues
```bash
# Template: Debug schema validation problems
# Use when schema validation fails

# 1. Detailed validation
backbone schema validate <module> --verbose

# 2. Parse schema for syntax errors
backbone schema parse <module>

# 3. Check dependencies
backbone module deps <module>

# 4. Validate referenced modules
backbone module validate <module>

# 5. Common fixes:
# - Check YAML syntax
# - Verify field types and constraints
# - Check circular references
# - Validate import paths
```

### Generation Errors
```bash
# Template: Fix code generation issues
# Use when code generation fails or produces errors

# 1. Clean generated files
rm -rf libs/modules/<module>/src/generated/

# 2. Force regeneration
backbone schema generate --target all --force <module>

# 3. Check compilation errors
cargo check --message-format=short 2>&1 | head -20

# 4. Fix common issues:
# - Missing imports in generated files
# - Conflicting trait implementations
# - Incorrect type references
# - Missing dependencies in Cargo.toml
```

### Compilation Issues
```bash
# Template: Resolve compilation errors after generation
# Use when generated code doesn't compile

# 1. Check compilation output
cargo check --message-format=short

# 2. Identify error patterns:
# - Missing trait implementations
# - Type mismatches
# - Import errors
# - Dependency conflicts

# 3. Common solutions:
# - Update dependencies in Cargo.toml
# - Add missing trait implementations manually
# - Fix type definitions in schema
# - Regenerate with --force flag
```

## Optimization Workflows

### Development Optimization
```bash
# Template: Optimized development workflow
# For faster iteration during development

# 1. Watch mode for continuous development
backbone schema watch <module> &
WATCH_PID=$!

# 2. Parallel compilation
cargo check --jobs $(nproc)

# 3. Run tests in parallel
cargo test --package backbone-<module> --jobs $(nproc)

# 4. Stop watch when done
kill $WATCH_PID
```

### Performance Testing Workflow
```bash
# Template: Performance testing of generated code
# Use to validate performance of generated components

# 1. Generate with optimizations
backbone schema generate --target all --release <module>

# 2. Build release version
cargo build --release

# 3. Run benchmarks
cargo bench --package backbone-<module>

# 4. Profile if needed
cargo run --release --bin backbone --profile <module>
```