# Database Seed Data Patterns

## Seed Data Structure

### Environment-Specific Seeds
```bash
# Standard seed data directory structure
libs/modules/{module}/seeds/
├── development/
│   ├── users.sql
│   ├── roles.sql
│   └── sample_data.sql
├── staging/
│   ├── users.sql
│   └── configuration.sql
├── production/
│   ├── initial_data.sql
│   └── system_configuration.sql
└── README.md
```

## User and Authentication Seeds

### Development User Seeds
```sql
-- File: seeds/development/users.sql
-- Description: Development users with known credentials

-- Clear existing data (for development only)
TRUNCATE TABLE user_sessions CASCADE;
TRUNCATE TABLE user_roles CASCADE;
TRUNCATE TABLE users CASCADE;

-- Insert development users with known passwords
INSERT INTO users (id, email, username, status, metadata) VALUES
('550e8400-e29b-41d4-a716-446655440001', 'root@startapp.id', 'root', 'active',
    '{"first_name": "System", "last_name": "Administrator", "roles": ["admin"], "created_by": "system"}'::jsonb),

('550e8400-e29b-41d4-a716-446655440002', 'admin@test.local', 'admin', 'active',
    '{"first_name": "Admin", "last_name": "User", "roles": ["admin"], "created_by": "system"}'::jsonb),

('550e8400-e29b-41d4-a716-446655440003', 'user@test.local', 'testuser', 'active',
    '{"first_name": "Test", "last_name": "User", "roles": ["user"], "created_by": "system"}'::jsonb),

('550e8400-e29b-41d4-a716-446655440004', 'premium@test.local', 'premium', 'active',
    '{"first_name": "Premium", "last_name": "User", "roles": ["user", "premium"], "created_by": "system"}'::jsonb);

-- Standard roles
INSERT INTO roles (id, name, description, permissions, metadata) VALUES
('550e8400-e29b-41d4-a716-446655440100', 'admin', 'System Administrator',
    ARRAY['*'], -- All permissions
    '{"level": 100, "system_role": true}'::jsonb),

('550e8400-e29b-41d4-a716-446655440101', 'premium', 'Premium User',
    ARRAY['read:own', 'write:own', 'access:premium_features'],
    '{"level": 50, "paid_tier": true}'::jsonb),

('550e8400-e29b-41d4-a716-446655440102', 'user', 'Standard User',
    ARRAY['read:own', 'write:own'],
    '{"level": 10, "default_role": true}'::jsonb);

-- User role assignments
INSERT INTO user_roles (user_id, role_id, assigned_at) VALUES
('550e8400-e29b-41d4-a716-446655440001', '550e8400-e29b-41d4-a716-446655440100', NOW()),
('550e8400-e29b-41d4-a716-446655440002', '550e8400-e29b-41d4-a716-446655440100', NOW()),
('550e8400-e29b-41d4-a716-446655440003', '550e8400-e29b-41d4-a716-446655440102', NOW()),
('550e8400-e29b-41d4-a716-446655440004', '550e8400-e29b-41d4-a716-446655440102', NOW()),
('550e8400-e29b-41d4-a716-446655440004', '550e8400-e29b-41d4-a716-446655440101', NOW());

-- User profiles with preferences
INSERT INTO user_profiles (user_id, first_name, last_name, preferences, metadata) VALUES
('550e8400-e29b-41d4-a716-446655440001', 'System', 'Administrator',
    '{"theme": "dark", "notifications": true, "language": "en"}'::jsonb,
    '{"is_system": true}'::jsonb),

('550e8400-e29b-41d4-a716-446655440002', 'Admin', 'User',
    '{"theme": "auto", "notifications": true, "language": "en"}'::jsonb,
    '{"is_admin": true}'::jsonb),

('550e8400-e29b-41d4-a716-446655440003', 'Test', 'User',
    '{"theme": "light", "notifications": false, "language": "en"}'::jsonb,
    '{"test_user": true}'::jsonb),

('550e8400-e29b-41d4-a716-446655440004', 'Premium', 'User',
    '{"theme": "dark", "notifications": true, "language": "en", "beta_features": true}'::jsonb,
    '{"premium_user": true}'::jsonb);
```

