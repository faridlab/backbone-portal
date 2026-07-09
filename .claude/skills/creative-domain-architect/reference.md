# Reference Documentation for Creative Domain Architect

## Project Documentation References

### Domain-Driven Design Documentation
- **[DDD_BOUNDED_CONTEXTS.md](../../docs/technical/DDD_BOUNDED_CONTEXTS.md)** - Bounded context patterns and design
- **[DDD_IMPLEMENTATION_GUIDE.md](../../docs/DDD_IMPLEMENTATION_GUIDE.md)** - DDD implementation in Backbone
- **[FRAMEWORK.md](../../docs/technical/FRAMEWORK.md)** - Framework DDD tactical patterns
- **[MODULAR_MONOLITH_GUIDE.md](../../docs/MODULAR_MONOLITH_GUIDE.md)** - Modular monolith DDD architecture

### Architecture and Patterns
- **[ARCHITECTURE_PATTERNS.md](../../docs/technical/ARCHITECTURE_PATTERNS.md)** - Architectural patterns for domain design
- **[MANUAL_LOGIC_GUIDE.md](../../docs/technical/MANUAL_LOGIC_GUIDE.md)** - Adding business logic to generated code
- **[FINAL_ARCHITECTURE_DECISIONS.md](../../docs/technical/FINAL_ARCHITECTURE_DECISIONS.md)** - Key architectural decisions
- **[MODULE_ECOSYSTEM.md](../../docs/technical/MODULE_ECOSYSTEM.md)** - Module relationship patterns

### Schema and Generation
- **[SCHEMA/DATA_MODEL.md](../../docs/schema/DATA_MODEL.md)** - Schema data modeling for domains
- **[SCHEMA/TYPES.md](../../docs/schema/TYPES.md)** - Domain type system
- **[SCHEMA/WORKFLOW.md](../../docs/schema/WORKFLOW.md)** - Domain workflow modeling
- **[SCHEMA/GENERATION.md](../../docs/schema/GENERATION.md)** - Code generation for domain logic

## Domain Design Patterns

### Aggregate Root Pattern
```yaml
# libs/modules/{module}/schema/models/aggregate_root.model.yaml
name: Order
description: "Order aggregate root - manages order lifecycle and consistency"
version: "1.0"
type: aggregate_root

# Aggregate invariant rules
invariants:
  - name: order_total_must_be_positive
    description: "Order total must always be greater than 0"

  - name: cannot_modify_confirmed_order
    description: "Confirmed orders cannot be modified"

  - name: payment_required_for_confirmation
    description: "Order must have payment before confirmation"

fields:
  id:
    type: uuid
    primary_key: true
    auto_generate: true
    description: "Order identifier"

  order_number:
    type: string
    required: true
    unique: true
    indexed: true
    description: "Human-readable order number"

  customer_id:
    type: uuid
    required: true
    indexed: true
    relation:
      type: belongs_to
      target: Customer
      foreign_key: id

  status:
    type: enum
    values: [draft, pending_payment, confirmed, preparing, shipped, delivered, cancelled]
    default: draft
    indexed: true
    description: "Order status in lifecycle"

  total_amount:
    type: float
    required: true
    precision: 10
    scale: 2
    min: 0
    description: "Total order amount"

  currency:
    type: enum
    values: [USD, EUR, GBP]
    required: true
    default: USD
    description: "Order currency"

# Business methods (implemented in generated code)
business_methods:
  - name: add_item
    description: "Add item to order"
    parameters:
      - name: product_id
        type: uuid
      - name: quantity
        type: integer
    conditions:
      - status == draft
      - product_available
    effects:
      - calculate_total
      - emit_order_item_added_event

  - name: confirm_order
    description: "Confirm order for processing"
    parameters: []
    conditions:
      - status == pending_payment
      - has_payment
      - total_amount > 0
    effects:
      - status = confirmed
      - allocate_inventory
      - emit_order_confirmed_event

  - name: cancel_order
    description: "Cancel order"
    parameters:
      - name: reason
        type: string
    conditions:
      - status in [draft, pending_payment, confirmed]
      - cancellation_allowed
    effects:
      - status = cancelled
      - release_inventory
      - refund_payment
      - emit_order_cancelled_event

# Domain events
events:
  - OrderCreated
  - OrderItemAdded
  - OrderConfirmed
  - OrderCancelled
  - OrderShipped
  - OrderDelivered

# Aggregate rules
rules:
  - type: consistency
    description: "Order total must match sum of items"

  - type: state_transition
    transitions:
      - from: draft
        to: [pending_payment, cancelled]
      - from: pending_payment
        to: [confirmed, cancelled]
      - from: confirmed
        to: [preparing, cancelled]
```

