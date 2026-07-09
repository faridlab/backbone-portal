# Reference Documentation for Database Migration Specialist

## Project Documentation References

### Database and Migration Documentation
- **[SCHEMA/MIGRATION_GUIDE.md](../../docs/schema/MIGRATION_GUIDE.md)** - Schema migration guide and best practices
- **[SCHEMA/WORKFLOW.md](../../docs/schema/WORKFLOW.md)** - Schema workflow including migrations
- **[SCHEMA/GENERATION.md](../../docs/schema/GENERATION.md)** - Code generation for migrations
- **[FRAMEWORK.md](../../docs/technical/FRAMEWORK.md)** - Framework database patterns

### Implementation Guides
- **[LOCAL_DEVELOPMENT.md](../../docs/LOCAL_DEVELOPMENT.md)** - Local development database setup
- **[DEPLOYMENT_STRATEGY.md](../../docs/technical/DEPLOYMENT_STRATEGY.md)** - Database deployment strategies
- **[PRODUCTION_READINESS.md](../../docs/PRODUCTION_READINESS.md)** - Production database considerations

### Module-Specific References
- **[SAPIENS_FIXING_ERRORS.md](../../docs/plans/SAPIENS_FIXING_ERRORS.md)** - Sapiens database migration issues
- **[SAPIENS_IMPLEMENTATION_PLAN.md](../../docs/plans/SAPIENS_IMPLEMENTATION_PLAN.md)** - Sapiens module database design

## Database Configuration

### PostgreSQL as Primary Database
```yaml
# Framework standard: PostgreSQL is primary database
database_priorities:
  primary: PostgreSQL
  reasons:
    - ACID compliance
    - Powerful query capabilities
    - JSON/JSONB support
    - Better tooling ecosystem
    - Industry standard

  legacy: MongoDB
  usage:
    - Existing modules (Sapiens, Postman)
    - Gradual migration to PostgreSQL
    - Specific document store use cases
```

### Database Connection Patterns
```yaml
# Standard database configuration
database:
  url: "postgresql://root:password@localhost:5432/{database}"
  max_connections: 20
  min_connections: 5
  connection_timeout: 30s
  idle_timeout: 10m

# Environment-specific configurations
environments:
  development:
    database: "{module}_dev"
    ssl_mode: disable
    log_queries: true

  staging:
    database: "{module}_staging"
    ssl_mode: require
    log_queries: false

  production:
    database: "{module}_prod"
    ssl_mode: require
    log_queries: false
    read_replica: true
```

## Migration Standards

### Backbone Migration Rules
```yaml
# STRICT ENFORCEMENT: Only PostgreSQL migrations from Backbone schema
migration_rules:
  only_use:
    - "backbone schema migration <module>"
    - PostgreSQL-specific patterns
    - Generated migration files

  never_use:
    - Manual SQL for core schema changes
    - Backbone-generated migration file modifications
    - Direct database modifications
    - Non-PostgreSQL patterns

  always_do:
    - Validate before applying
    - Test on development first
    - Use transaction blocks
    - Include rollback procedures
```

### Migration File Structure
```sql
-- Migration naming convention: {timestamp}_{description}.sql
-- Example: 20241212_143022_create_user_table.sql

-- Standard migration template
BEGIN;

-- Create table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(50) UNIQUE NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE NULL
);

-- Create indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_created_at ON users(created_at);

-- Create trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert initial data if needed
INSERT INTO users (email, username) VALUES
    ('admin@example.com', 'admin');

COMMIT;

-- Rollback section (commented, used for rollback)
-- DROP TRIGGER IF EXISTS update_users_updated_at ON users;
-- DROP FUNCTION IF EXISTS update_updated_at_column();
-- DROP TABLE IF EXISTS users;
```

### Metadata Storage Pattern
```sql
-- Standard metadata columns for all entities
CREATE TABLE example_entity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Business fields here

    -- Standard metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by UUID NULL,
    updated_by UUID NULL,
    version INTEGER DEFAULT 1,
    deleted_at TIMESTAMP WITH TIME ZONE NULL,

    -- JSON metadata for flexibility
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Metadata indexes
CREATE INDEX idx_example_entity_created_at ON example_entity(created_at);
CREATE INDEX idx_example_entity_updated_at ON example_entity(updated_at);
CREATE INDEX idx_example_entity_deleted_at ON example_entity(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_example_entity_metadata ON example_entity USING GIN(metadata);
```

## Schema to Database Mapping

