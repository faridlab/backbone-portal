# Reference Documentation for Backbone Schema Maintainer

## Project Documentation References

### Core Schema Documentation
- **[README.md](../../../docs/schema/README.md)** - Schema system overview and quick start
- **[DATA_MODEL.md](../../../docs/schema/DATA_MODEL.md)** - Entity and data structure definitions
- **[HOOK.md](../../../docs/schema/HOOK.md)** - Entity lifecycle: state machines, rules, permissions
- **[WORKFLOW.md](../../../docs/schema/WORKFLOW.md)** - Multi-step business process orchestration (Saga)
- **[TYPES.md](../../../docs/schema/TYPES.md)** - Type system reference and JSONB validation
- **[GENERATION.md](../../../docs/schema/GENERATION.md)** - Code generation targets and CLI

### DDD/Clean Architecture Documentation
- **[ARCHITECTURE.md](../../../docs/schema/ARCHITECTURE.md)** - DDD layers and schema structure overview
- **[DOMAIN.md](../../../docs/schema/DOMAIN.md)** - Domain layer: entities, value objects, events, domain services
- **[APPLICATION.md](../../../docs/schema/APPLICATION.md)** - Application layer: use cases, services, event handlers
- **[INFRASTRUCTURE.md](../../../docs/schema/INFRASTRUCTURE.md)** - Infrastructure layer: projections, event store, repositories
- **[PRESENTATION.md](../../../docs/schema/PRESENTATION.md)** - Presentation layer: HTTP routes, gRPC, DTOs, versioning
- **[INTEGRATION.md](../../../docs/schema/INTEGRATION.md)** - Module integration: dependencies, exports, event subscriptions

### Cross-Cutting Concerns
- **[AUTHORIZATION.md](../../../docs/schema/AUTHORIZATION.md)** - Permissions, roles, policies (RBAC/ABAC)
- **[VALIDATION_RULES.md](../../../docs/schema/VALIDATION_RULES.md)** - Field, entity, and async validation rules
- **[IMPLEMENTATION_STRATEGY.md](../../../docs/schema/IMPLEMENTATION_STRATEGY.md)** - Implementation phases and generator architecture

### Additional Documentation
- **[OPENAPI.md](../../../docs/schema/OPENAPI.md)** - REST API specification generation
- **[EXAMPLES.md](../../../docs/schema/EXAMPLES.md)** - Complete examples for common patterns
- **[ERROR_CODES.md](../../../docs/schema/ERROR_CODES.md)** - Error code reference and troubleshooting
- **[MIGRATION_GUIDE.md](../../../docs/schema/MIGRATION_GUIDE.md)** - Migrate from DSL to YAML format

### Framework Integration
- **[FRAMEWORK.md](../../../docs/technical/FRAMEWORK.md)** - Framework schema integration
- **[MODULE_ECOSYSTEM.md](../../../docs/technical/MODULE_ECOSYSTEM.md)** - Module schema patterns
- **[MANUAL_LOGIC_GUIDE.md](../../../docs/technical/MANUAL_LOGIC_GUIDE.md)** - Manual logic addition to generated code

## Backbone Schema Structure

### Module Schema Organization
```
libs/modules/{module}/schema/
├── models/                 # Entity and value object definitions
│   ├── {entity}.model.yaml
│   └── index.model.yaml   # Shared types and imports
├── hooks/                  # Event-driven hooks
│   └── {workflow}.hook.yaml
├── workflows/              # Business process workflows
│   └── {process}.workflow.yaml
├── openapi/               # OpenAPI extensions
│   └── {api}.yaml
└── schema.yaml            # Module schema configuration
```

### Schema File Types

#### Models (*.model.yaml)
Entity and value object definitions with:
- Field definitions and types
- Validation rules
- Relationships and references
- Indexes and constraints
- Metadata configuration

#### Hooks (*.hook.yaml)
Event-driven architecture hooks:
- Before/after create/update/delete hooks
- Domain event publications
- External service integrations
- State machine transitions

#### Workflows (*.workflow.yaml)
Business process orchestrations:
- Multi-step workflows
- Conditional logic
- Human approval steps
- Automated processes

#### OpenAPI Extensions
API specification enhancements:
- Custom endpoints
- Authentication patterns
- Rate limiting rules
- Response transformations

## Schema Field Types Reference

### Basic Types
```yaml
string:         # Text data
  min_length: 1
  max_length: 255
  pattern: "^[a-zA-Z0-9]+$"

integer:        # Numeric data
  min: 0
  max: 1000000

float:          # Decimal data
  precision: 2
  scale: 2

boolean:        # True/false values
  default: false

timestamp:      # Date/time values
  auto_now: true

uuid:           # Unique identifiers
  auto_generate: true

email:          # Email addresses
  verified: true

password:       # Hashed passwords
  min_strength: 8
```

