# Backbone Schema Patterns

## Entity Schema Patterns

### Basic Entity Pattern
```yaml
# libs/modules/{module}/schema/models/{entity}.model.yaml
name: User
description: "User account entity"
version: "1.0"

fields:
  id:
    type: uuid
    primary_key: true
    auto_generate: true
    description: "Unique user identifier"

  email:
    type: email
    required: true
    unique: true
    indexed: true
    description: "User email address"
    validate:
      - { rule: email_format }
      - { rule: max_length, value: 255 }

  username:
    type: string
    required: true
    unique: true
    indexed: true
    min_length: 3
    max_length: 50
    pattern: "^[a-zA-Z0-9_]+$"
    description: "Unique username"

  status:
    type: enum
    values: [active, inactive, suspended, pending]
    default: pending
    indexed: true
    description: "Account status"

  created_at:
    type: timestamp
    auto_now: true
    description: "Account creation timestamp"

  updated_at:
    type: timestamp
    auto_now_update: true
    description: "Last update timestamp"

indexes:
  - name: idx_user_email
    fields: [email]
    unique: true

  - name: idx_user_status
    fields: [status]

metadata:
  soft_delete: true
  audit_fields: true
  timestamps: true
```

### Entity with Relationships Pattern
```yaml
name: UserProfile
description: "Extended user profile information"
version: "1.0"

fields:
  id:
    type: uuid
    primary_key: true
    auto_generate: true

  user_id:
    type: uuid
    required: true
    indexed: true
    description: "Reference to User entity"
    relation:
      type: one
      target: User
      foreign_key: id

  first_name:
    type: string
    required: true
    max_length: 100
    description: "First name"

  last_name:
    type: string
    required: true
    max_length: 100
    description: "Last name"

  avatar_url:
    type: string
    nullable: true
    max_length: 500
    validate:
      - { rule: url }
    description: "Profile image URL"

  preferences:
    type: object
    nullable: true
    description: "User preferences"
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
        default: en

relationships:
  - name: user
    type: belongs_to
    target: User
    foreign_key: user_id
    cascade: delete

indexes:
  - name: idx_user_profile_user_id
    fields: [user_id]
    unique: true

metadata:
  soft_delete: true
```

### Polymorphic Relationship Pattern
```yaml
name: Attachment
description: "Generic file attachment for multiple entities"
version: "1.0"

fields:
  id:
    type: uuid
    primary_key: true
    auto_generate: true

  attachable_id:
    type: uuid
    required: true
    indexed: true
    description: "ID of attached entity"

  attachable_type:
    type: enum
    values: [User, Post, Comment, Document]
    required: true
    indexed: true
    description: "Type of attached entity"

  filename:
    type: string
    required: true
    max_length: 255
    description: "Original filename"

  file_path:
    type: string
    required: true
    max_length: 500
    description: "Storage path"

  file_size:
    type: integer
    required: true
    description: "File size in bytes"

  mime_type:
    type: string
    required: true
    max_length: 100
    description: "MIME type"

indexes:
  - name: idx_attachment_attachable
    fields: [attachable_type, attachable_id]

metadata:
  soft_delete: true
```

## Value Object Patterns

### Email Value Object
```yaml
name: Email
description: "Email address value object with validation"
version: "1.0"
type: value_object

fields:
  address:
    type: string
    required: true
    max_length: 255
    validate:
      - { rule: email_format }
      - { rule: mx_check }
    description: "Email address"

  verified:
    type: boolean
    default: false
    description: "Email verification status"

  verification_token:
    type: string
    nullable: true
    max_length: 255
    description: "Email verification token"

metadata:
  immutable: [address]  # Address cannot be changed after creation
```

### Money Value Object
```yaml
name: Money
description: "Monetary amount with currency"
version: "1.0"
type: value_object

fields:
  amount:
    type: float
    required: true
    precision: 10
    scale: 2
    min: 0
    description: "Monetary amount"

  currency:
    type: enum
    values: [USD, EUR, GBP, JPY, CNY]
    required: true
    default: USD
    description: "Currency code"

metadata:
  immutable: true  # Money objects are immutable
```

## Hook Patterns

### Entity Lifecycle Hooks
```yaml
# libs/modules/{module}/schema/hooks/{entity}_lifecycle.hook.yaml
name: UserLifecycle
description: "User entity lifecycle hooks"
version: "1.0"

hooks:
  before_create:
    trigger: user.before_create
    actions:
      - type: validate
        rules: [email_unique, username_format]

      - type: transform
        field: email
        function: lowercase

      - type: set
        field: status
        value: pending

      - type: generate
        field: verification_token
        function: random_string
        length: 32

  after_create:
    trigger: user.after_create
    actions:
      - type: event
        event: UserCreated
        data:
          user_id: "{{entity.id}}"
          email: "{{entity.email}}"

      - type: workflow
        workflow: UserOnboarding
        context:
          user_id: "{{entity.id}}"

  before_update:
    trigger: user.before_update
    actions:
      - type: validate
        condition: "entity.email != original.email"
        true:
          - type: transform
            field: email
            function: lowercase

          - type: set
            field: email_verified
            value: false

  after_update:
    trigger: user.after_update
    condition: "entity.status != original.status"
    true:
      - type: event
        event: UserStatusChanged
        data:
          user_id: "{{entity.id}}"
          old_status: "{{original.status}}"
          new_status: "{{entity.status}}"
```

