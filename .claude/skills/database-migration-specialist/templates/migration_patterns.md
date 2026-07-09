# Database Migration Patterns

## Migration File Templates

### Basic Table Creation
```sql
-- Migration: {timestamp}_create_{entity}_table.sql
-- Description: Create {Entity} table with standard Backbone patterns

BEGIN;

-- Create table with Backbone standard fields
CREATE TABLE {table_name} (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Business fields (customize per entity)
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'active',

    -- Standard Backbone metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by UUID NULL,
    updated_by UUID NULL,
    version INTEGER DEFAULT 1,
    deleted_at TIMESTAMP WITH TIME ZONE NULL,

    -- Flexible metadata storage
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Standard indexes
CREATE INDEX idx_{table}_status ON {table_name}(status);
CREATE INDEX idx_{table}_created_at ON {table_name}(created_at);
CREATE INDEX idx_{table}_updated_at ON {table_name}(updated_at);
CREATE INDEX idx_{table}_deleted_at ON {table_name}(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_{table}_metadata ON {table_name} USING GIN(metadata);

-- Updated at trigger
CREATE OR REPLACE FUNCTION update_{table}_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    NEW.version = OLD.version + 1;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_{table}_updated_at
    BEFORE UPDATE ON {table_name}
    FOR EACH ROW EXECUTE FUNCTION update_{table}_updated_at_column();

-- Insert default data if needed
INSERT INTO {table_name} (name, description, status) VALUES
    ('Default Item', 'Default description', 'active');

COMMIT;

-- Rollback section
-- DROP TRIGGER IF EXISTS update_{table}_updated_at ON {table_name};
-- DROP FUNCTION IF EXISTS update_{table}_updated_at_column();
-- DROP TABLE IF EXISTS {table_name};
```

### Entity with Relationships
```sql
-- Migration: {timestamp}_create_{entity}_with_relationships.sql

BEGIN;

-- Create main entity table
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_number VARCHAR(50) UNIQUE NOT NULL,
    customer_id UUID NOT NULL,
    status VARCHAR(20) DEFAULT 'draft',
    total_amount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    currency VARCHAR(3) DEFAULT 'USD',

    -- Standard metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by UUID NULL,
    updated_by UUID NULL,
    version INTEGER DEFAULT 1,
    deleted_at TIMESTAMP WITH TIME ZONE NULL,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Create related entity table
CREATE TABLE order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL,
    product_id UUID NOT NULL,
    product_name VARCHAR(255) NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price DECIMAL(10,2) NOT NULL CHECK (unit_price >= 0),
    total_price DECIMAL(10,2) NOT NULL GENERATED ALWAYS AS (quantity * unit_price) STORED,

    -- Standard metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Foreign key constraints
ALTER TABLE orders
ADD CONSTRAINT fk_orders_customer_id
FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE;

ALTER TABLE order_items
ADD CONSTRAINT fk_order_items_order_id
FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE;

ALTER TABLE order_items
ADD CONSTRAINT fk_order_items_product_id
FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE RESTRICT;

-- Indexes for performance
CREATE INDEX idx_orders_customer_id ON orders(customer_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_at ON orders(created_at);
CREATE INDEX idx_orders_order_number ON orders(order_number);

CREATE INDEX idx_order_items_order_id ON order_items(order_id);
CREATE INDEX idx_order_items_product_id ON order_items(product_id);

-- Composite index for order status and date
CREATE INDEX idx_orders_status_created_at ON orders(status, created_at);

-- Updated at triggers
CREATE OR REPLACE FUNCTION update_orders_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    NEW.version = OLD.version + 1;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_orders_updated_at
    BEFORE UPDATE ON orders
    FOR EACH ROW EXECUTE FUNCTION update_orders_updated_at_column();

CREATE OR REPLACE FUNCTION update_order_items_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_order_items_updated_at
    BEFORE UPDATE ON order_items
    FOR EACH ROW EXECUTE FUNCTION update_order_items_updated_at_column();

COMMIT;
```