### Complex Types
```yaml
object:         # Nested objects
  properties:
    name: { type: string }
    age: { type: integer }

array:          # Collections
  items: { type: string }
  min_items: 1
  max_items: 100

enum:           # Fixed values
  values: [active, inactive, pending]

json:           # Flexible JSON data
  schema: {...}  # Optional JSON schema
```

### Validation Rules
```yaml
# Field validation
field_name:
  type: string
  required: true
  unique: true
  indexed: true
  validate:
    - { rule: email }
    - { rule: min_length, value: 5 }

# Custom validation
custom_field:
  type: string
  validate:
    - {
        rule: custom,
        function: validate_business_id,
        message: "Invalid business ID format"
      }
```

## Schema Relationships

### One-to-Many
```yaml
# User has many Posts
User:
  fields:
    posts:
      type: array
      relation: many
      target: Post
      foreign_key: user_id

Post:
  fields:
    user:
      type: uuid
      relation: one
      target: User
      required: true
```

### Many-to-Many
```yaml
# Users have many Roles
User:
  fields:
    roles:
      type: array
      relation: many_many
      target: Role
      through: UserRole

Role:
  fields:
    users:
      type: array
      relation: many_many
      target: User
      through: UserRole
```

### Polymorphic
```yaml
# Generic attachments
Attachment:
  fields:
    attachable:
      type: uuid
      relation: polymorphic
      types: [User, Post, Comment]
      required: true
```

## Schema Hooks Reference

### Entity Hooks
```yaml
# Before create hook
before_create:
  trigger: user.before_create
  actions:
    - type: validate
      rules: [email_unique, password_strength]
    - type: transform
      field: email
      function: lowercase
    - type: set
      field: created_at
      value: "{{now}}"

# After update hook
after_update:
  trigger: user.after_update
  actions:
    - type: event
      event: UserUpdated
      data: {...}
    - type: notify
      service: postman
      template: user_updated
```

### Workflow Hooks
```yaml
# State machine hook
state_change:
  trigger: order.status_change
  from: pending
  to: processing
  actions:
    - type: validate
      condition: payment_received
    - type: create
      entity: OrderHistory
      data: {...}
    - type: notify
      service: postman
      template: order_processing
```

## Schema Workflows

### Business Process Workflow
```yaml
# User onboarding workflow
name: UserOnboarding
trigger: user.created
steps:
  - name: send_welcome_email
    type: service
    service: postman
    action: send_email
    template: welcome

  - name: create_default_profile
    type: create
    entity: UserProfile
    data:
      user_id: "{{user.id}}"
      theme: default

  - name: assign_default_roles
    type: update
    entity: User
    where: { id: "{{user.id}}" }
    data:
      roles: ["user"]

  - name: notify_admin
    type: condition
    condition: user.type == "premium"
    true:
      - type: notify
        service: admin
        template: premium_user_joined
```

## Generation Targets

### Core Generation
```yaml
# Rust entities and value objects
rust:
  path: src/domain/entities/
  template: entity.rs.j2

# SQL tables and migrations
sql:
  path: migrations/
  template: table.sql.j2

# Repository implementations
repository:
  path: src/infrastructure/persistence/
  template: repository.rs.j2
```

### Advanced Generation
```yaml
# gRPC services
grpc:
  path: src/presentation/grpc/
  template: service.rs.j2

# REST handlers
handler:
  path: src/presentation/http/
  template: handler.rs.j2

# OpenAPI specifications
openapi:
  path: docs/openapi/
  template: openapi.yaml.j2
```

## Schema Validation

### Validation Rules
```yaml
# Field-level validation
fields:
  email:
    type: email
    required: true
    unique: true
    validate:
      - { rule: email_format }
      - { rule: mx_record }

  age:
    type: integer
    validate:
      - { rule: min, value: 18 }
      - { rule: max, value: 120 }
```

### Cross-Field Validation
```yaml
# Schema-level validation
validation:
  - name: password_confirmation
    fields: [password, password_confirmation]
    rule: equal
    message: "Passwords must match"

  - name: date_range
    fields: [start_date, end_date]
    rule: end_after_start
    message: "End date must be after start date"
```

## Schema Migration

### Migration Types
```yaml
# Field changes
add_field:
  entity: User
  field: profile_image
  type: string
  nullable: true

remove_field:
  entity: User
  field: old_field

modify_field:
  entity: User
  field: email
  type: string
  max_length: 500

# Entity changes
create_entity:
  name: UserProfile
  fields:
    user_id: { type: uuid, required: true }
    bio: { type: text }

rename_entity:
  from: OldName
  to: NewName
```

### Migration Strategies
```yaml
# Safe migration with rollback
migration:
  type: multi_step
  steps:
    - add_column: profile_image
    - backfill_data: "UPDATE users SET profile_image = NULL"
    - add_constraint: "ALTER TABLE users ADD CONSTRAINT check_profile_image..."
    - drop_column: old_avatar_field
  rollback: true
```