### Production User Seeds
```sql
-- File: seeds/production/users.sql
-- Description: Production-ready initial user data

-- Insert only essential system accounts
INSERT INTO users (id, email, username, status, metadata) VALUES
('550e8400-e29b-41d4-a716-446655440001', 'root@startapp.id', 'root', 'active',
    '{"first_name": "System", "last_name": "Administrator", "roles": ["admin"], "created_by": "system", "initial_setup": true}'::jsonb);

-- Essential roles
INSERT INTO roles (id, name, description, permissions, metadata) VALUES
('550e8400-e29b-41d4-a716-446655440100', 'admin', 'System Administrator',
    ARRAY['*'], -- All permissions
    '{"level": 100, "system_role": true}'::jsonb),

('550e8400-e29b-41d4-a716-446655440102', 'user', 'Standard User',
    ARRAY['read:own', 'write:own'],
    '{"level": 10, "default_role": true}'::jsonb);

-- System administrator role assignment
INSERT INTO user_roles (user_id, role_id, assigned_at) VALUES
('550e8400-e29b-41d4-a716-446655440001', '550e8400-e29b-41d4-a716-446655440100', NOW());
```

## Configuration Seeds

### System Configuration Seeds
```sql
-- File: seeds/production/system_configuration.sql
-- Description: System configuration settings

INSERT INTO system_settings (key, value, description, category, metadata) VALUES
-- Application settings
('app.name', 'Backbone Framework', 'Application name', 'application',
    '{"read_only": true}'::jsonb),

('app.version', '1.0.0', 'Application version', 'application',
    '{"read_only": true}'::jsonb),

('app.environment', 'production', 'Application environment', 'application',
    '{"read_only": true}'::jsonb),

-- Security settings
('security.session_timeout', '3600', 'Session timeout in seconds', 'security',
    '{"type": "integer", "min": 300}'::jsonb),

('security.max_login_attempts', '5', 'Maximum login attempts before lockout', 'security',
    '{"type": "integer", "min": 1, "max": 10}'::jsonb),

('security.password_min_length', '8', 'Minimum password length', 'security',
    '{"type": "integer", "min": 6, "max": 128}'::jsonb),

-- Database settings
('database.max_connections', '100', 'Maximum database connections', 'database',
    '{"type": "integer", "min": 10, "max": 1000}'::jsonb),

('database.connection_timeout', '30', 'Connection timeout in seconds', 'database',
    '{"type": "integer", "min": 5, "max": 300}'::jsonb),

-- Email settings
('email.from_address', 'noreply@startapp.id', 'Default from email address', 'email',
    '{"type": "email", "required": true}'::jsonb),

('email.from_name', 'Backbone Framework', 'Default from name', 'email',
    '{"type": "string", "required": true}'::jsonb);

-- Feature flags
INSERT INTO feature_flags (key, enabled, description, metadata) VALUES
('beta_features', 'false', 'Enable beta features for users',
    '{"rollout_percentage": 0, "user_roles": ["admin"]}'::jsonb),

('maintenance_mode', 'false', 'Enable maintenance mode',
    '{"message": "System under maintenance", "allow_admins": true}'::jsonb),

('new_ui_theme', 'false', 'Enable new UI theme',
    '{"rollout_percentage": 100, "user_tiers": ["premium"]}'::jsonb);
```

### Development Configuration Seeds
```sql
-- File: seeds/development/configuration.sql
-- Description: Development-specific configuration

-- Development feature flags
INSERT INTO feature_flags (key, enabled, description, metadata) VALUES
('beta_features', 'true', 'Enable beta features for all users',
    '{"rollout_percentage": 100, "all_users": true}'::jsonb),

('debug_mode', 'true', 'Enable debug mode',
    '{"show_sql_queries": true, "log_level": "debug"}'::jsonb),

('mock_external_services', 'true', 'Mock external service calls',
    '{"services": ["email", "payment", "shipping"]}'::jsonb),

('test_data_generator', 'true', 'Enable test data generation',
    '{"auto_generate": true, "sample_size": 100}'::jsonb);
```