### Enum Type Creation
```sql
-- Migration: {timestamp}_create_{entity}_status_enum.sql

BEGIN;

-- Create enum type
CREATE TYPE order_status AS ENUM (
    'draft',
    'pending_payment',
    'confirmed',
    'processing',
    'shipped',
    'delivered',
    'cancelled',
    'refunded'
);

-- Add enum column to existing table
ALTER TABLE orders
ADD COLUMN status order_status DEFAULT 'draft';

-- Add constraint for data integrity
ALTER TABLE orders
ADD CONSTRAINT chk_orders_status_valid
CHECK (status IN ('draft', 'pending_payment', 'confirmed', 'processing', 'shipped', 'delivered', 'cancelled', 'refunded'));

-- Create index for filtering
CREATE INDEX idx_orders_status ON orders(status);

-- Update existing data if table has data
UPDATE orders SET status = 'draft' WHERE status IS NULL;

-- Make column NOT NULL after updating existing data
ALTER TABLE orders ALTER COLUMN status SET NOT NULL;

COMMIT;

-- Rollback section
-- ALTER TABLE orders DROP COLUMN IF EXISTS status;
-- DROP TYPE IF EXISTS order_status;
```

### Many-to-Many Relationship
```sql
-- Migration: {timestamp}_create_{junction}_table.sql

BEGIN;

-- Create junction table for many-to-many relationship
CREATE TABLE user_roles (
    user_id UUID NOT NULL,
    role_id UUID NOT NULL,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    assigned_by UUID NULL,
    expires_at TIMESTAMP WITH TIME ZONE NULL,

    PRIMARY KEY (user_id, role_id)
);

-- Foreign key constraints
ALTER TABLE user_roles
ADD CONSTRAINT fk_user_roles_user_id
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE user_roles
ADD CONSTRAINT fk_user_roles_role_id
FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE;

-- Additional constraints
ALTER TABLE user_roles
ADD CONSTRAINT chk_user_roles_expires_future
CHECK (expires_at IS NULL OR expires_at > assigned_at);

-- Indexes
CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX idx_user_roles_expires_at ON user_roles(expires_at) WHERE expires_at IS NOT NULL;

-- Unique constraint to prevent duplicate assignments
CREATE UNIQUE INDEX idx_user_roles_unique_active
ON user_roles(user_id, role_id)
WHERE expires_at IS NULL OR expires_at > NOW();

COMMIT;
```

## Migration Modification Patterns

### Add Column Migration
```sql
-- Migration: {timestamp}_add_{column}_to_{table}.sql

BEGIN;

-- Add new column (nullable for safety)
ALTER TABLE {table_name}
ADD COLUMN {column_name} {column_type} NULL;

-- Add index if column will be queried frequently
CREATE INDEX CONCURRENTLY idx_{table}_{column} ON {table_name}({column_name});

-- Update existing data if needed
UPDATE {table_name}
SET {column_name} {default_value}
WHERE {column_name} IS NULL;

-- Add constraint if column should be NOT NULL
ALTER TABLE {table_name}
ALTER COLUMN {column_name} SET NOT NULL;

-- Add default value for new rows
ALTER TABLE {table_name}
ALTER COLUMN {column_name} SET DEFAULT {default_value};

COMMIT;

-- Rollback section
-- ALTER TABLE {table_name} DROP COLUMN IF EXISTS {column_name};
```

### Modify Column Migration
```sql
-- Migration: {timestamp}_modify_{column}_in_{table}.sql

BEGIN;

-- Option 1: Simple type change (compatible)
ALTER TABLE {table_name}
ALTER COLUMN {column_name} TYPE {new_type} USING {conversion_expression};

-- Option 2: Complex type change (requires temporary column)
-- Add new column
ALTER TABLE {table_name}
ADD COLUMN {column_name}_new {new_type};

-- Migrate data
UPDATE {table_name}
SET {column_name}_new = {migration_expression};

-- Add constraints to new column
ALTER TABLE {table_name}
ADD CONSTRAINT chk_{table}_{column}_new
CHECK ({constraint_expression});

-- Drop old column and rename new one
ALTER TABLE {table_name}
DROP COLUMN {column_name};

ALTER TABLE {table_name}
RENAME COLUMN {column_name}_new TO {column_name};

-- Rebuild indexes if needed
DROP INDEX IF EXISTS idx_{table}_{column};
CREATE INDEX CONCURRENTLY idx_{table}_{column} ON {table_name}({column_name});

COMMIT;
```