### Entity to Table Mapping
```yaml
# Backbone schema field types to PostgreSQL types
field_mapping:
  uuid: UUID
  string: VARCHAR(255)
  integer: INTEGER
  float: DECIMAL(10,2)
  boolean: BOOLEAN
  timestamp: TIMESTAMP WITH TIME ZONE
  email: VARCHAR(255)
  password: VARCHAR(255)
  json: JSONB
  enum: VARCHAR(50) or CHECK constraint
  object: JSONB
  array: ARRAY or JSONB

# Example schema to table conversion
schema:
  User:
    id: { type: uuid, primary_key: true }
    email: { type: email, unique: true, indexed: true }
    status: { type: enum, values: [active, inactive, pending] }

sql_output:
  CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('active', 'inactive', 'pending')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
  );

  CREATE INDEX idx_users_email ON users(email);
```

### Relationship Mapping
```yaml
# One-to-Many relationship
schema:
  User:
    posts: { type: array, relation: many, target: Post }
  Post:
    user_id: { type: uuid, relation: belongs_to, target: User }

sql_output:
  CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- other user fields
  );

  CREATE TABLE posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- other post fields
  );

  CREATE INDEX idx_posts_user_id ON posts(user_id);

# Many-to-Many relationship
schema:
  User:
    roles: { type: array, relation: many_many, target: Role, through: UserRole }

sql_output:
  CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- user fields
  );

  CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- role fields
  );

  CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
  );
```

## Migration Workflows

### Development Migration Workflow
```bash
# 1. Make schema changes
# Edit libs/modules/{module}/schema/models/{entity}.model.yaml

# 2. Generate migration
backbone schema migration <module>

# 3. Review generated migration
cat libs/modules/<module>/migrations/*_create_{table}.sql

# 4. Test migration on development
DATABASE_URL="postgresql://root:password@localhost:5432/{module}_dev" \
backbone schema migrate up <module>

# 5. Verify migration
psql -h localhost -U root -d {module}_dev -c "\dt"
psql -h localhost -U root -d {module}_dev -c "\d {table}"

# 6. Test rollback (if needed)
backbone schema migrate down <module>
```

### Production Migration Workflow
```bash
# 1. Test on staging environment
DATABASE_URL="postgresql://root:password@staging-db:5432/{module}_staging" \
backbone schema migrate up <module>

# 2. Verify data integrity
SELECT COUNT(*) FROM {table};
SELECT * FROM {table} LIMIT 5;

# 3. Backup production database
pg_dump -h production-db -U root -d {module}_prod > backup_$(date +%Y%m%d_%H%M%S).sql

# 4. Run production migration with monitoring
DATABASE_URL="postgresql://root:password@production-db:5432/{module}_prod" \
backbone schema migrate up <module>

# 5. Verify migration success
backbone schema migration status <module>
```

### Migration Testing Workflow
```bash
# 1. Create test database
createdb {module}_test

# 2. Apply migrations to test database
DATABASE_URL="postgresql://root:password@localhost:5432/{module}_test" \
backbone schema migrate up <module>

# 3. Run test suite
DATABASE_URL="postgresql://root:password@localhost:5432/{module}_test" \
cargo test --package backbone-{module}

# 4. Test rollback scenarios
DATABASE_URL="postgresql://root:password@localhost:5432/{module}_test" \
backbone schema migrate down <module>

# 5. Cleanup test database
dropdb {module}_test
```

## Migration Types

### DDL Migrations
```sql
-- Table creation
CREATE TABLE new_table (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Add column
ALTER TABLE existing_table
ADD COLUMN new_column VARCHAR(100);

-- Modify column
ALTER TABLE existing_table
ALTER COLUMN existing_column TYPE VARCHAR(200);

-- Add index
CREATE INDEX idx_table_column ON existing_table(column);

-- Add constraint
ALTER TABLE existing_table
ADD CONSTRAINT chk_column_positive CHECK (column >= 0);
```

### DML Migrations
```sql
-- Data migration
UPDATE users
SET status = 'active'
WHERE status = 'pending' AND created_at < NOW() - INTERVAL '1 day';

-- Data transformation
INSERT INTO user_profiles (user_id, preferences)
SELECT id, '{"theme": "light"}'::jsonb
FROM users
WHERE id NOT IN (SELECT user_id FROM user_profiles);

-- Bulk data insertion
INSERT INTO audit_logs (entity_id, entity_type, action, created_at)
SELECT id, 'user', 'created', created_at
FROM users
WHERE created_at > NOW() - INTERVAL '1 day';
```