## Business Data Seeds

### Product Catalog Seeds
```sql
-- File: seeds/development/products.sql
-- Description: Sample product data for development

-- Categories
INSERT INTO product_categories (id, name, description, parent_id, metadata) VALUES
('cat-electronics', 'Electronics', 'Electronic devices and accessories', NULL,
    '{"icon": "electronics", "sort_order": 1}'::jsonb),

('cat-computers', 'Computers', 'Laptops, desktops, and computer accessories', 'cat-electronics',
    '{"icon": "computer", "sort_order": 1}'::jsonb),

('cat-phones', 'Phones', 'Smartphones and phone accessories', 'cat-electronics',
    '{"icon": "phone", "sort_order": 2}'::jsonb),

('cat-books', 'Books', 'Physical and digital books', NULL,
    '{"icon": "book", "sort_order": 2}'::jsonb);

-- Products
INSERT INTO products (id, name, description, category_id, price, currency, metadata) VALUES
('prod-laptop-001', 'Pro Laptop 15"', 'High-performance laptop for professionals', 'cat-computers',
    1299.99, 'USD',
    '{"brand": "TechCorp", "model": "Pro-15", "specs": {"cpu": "Intel i7", "ram": "16GB", "storage": "512GB SSD"}}'::jsonb),

('prod-phone-001', 'SmartPhone X', 'Latest smartphone with advanced features', 'cat-phones',
    799.99, 'USD',
    '{"brand": "PhoneMaker", "model": "X", "specs": {"screen": "6.5\"", "camera": "48MP", "battery": "4000mAh"}}'::jsonb),

('prod-book-001', 'Rust Programming Guide', 'Comprehensive guide to Rust programming', 'cat-books',
    49.99, 'USD',
    '{"author": "Rust Expert", "pages": 450, "format": "hardcover", "isbn": "978-0123456789"}'::jsonb);

-- Inventory
INSERT INTO inventory (product_id, available_quantity, reserved_quantity, reorder_point, metadata) VALUES
('prod-laptop-001', 50, 5, 20,
    '{"warehouse_location": "A1-234", "supplier": "TechCorp", "last_restocked": "2024-01-15"}'::jsonb),

('prod-phone-001', 100, 15, 30,
    '{"warehouse_location": "B2-567", "supplier": "PhoneMaker", "last_restocked": "2024-01-10"}'::jsonb),

('prod-book-001', 500, 0, 100,
    '{"warehouse_location": "C3-890", "supplier": "BookPublisher", "last_restocked": "2024-01-20"}'::jsonb);
```

### Order Seeds
```sql
-- File: seeds/development/orders.sql
-- Description: Sample order data for testing

-- Sample orders
INSERT INTO orders (id, order_number, customer_id, status, total_amount, currency, metadata) VALUES
('order-001', 'ORD-2024-0001', '550e8400-e29b-41d4-a716-446655440002', 'delivered', 1349.98, 'USD',
    '{"payment_method": "credit_card", "shipping_address": {"street": "123 Main St", "city": "Test City"}, "order_source": "web"}'::jsonb),

('order-002', 'ORD-2024-0002', '550e8400-e29b-41d4-a716-446655440003', 'shipped', 799.99, 'USD',
    '{"payment_method": "paypal", "shipping_address": {"street": "456 Oak Ave", "city": "Test Town"}, "order_source": "mobile"}'::jsonb),

('order-003', 'ORD-2024-0003', '550e8400-e29b-41d4-a716-446655440004', 'processing', 1599.97, 'USD',
    '{"payment_method": "credit_card", "shipping_address": {"street": "789 Pine Rd", "city": "Test Village"}, "order_source": "web", "priority": true}'::jsonb);

-- Order items
INSERT INTO order_items (id, order_id, product_id, product_name, quantity, unit_price, total_price) VALUES
('item-001-1', 'order-001', 'prod-laptop-001', 'Pro Laptop 15"', 1, 1299.99, 1299.99),
('item-001-2', 'order-001', 'prod-book-001', 'Rust Programming Guide', 1, 49.99, 49.99),

('item-002-1', 'order-002', 'prod-phone-001', 'SmartPhone X', 1, 799.99, 799.99),

('item-003-1', 'order-003', 'prod-laptop-001', 'Pro Laptop 15"', 1, 1299.99, 1299.99),
('item-003-2', 'order-003', 'prod-phone-001', 'SmartPhone X', 1, 799.99, 799.99);
```