### Add Constraint Migration
```sql
-- Migration: {timestamp}_add_{constraint}_to_{table}.sql

BEGIN;

-- Add check constraint
ALTER TABLE {table_name}
ADD CONSTRAINT chk_{constraint_name}
CHECK ({constraint_expression});

-- Add unique constraint
ALTER TABLE {table_name}
ADD CONSTRAINT uniq_{constraint_name}
UNIQUE ({column_list});

-- Add foreign key constraint
ALTER TABLE {table_name}
ADD CONSTRAINT fk_{constraint_name}
FOREIGN KEY ({column_name})
REFERENCES {referenced_table}({referenced_column})
ON DELETE {action};

-- Create index for foreign key performance
CREATE INDEX CONCURRENTLY idx_{table}_{column}_fk
ON {table_name}({column_name});

COMMIT;

-- Rollback section
-- ALTER TABLE {table_name} DROP CONSTRAINT IF EXISTS {constraint_name};
```

## Data Migration Patterns

### Data Transformation Migration
```sql
-- Migration: {timestamp}_transform_{data}_in_{table}.sql

BEGIN;

-- Create backup table
CREATE TABLE {table_name}_backup AS
SELECT * FROM {table_name};

-- Transform data in batches for large tables
DO $$
DECLARE
    batch_size INTEGER := 1000;
    offset_val INTEGER := 0;
    affected_rows INTEGER;
BEGIN
    LOOP
        UPDATE {table_name}
        SET {target_column} = {transformation_expression}
        WHERE {condition}
        LIMIT batch_size;

        GET DIAGNOSTICS affected_rows = ROW_COUNT;

        EXIT WHEN affected_rows = 0;

        RAISE NOTICE 'Updated % rows, offset %', affected_rows, offset_val;
        offset_val := offset_val + batch_size;

        -- Commit each batch to avoid long transactions
        COMMIT;
    END LOOP;
END $$;

-- Verify transformation results
SELECT
    COUNT(*) as total_rows,
    COUNT(CASE WHEN {verification_condition} THEN 1 END) as transformed_rows
FROM {table_name}
WHERE {data_filter};

COMMIT;

-- Rollback: Restore from backup if needed
-- TRUNCATE TABLE {table_name};
-- INSERT INTO {table_name} SELECT * FROM {table_name}_backup;
-- DROP TABLE {table_name}_backup;
```

### JSONB Data Migration
```sql
-- Migration: {timestamp}_migrate_{entity}_metadata.sql

BEGIN;

-- Add new fields to JSONB metadata for existing records
UPDATE {table_name}
SET metadata = jsonb_set(
    metadata,
    '{new_field}',
    to_jsonb({default_value})
)
WHERE NOT metadata ? 'new_field';

-- Restructure existing JSONB data
UPDATE {table_name}
SET metadata = jsonb_build_object(
    'version', COALESCE((metadata->>'version')::integer, 1),
    'settings', metadata->'settings',
    'preferences', COALESCE(metadata->'preferences', '{}'::jsonb),
    'migrated_at', NOW()
)
WHERE metadata IS NOT NULL;

-- Create partial index for specific JSONB queries
CREATE INDEX CONCURRENTLY idx_{table}_metadata_field
ON {table_name} USING GIN((metadata->'specific_field'));

-- Validate JSONB structure
SELECT
    COUNT(*) as total_records,
    COUNT(CASE WHEN metadata ? 'required_field' THEN 1 END) as has_required_field
FROM {table_name};

COMMIT;
```

