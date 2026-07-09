# Backbone Schema Validation

## Schema Structure Validation

### Required Schema Elements
```yaml
# Every model.yaml must include:
name: EntityName           # Required: PascalCase
description: "Description" # Required: Human readable
version: "1.0"            # Required: Semantic version
type: entity              # Optional: entity (default) or value_object

# Fields section (at least one field required)
fields:
  id:
    type: uuid
    primary_key: true
    # ... field definition
```

### Module Schema Configuration
```yaml
# libs/modules/{module}/schema/schema.yaml
module: {module_name}
version: "1.0"
description: "Module schema configuration"

settings:
  default_database: postgresql
  soft_delete: true
  audit_fields: true
  timestamps: true

imports:
  - module: common
    entities: [Email, Money, Address]

exports:
  - User
  - UserProfile
```

## Field Validation Rules

### Type Validation
```yaml
# Valid field types:
primitive_types:
  - string       # Text data
  - integer      # Whole numbers
  - float        # Decimal numbers
  - boolean      # True/false
  - timestamp    # Date/time
  - uuid         # Unique identifier
  - email        # Email address
  - password     # Hashed password
  - json         # JSON data
  - enum         # Fixed value set

complex_types:
  - object       # Nested structure
  - array        # Collections
  - relation     # Entity relationships

# Type-specific validations
string_field:
  type: string
  min_length: 1
  max_length: 255
  pattern: "^[a-zA-Z0-9\s]+$"
  nullable: false
  default: "default_value"

email_field:
  type: email
  required: true
  unique: true
  validate:
    - { rule: email_format }
    - { rule: mx_record }
    - { rule: disposable_check }
```

### Enum Validation
```yaml
status_field:
  type: enum
  values: [active, inactive, pending, suspended]
  default: pending
  required: true
  # All enum values must be lowercase snake_case
```

### Array Validation
```yaml
tags_field:
  type: array
  items:
    type: string
    min_length: 1
    max_length: 50
  min_items: 0
  max_items: 10
  unique_items: true
```

### Object Validation
```yaml
preferences_field:
  type: object
  nullable: true
  properties:
    theme:
      type: enum
      values: [light, dark, auto]
      default: auto
    notifications:
      type: boolean
      default: true
    language:
      type: string
      pattern: "^[a-z]{2}$"
      default: en
  required_properties: [theme]
```

## Relationship Validation

### Foreign Key Constraints
```yaml
# Valid relationship definition
user_id:
  type: uuid
  required: true
  indexed: true
  relation:
    type: belongs_to
    target: User
    foreign_key: id
    cascade: delete
```

### Polymorphic Relationships
```yaml
attachable_id:
  type: uuid
  required: true
  indexed: true

attachable_type:
  type: enum
  values: [User, Post, Comment]
  required: true
  indexed: true
  relation:
    type: polymorphic
    types: [User, Post, Comment]
```

### Many-to-Many Relationships
```yaml
# In junction entity (UserRole)
user_id:
  type: uuid
  required: true
  indexed: true
  relation:
    type: belongs_to
    target: User
    foreign_key: id

role_id:
  type: uuid
  required: true
  indexed: true
  relation:
    type: belongs_to
    target: Role
    foreign_key: id

# Unique constraint on both fields
indexes:
  - name: idx_user_role_unique
    fields: [user_id, role_id]
    unique: true
```

## Validation Rules Reference

### Built-in Validation Rules
```yaml
# String validations
email_format:     # Valid email format
url_format:       # Valid URL format
phone_format:     # Valid phone format
uuid_format:      # Valid UUID format
min_length: n     # Minimum length
max_length: n     # Maximum length
pattern: regex    # Regex pattern match

# Numeric validations
min_value: n      # Minimum value
max_value: n      # Maximum value
positive: true    # Must be positive
negative: true    # Must be negative

# Date/time validations
past_only: true   # Must be in past
future_only: true # Must be in future
business_hours: true # Must be business hours

# Array validations
min_items: n      # Minimum items
max_items: n      # Maximum items
unique_items: true # Items must be unique

# Custom validations
custom: function_name
business_rule: rule_name
```