### Entity Pattern
```yaml
# libs/modules/{module}/schema/models/entity.model.yaml
name: OrderItem
description: "Order item entity - part of Order aggregate"
version: "1.0"
type: entity

fields:
  id:
    type: uuid
    primary_key: true
    auto_generate: true

  order_id:
    type: uuid
    required: true
    indexed: true
    relation:
      type: belongs_to
      target: Order
      foreign_key: id

  product_id:
    type: uuid
    required: true
    indexed: true
    relation:
      type: belongs_to
      target: Product
      foreign_key: id

  product_name:
    type: string
    required: true
    max_length: 255
    description: "Product name snapshot"

  quantity:
    type: integer
    required: true
    min: 1
    description: "Item quantity"

  unit_price:
    type: float
    required: true
    precision: 10
    scale: 2
    min: 0
    description: "Unit price at time of order"

  total_price:
    type: float
    required: true
    precision: 10
    scale: 2
    min: 0
    description: "Total price (quantity * unit_price)"

# Business methods
business_methods:
  - name: update_quantity
    description: "Update item quantity"
    parameters:
      - name: new_quantity
        type: integer
    conditions:
      - new_quantity > 0
      - order.status == draft
    effects:
      - recalculate_total
      - update_order_total

  - name: apply_discount
    description: "Apply discount to item"
    parameters:
      - name: discount_percentage
        type: float
        min: 0
        max: 100
    conditions:
      - discount_allowed
    effects:
      - calculate_discounted_price
      - update_order_total
```

### Value Object Pattern
```yaml
# libs/modules/{module}/schema/models/value_object.model.yaml
name: Money
description: "Money value object - immutable monetary amount"
version: "1.0"
type: value_object

fields:
  amount:
    type: float
    required: true
    precision: 15
    scale: 2
    description: "Monetary amount"

  currency:
    type: enum
    values: [USD, EUR, GBP, JPY]
    required: true
    description: "Currency code"

# Value objects are immutable
metadata:
  immutable: true

# Business methods (pure functions)
business_methods:
  - name: add
    description: "Add two money values"
    parameters:
      - name: other
        type: Money
    conditions:
      - currency == other.currency
    returns: Money
    implementation: "new Money(amount + other.amount, currency)"

  - name: multiply
    description: "Multiply money by factor"
    parameters:
      - name: factor
        type: float
    returns: Money
    implementation: "new Money(amount * factor, currency)"

  - name: to_string
    description: "Format money as string"
    returns: string
    implementation: "format_currency(amount, currency)"
```

### Domain Service Pattern
```yaml
# Domain service definition (in workflow or service)
name: PricingService
description: "Domain service for pricing calculations"
version: "1.0"
type: domain_service

methods:
  - name: calculate_order_total
    description: "Calculate total for order with discounts and taxes"
    parameters:
      - name: order
        type: Order
      - name: discount_code
        type: string
        nullable: true
    returns:
      type: PricingResult
      properties:
        subtotal: Money
        discount_amount: Money
        tax_amount: Money
        total: Money
        applied_discounts: array

  - name: apply_promotion_rules
    description: "Apply applicable promotion rules"
    parameters:
      - name: customer
        type: Customer
      - name: order_items
        type: array
        items: OrderItem
    returns:
      type: array
      items: Promotion

# Service rules and invariants
rules:
  - type: business_rule
    description: "VIP customers get automatic 10% discount"
    condition: "customer.tier == vip"
    effect: "apply_percentage_discount(10)"

  - type: temporal_rule
    description: "Happy hour pricing on weekends"
    condition: "current_time.weekend && current_time.in_happy_hour"
    effect: "apply_percentage_discount(15)"
```

## Domain Event Patterns

### Event Definition
```yaml
# libs/modules/{module}/schema/events/{domain_event}.event.yaml
name: OrderConfirmed
description: "Order confirmed event - order ready for fulfillment"
version: "1.0"
type: domain_event

event_data:
  order_id:
    type: uuid
    required: true
    description: "Order identifier"

  customer_id:
    type: uuid
    required: true
    description: "Customer identifier"

  order_number:
    type: string
    required: true
    description: "Human-readable order number"

  total_amount:
    type: float
    required: true
    description: "Order total amount"

  currency:
    type: enum
    values: [USD, EUR, GBP]
    required: true
    description: "Order currency"

  confirmed_at:
    type: timestamp
    auto_now: true
    description: "Confirmation timestamp"

# Event metadata
metadata:
  event_version: "1.0"
  source_service: "order_service"
  correlation_id_field: "order_id"

# Event handlers (reactive)
handlers:
  - name: allocate_inventory
    service: inventory_service
    description: "Allocate inventory for order items"

  - name: create_shipment
    service: shipping_service
    description: "Create shipment for order fulfillment"

  - name: send_confirmation_email
    service: notification_service
    description: "Send order confirmation email to customer"

  - name: update_analytics
    service: analytics_service
    description: "Update order analytics"
```