### Bulk Data Insertion
```sql
-- Migration: {timestamp}_populate_{table}_data.sql

BEGIN;

-- Disable triggers for performance (remember to re-enable)
ALTER TABLE {table_name} DISABLE TRIGGER ALL;

-- Insert data in batches for large datasets
INSERT INTO {table_name} ({columns})
SELECT {source_columns}
FROM {source_table}
WHERE {filter_condition}
LIMIT 10000;

-- Re-enable triggers
ALTER TABLE {table_name} ENABLE TRIGGER ALL;

-- Update sequence values if using serial columns
SELECT setval('{table_name}_id_seq', (SELECT MAX(id) FROM {table_name}));

-- Analyze table for query planner
ANALYZE {table_name};

-- Verify data insertion
SELECT COUNT(*) as inserted_rows FROM {table_name};

COMMIT;
```

## Advanced Migration Patterns

### Zero-Downtime Migration
```sql
-- Step 1: Add new column (nullable)
ALTER TABLE {table_name} ADD COLUMN {new_column} {data_type};

-- Step 2: Deploy application code that reads from old column, writes to both
-- (Deploy this change)

-- Step 3: Backfill data for existing records
UPDATE {table_name}
SET {new_column} = {backfill_expression}
WHERE {new_column} IS NULL;

-- Step 4: Verify data consistency
SELECT
    COUNT(*) as total,
    COUNT(CASE WHEN {new_column} IS NOT NULL THEN 1 END) as has_new_column
FROM {table_name};

-- Step 5: Deploy application code that reads from new column
-- (Deploy this change)

-- Step 6: Make new column NOT NULL
ALTER TABLE {table_name} ALTER COLUMN {new_column} SET NOT NULL;

-- Step 7: Drop old column (after verification)
ALTER TABLE {table_name} DROP COLUMN {old_column};
```

### Partitioned Table Migration
```sql
-- Migration: {timestamp}_create_partitioned_{table}.sql

BEGIN;

-- Create partitioned table
CREATE TABLE {table_name} (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    -- Other columns...

    -- Partition key
    PARTITION BY RANGE (created_at)
);

-- Create partitions
CREATE TABLE {table_name}_{year}_{month} PARTITION OF {table_name}
FOR VALUES FROM ('{start_date}') TO ('{end_date}');

-- Create indexes on partitioned table
CREATE INDEX idx_{table}_created_at ON {table_name} (created_at);
CREATE INDEX idx_{table}_metadata ON {table_name} USING GIN(metadata);

-- Migrate data from existing table
INSERT INTO {table_name}
SELECT * FROM {old_table_name}
ORDER BY created_at;

-- Rename tables (swap old with new)
ALTER TABLE {old_table_name} RENAME TO {old_table_name}_old;
ALTER TABLE {table_name} RENAME TO {old_table_name};

-- Verify migration
SELECT COUNT(*) as total_rows FROM {table_name};

COMMIT;
```

### Column Encryption Migration
```sql
-- Migration: {timestamp}_encrypt_{column}_in_{table}.sql

BEGIN;

-- Add encrypted column
ALTER TABLE {table_name}
ADD COLUMN {column}_encrypted BYTEA;

-- Encrypt existing data in batches
DO $$
DECLARE
    batch_id UUID;
    batch_records RECORD;
BEGIN
    FOR batch_records IN
        SELECT id, {column}
        FROM {table_name}
        WHERE {column}_encrypted IS NULL
        LIMIT 1000
    LOOP
        UPDATE {table_name}
        SET {column}_encrypted = pgp_sym_encrypt({column}, {encryption_key})
        WHERE id = batch_records.id;

        RAISE NOTICE 'Encrypted record %', batch_records.id;
    END LOOP;
END $$;

-- Add constraint to ensure encryption
ALTER TABLE {table_name}
ADD CONSTRAINT chk_{column}_encrypted_not_null
CHECK ({column}_encrypted IS NOT NULL);

-- After verification, drop unencrypted column
-- ALTER TABLE {table_name} DROP COLUMN {column};
-- ALTER TABLE {table_name} RENAME COLUMN {column}_encrypted TO {column};

COMMIT;
```