### Custom Validation Functions
```yaml
# Schema-level custom validation
validation:
  - name: password_strength
    fields: [password]
    rule: custom
    function: validate_password_strength
    parameters:
      min_length: 8
      require_uppercase: true
      require_lowercase: true
      require_numbers: true
      require_symbols: true

  - name: business_email
    fields: [email]
    rule: custom
    function: validate_business_email
    message: "Must use business email address"
```

## Schema Validation Commands

### Validation Commands
```bash
# Validate single module schema
backbone schema validate <module>

# Validate all modules
for module in $(ls libs/modules/); do
  backbone schema validate $module
done

# Validate with verbose output
backbone schema validate <module> --verbose

# Validate with warnings
backbone schema validate <module> --warnings
```

### Validation Output
```bash
# Successful validation
✓ Schema validation passed for sapiens

# With warnings
⚠ Schema validation passed with warnings for sapiens
  - User.email: Consider adding unique index
  - UserProfile.preferences: Large object field

# With errors
✗ Schema validation failed for sapiens
  - User.username: Missing required field 'username'
  - User.status: Invalid enum value 'active_state'
  - UserProfile.user_id: Missing foreign key constraint
```

## Common Validation Errors

### Structural Errors
```yaml
# Missing required fields
error: "Missing required field 'name'"
solution: "Add 'name: EntityName' to model definition"

# Invalid field type
error: "Invalid field type 'text'"
solution: "Use 'string' instead of 'text'"

# Circular references
error: "Circular reference detected: User -> Company -> User"
solution: "Remove circular dependency or use indirect reference"
```

### Relationship Errors
```yaml
# Missing foreign key
error: "Belongs-to relationship missing foreign key"
solution: "Add foreign_key field with correct type"

# Invalid polymorphic target
error: "Invalid polymorphic target 'InvalidEntity'"
solution: "Ensure target entity exists and is exported"
```

### Validation Rule Errors
```yaml
# Invalid regex pattern
error: "Invalid regex pattern: '[unclosed bracket'"
solution: "Fix regex pattern syntax"

# Conflicting validation rules
error: "Conflicting validation: min_length 10, max_length 5"
solution: "Adjust validation rules to be compatible"
```

## Schema Validation Best Practices

### Field Validation Guidelines
```yaml
# ✅ Good field validation
email:
  type: email
  required: true
  unique: true
  indexed: true
  validate:
    - { rule: email_format }
    - { rule: max_length, value: 255 }

# ❌ Avoid overly complex validation
email:
  type: string
  validate:
    - { rule: custom, function: complex_email_validation }
    - { rule: custom, function: check_domain_reputation }
    - { rule: custom, function: verify_email_exists }
```

### Relationship Validation Guidelines
```yaml
# ✅ Clear relationship definition
user_id:
  type: uuid
  required: true
  indexed: true
  relation:
    type: belongs_to
    target: User
    foreign_key: id
    cascade: delete

# ❌ Ambiguous relationship
user:
  type: uuid
  relation: User  # Missing type and details
```

### Index Validation Guidelines
```yaml
# ✅ Appropriate indexing
indexes:
  - name: idx_user_email
    fields: [email]
    unique: true

  - name: idx_user_status_created
    fields: [status, created_at]

# ❌ Over-indexing
indexes:
  - name: idx_every_field
    fields: [id, email, username, status, created_at, updated_at]
```

## Schema Migration Validation

### Migration Validation Rules
```yaml
# Breaking changes must be explicit
breaking_changes:
  - remove_field
  - change_field_type
  - remove_entity
  - change_primary_key

# Safe changes can be automatic
safe_changes:
  - add_field
  - add_index
  - add_nullable_field
  - extend_enum_values

# Conditional changes require validation
conditional_changes:
  - modify_field_type  # Requires data validation
  - add_required_field # Requires default or migration
```