### JSONB Data Migrations
```sql
-- Add new field to JSONB metadata
UPDATE users
SET metadata = jsonb_set(
    metadata,
    '{preferences}',
    '{"theme": "auto", "notifications": true}'::jsonb
)
WHERE metadata->>'preferences' IS NULL;

-- Migrate from old JSON structure to new
UPDATE orders
SET metadata = jsonb_build_object(
    'old_data', metadata,
    'new_field', 'default_value',
    'migrated_at', NOW()
)
WHERE metadata ? 'old_structure';
```

## Error Handling and Recovery

### Common Migration Errors
```yaml
# Syntax errors
syntax_error:
  detection: "PostgreSQL syntax error"
  solution: "Check SQL syntax, verify keywords, test on development"

# Constraint violations
constraint_violation:
  detection: "Constraint violation error"
  solution: "Check data integrity, clean data before migration"

# Lock timeouts
lock_timeout:
  detection: "Lock timeout exceeded"
  solution: "Run during maintenance window, use shorter transactions"

# Memory issues
memory_issue:
  detection: "Out of memory error"
  solution: "Process data in batches, increase work_mem"
```

### Rollback Procedures
```sql
-- Rollback migration template
BEGIN;

-- Reverse the changes
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
DROP FUNCTION IF EXISTS update_updated_at_column();
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_status;
DROP TABLE IF EXISTS users;

COMMIT;

-- Verify rollback
SELECT table_name FROM information_schema.tables
WHERE table_schema = 'public' AND table_name = 'users';
```

### Migration Rollback Workflow
```bash
# 1. Identify last successful migration
backbone schema migration status <module>

# 2. Rollback to previous version
backbone schema migrate down <module>

# 3. Verify rollback
psql -h localhost -U root -d {module}_dev -c "\dt"

# 4. Fix migration issue
# Edit migration file or schema

# 5. Regenerate migration if needed
backbone schema migration <module> --force

# 6. Re-apply migration
DATABASE_URL="postgresql://root:password@localhost:5432/{module}_dev" \
backbone schema migrate up <module>
```

## Performance Considerations

### Indexing Strategy
```sql
-- Primary indexes
CREATE INDEX CONCURRENTLY idx_table_column ON table(column);

-- Composite indexes for common queries
CREATE INDEX CONCURRENTLY idx_orders_status_date ON orders(status, created_at);

-- Partial indexes
CREATE INDEX CONCURRENTLY idx_active_users ON users(id) WHERE status = 'active';

-- JSONB indexes
CREATE INDEX CONCURRENTLY idx_users_metadata ON users USING GIN(metadata);
CREATE INDEX CONCURRENTLY idx_users_metadata_specific
ON users USING GIN((metadata->'preferences'));
```

### Migration Performance
```sql
-- Large table modifications (minimal locking)
ALTER TABLE large_table
ADD COLUMN new_column VARCHAR(100) DEFAULT null;

-- Update in batches
UPDATE large_table
SET new_column = calculate_value()
WHERE id % 1000 = 0;

-- Use transactions for consistency
BEGIN;
ALTER TABLE table_name ADD COLUMN column_name VARCHAR(100);
UPDATE table_name SET column_name = 'default_value' WHERE column_name IS NULL;
ALTER TABLE table_name ALTER COLUMN column_name SET NOT NULL;
COMMIT;
```

## Monitoring and Validation

### Migration Validation Scripts
```sql
-- Validate table structure
SELECT column_name, data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_name = 'users'
ORDER BY ordinal_position;

-- Validate constraints
SELECT constraint_name, constraint_type
FROM information_schema.table_constraints
WHERE table_name = 'users';

-- Validate indexes
SELECT indexname, indexdef
FROM pg_indexes
WHERE tablename = 'users';

-- Validate row counts
SELECT
    schemaname,
    tablename,
    n_tup_ins as inserts,
    n_tup_upd as updates,
    n_tup_del as deletes,
    n_live_tup as live_rows,
    n_dead_tup as dead_rows
FROM pg_stat_user_tables
WHERE tablename = 'users';
```

### Migration Monitoring
```bash
# Monitor migration progress
watch -n 5 "psql -h localhost -U root -d {database} -c 'SELECT COUNT(*) FROM {table}'"

# Check for long-running queries
SELECT
    pid,
    now() - pg_stat_activity.query_start AS duration,
    query,
    state
FROM pg_stat_activity
WHERE (now() - pg_stat_activity.query_start) > interval '5 minutes'
ORDER BY duration DESC;

# Check table sizes after migration
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables
WHERE schemaname NOT IN ('information_schema', 'pg_catalog')
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```