### Business Logic Hooks
```yaml
name: PasswordReset
description: "Password reset business logic"
version: "1.0"

hooks:
  before_update:
    trigger: user.before_update
    condition: "entity.password != original.password"
    actions:
      - type: validate
        rules: [password_strength]

      - type: transform
        field: password
        function: hash_password

      - type: set
        field: password_changed_at
        value: "{{now}}"

      - type: set
        field: password_reset_token
        value: null

      - type: event
        event: PasswordChanged
        data:
          user_id: "{{entity.id}}"
          changed_at: "{{now}}"
```

## Workflow Patterns

### User Onboarding Workflow
```yaml
# libs/modules/{module}/schema/workflows/user_onboarding.workflow.yaml
name: UserOnboarding
description: "New user onboarding process"
version: "1.0"
trigger: user.created

steps:
  - name: send_welcome_email
    type: service
    service: postman
    action: send_email
    template: welcome_email
    data:
      user_id: "{{user.id}}"
      email: "{{user.email}}"
      verification_token: "{{user.verification_token}}"

  - name: create_user_profile
    type: create
    entity: UserProfile
    data:
      user_id: "{{user.id}}"
      theme: default
      notifications: true

  - name: assign_default_permissions
    type: create
    entity: UserRole
    data:
      user_id: "{{user.id}}"
      role_id: "{{roles.user}}"

  - name: setup_default_preferences
    type: update
    entity: User
    where: { id: "{{user.id}}" }
    data:
      preferences:
        theme: auto
        notifications: true
        language: en

  - name: audit_log
    type: create
    entity: AuditLog
    data:
      action: user_onboarding_completed
      entity_type: User
      entity_id: "{{user.id}}"
      data:
        steps_completed: 4
        completed_at: "{{now}}"

error_handling:
  on_failure:
    - type: notify
      service: admin
      template: onboarding_failed
      data:
        user_id: "{{user.id}}"
        error: "{{error}}"

    - type: rollback
      steps: [create_user_profile, assign_default_permissions]
```

### Approval Workflow Pattern
```yaml
name: DocumentApproval
description: "Document approval workflow"
version: "1.0"
trigger: document.created

states:
  - name: draft
    description: "Document is being drafted"

  - name: pending_review
    description: "Awaiting manager review"

  - name: approved
    description: "Document approved"

  - name: rejected
    description: "Document rejected"

transitions:
  - from: draft
    to: pending_review
    action: submit_for_review
    guard: "document.content != null"

  - from: pending_review
    to: approved
    action: approve
    role: manager
    condition: "document.risk_level != high"

  - from: pending_review
    to: approved
    action: approve
    role: senior_manager
    condition: "document.risk_level == high"

  - from: pending_review
    to: rejected
    action: reject
    role: manager

steps:
  - name: notify_reviewer
    state: pending_review
    type: service
    service: postman
    action: send_email
    template: document_review
    data:
      reviewer: "{{document.reviewer}}"
      document_url: "{{document.url}}"

  - name: archive_document
    state: approved
    type: update
    entity: Document
    where: { id: "{{document.id}}" }
    data:
      archived_at: "{{now}}"
```

## Validation Patterns

### Cross-Field Validation
```yaml
name: UserValidation
description: "Cross-field validation for User entity"
version: "1.0"

validation:
  - name: password_confirmation_match
    fields: [password, password_confirmation]
    rule: equal
    message: "Password and confirmation must match"

  - name: adult_user
    fields: [birth_date]
    rule: age_at_least
    value: 18
    message: "User must be at least 18 years old"

  - name: unique_username_in_company
    fields: [username, company_id]
    rule: unique_composite
    message: "Username must be unique within company"

custom_validators:
  age_at_least:
    function: validate_minimum_age
    parameters: [min_age]

  unique_composite:
    function: validate_unique_composite
    parameters: [fields]
```

### Business Rule Validation
```yaml
name: OrderValidation
description: "Order business rule validation"
version: "1.0"

validation:
  - name: sufficient_inventory
    condition: "order.quantity <= product.inventory"
    message: "Insufficient inventory for order"

  - name: order_minimum
    condition: "order.total >= 50.00"
    message: "Order total must be at least $50.00"

  - name: business_hours
    condition: "order.created_at.business_hours == true"
    message: "Orders can only be placed during business hours"

  - name: credit_limit
    condition: "customer.total_orders + order.total <= customer.credit_limit"
    message: "Order exceeds customer credit limit"
```