### Saga Pattern
```yaml
# libs/modules/{module}/schema/sagas/{process}.saga.yaml
name: OrderFulfillmentSaga
description: "Order fulfillment saga - coordinates multiple services"
version: "1.0"
type: saga

trigger_event: OrderConfirmed

steps:
  - name: reserve_inventory
    service: inventory_service
    action: reserve_inventory
    compensation: release_inventory
    timeout: 30s

  - name: process_payment
    service: payment_service
    action: charge_customer
    compensation: refund_payment
    timeout: 60s
    retry_policy:
      max_attempts: 3
      backoff: exponential

  - name: create_shipment
    service: shipping_service
    action: create_shipment
    compensation: cancel_shipment
    timeout: 45s

  - name: send_confirmation
    service: notification_service
    action: send_order_confirmation
    compensation: send_cancellation_notice
    timeout: 15s

# Error handling
error_handling:
  on_step_failure:
    - execute_compensations
    - emit_order_failed_event
    - notify_customer

  on_timeout:
    - mark_order_as_failed
    - release_resources
    - escalate_to_human
```

## Bounded Context Patterns

### Context Mapping
```yaml
# libs/modules/{module}/schema/bounded_context.yaml
name: SalesContext
description: "Sales bounded context - handles orders, pricing, and customers"
version: "1.0"

# Core domain
core_domain:
  - Order Management
  - Pricing
  - Customer Management
  - Product Catalog

# Subdomains
subdomains:
  generic:
    - Notifications
    - Analytics
    - Reporting
  supporting:
    - Inventory
    - Shipping
    - Payment

# Context relationships
relationships:
  - name: CustomerSharedKernel
    type: shared_kernel
    target: CustomerContext
    shared_entities:
      - Customer
      - Address

  - name: InventoryPartnership
    type: partnership
    target: InventoryContext
    interactions:
      - check_availability
      - reserve_items
      - release_reservations

  - name: PaymentConformist
    type: conformist
    target: PaymentContext
    description: "Sales context uses payment context without influencing"

# Anti-corruption layer
anti_corruption_layers:
  - name: LegacyProductAdapter
    target: LegacyProductSystem
    description: "Adapts legacy product data to domain model"
  - name: ExternalShippingAdapter
    target: ExternalShippingAPI
    description: "Adapts external shipping service"

# Domain boundaries
boundaries:
  public_apis:
    - CreateOrder
    - GetOrderDetails
    - UpdateOrderStatus
    - CancelOrder

  private_apis:
    - CalculatePricing
    - ApplyDiscounts
    - ValidateOrderRules

  published_events:
    - OrderCreated
    - OrderConfirmed
    - OrderCancelled
    - OrderShipped
```

## Advanced Domain Patterns

### Specification Pattern
```yaml
# libs/modules/{module}/schema/specifications/{rule}.spec.yaml
name: CustomerIsPremiumSpecification
description: "Specification for premium customer eligibility"
version: "1.0"
type: specification

condition:
  type: composite
  operator: and
  rules:
    - field: customer.total_orders
      operator: gte
      value: 10
    - field: customer.total_spent
      operator: gte
      value: 1000
    - field: customer.account_age_days
      operator: gte
      value: 365

# Usage in business methods
business_rule_applications:
  - entity: Order
    method: apply_premium_discount
    specification: CustomerIsPremiumSpecification
    effect: "apply_percentage_discount(15)"
```

### Strategy Pattern
```yaml
# libs/modules/{module}/schema/strategies/{strategy}.strategy.yaml
name: PricingStrategy
description: "Strategy for different pricing calculations"
version: "1.0"
type: strategy

strategies:
  - name: StandardPricing
    description: "Standard pricing calculation"
    conditions:
      - customer.tier != vip
    implementation:
      algorithm: simple_markup
      markup_percentage: 20

  - name: VIPPricing
    description: "VIP customer pricing"
    conditions:
      - customer.tier == vip
    implementation:
      algorithm: percentage_discount
      discount_percentage: 10

  - name: BulkPricing
    description: "Bulk order pricing"
    conditions:
      - order.total_items > 100
    implementation:
      algorithm: tiered_pricing
      tiers:
        - min: 100
          max: 500
          discount: 5
        - min: 501
          max: 1000
          discount: 10
        - min: 1001
          discount: 15

# Strategy selection
selection_rules:
  - priority: 1
    strategy: VIPPricing
  - priority: 2
    strategy: BulkPricing
  - priority: 3
    strategy: StandardPricing
```