## Seed Data Management Scripts

### Seed Execution Script
```bash
#!/bin/bash
# File: scripts/run_seeds.sh
# Description: Execute seed data for specific environment

set -e

ENVIRONMENT=${1:-development}
MODULE=${2:-all}

echo "Running seed data for environment: $ENVIRONMENT"
echo "Module: $MODULE"

# Base seed directory
SEED_DIR="libs/modules"

# Function to run seeds for a module
run_module_seeds() {
    local module=$1
    local env=$2
    local seed_dir="$SEED_DIR/$module/seeds/$env"

    if [ -d "$seed_dir" ]; then
        echo "Running seeds for module: $module"

        for seed_file in "$seed_dir"/*.sql; do
            if [ -f "$seed_file" ]; then
                echo "Executing: $seed_file"
                DATABASE_URL="postgresql://root:password@localhost:5432/${module}_${env}_db" \
                psql -f "$seed_file"
            fi
        done
    else
        echo "No seed directory found for module: $module"
    fi
}

# Run seeds for specific module or all modules
if [ "$MODULE" = "all" ]; then
    for module_dir in "$SEED_DIR"/*; do
        if [ -d "$module_dir" ]; then
            module=$(basename "$module_dir")
            run_module_seeds "$module" "$ENVIRONMENT"
        fi
    done
else
    run_module_seeds "$MODULE" "$ENVIRONMENT"
fi

echo "Seed data execution completed!"
```

### Seed Validation Script
```sql
-- File: scripts/validate_seeds.sql
-- Description: Validate seed data integrity

-- Check user seed data
SELECT
    (SELECT COUNT(*) FROM users WHERE email LIKE '%@test.local') as test_users,
    (SELECT COUNT(*) FROM users WHERE email = 'root@startapp.id') as root_user,
    (SELECT COUNT(*) FROM roles) as total_roles,
    (SELECT COUNT(*) FROM user_roles) as user_role_assignments;

-- Check product seed data
SELECT
    (SELECT COUNT(*) FROM product_categories) as categories,
    (SELECT COUNT(*) FROM products) as products,
    (SELECT COUNT(*) FROM inventory) as inventory_records;

-- Check configuration
SELECT
    (SELECT COUNT(*) FROM system_settings) as settings,
    (SELECT COUNT(*) FROM feature_flags) as feature_flags;

-- Sample data verification
SELECT
    u.email,
    u.username,
    array_agg(r.name) as roles
FROM users u
JOIN user_roles ur ON u.id = ur.user_id
JOIN roles r ON ur.role_id = r.id
GROUP BY u.id, u.email, u.username
ORDER BY u.created_at
LIMIT 10;
```

## Environment-Specific Considerations

### Development Seeds
- Full sample data for comprehensive testing
- Known credentials for easy login
- Diverse scenarios for edge case testing
- Mock external service configurations

### Staging Seeds
- Realistic data volume for performance testing
- Production-like configuration
- Limited user accounts with varied permissions
- Integration test scenarios

### Production Seeds
- Minimal essential data only
- System administrator account
- Default configuration settings
- No test or sample data
- Security-focused settings only