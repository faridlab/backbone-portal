# Domain Design Patterns

## Aggregate Root Patterns

### Order Aggregate Root
```yaml
name: Order
description: "Order aggregate root - manages order lifecycle and business rules"
version: "1.0"
type: aggregate_root

# Business invariants that must always hold
invariants:
  - order_total_must_be_positive
  - cannot_modify_shipped_order
  - payment_required_for_shipment
  - inventory_must_be_available

# Lifecycle state management
states:
  draft:
    description: "Order being created"
    allowed_transitions: [pending_payment, cancelled]

  pending_payment:
    description: "Awaiting payment confirmation"
    allowed_transitions: [confirmed, cancelled]

  confirmed:
    description: "Payment confirmed, preparing for shipment"
    allowed_transitions: [processing, cancelled]

  processing:
    description: "Order is being processed"
    allowed_transitions: [shipped]

  shipped:
    description: "Order has been shipped"
    allowed_transitions: [delivered]

  delivered:
    description: "Order has been delivered"
    allowed_transitions: [returned]

  cancelled:
    description: "Order has been cancelled"
    final: true

  returned:
    description: "Order has been returned"
    final: true

# Business methods with domain logic
business_methods:
  - name: add_item
    description: "Add item to order"
    parameters:
      - product_id: uuid
      - quantity: integer
    preconditions:
      - state == "draft"
      - quantity > 0
    business_logic:
      - check_product_availability
      - calculate_line_total
      - update_order_total
    postconditions:
      - order.total_updated
      - inventory_reserved
    events:
      - OrderItemAdded

  - name: apply_discount
    description: "Apply discount to order"
    parameters:
      - discount_code: string
    business_logic:
      - validate_discount_code
      - check_discount_applicability
      - calculate_discount_amount
      - apply_to_total
    events:
      - DiscountApplied

  - name: confirm_payment
    description: "Confirm payment and move to confirmed state"
    preconditions:
      - state == "pending_payment"
      - payment_received
    business_logic:
      - validate_payment_amount
      - allocate_inventory
      - update_state("confirmed")
    events:
      - OrderConfirmed
      - InventoryAllocated
      - PaymentConfirmed

  - name: ship_order
    description: "Ship order to customer"
    preconditions:
      - state == "processing"
      - all_items_processed
    business_logic:
      - generate_shipping_label
      - create_tracking_record
      - update_state("shipped")
    events:
      - OrderShipped

# Domain events published by this aggregate
events:
  OrderCreated
  OrderItemAdded
  OrderItemRemoved
  DiscountApplied
  PaymentReceived
  OrderConfirmed
  OrderCancelled
  OrderShipped
  OrderDelivered

# Integration events for other bounded contexts
integration_events:
  - OrderConfirmedEvent
    data:
      order_id
      customer_id
      total_amount
      items
    subscribers:
      - inventory_context
      - shipping_context
      - analytics_context

  - OrderShippedEvent
    data:
      order_id
      customer_id
      shipping_address
      tracking_number
    subscribers:
      - notification_context
      - tracking_context
```

### Customer Aggregate Root
```yaml
name: Customer
description: "Customer aggregate root - manages customer information and relationships"
version: "1.0"
type: aggregate_root

invariants:
  - email_must_be_unique
  - at_least_one_address_required
  - credit_limit_must_not_be_exceeded
  - loyalty_points_must_be_non_negative

business_methods:
  - name: register_customer
    description: "Register new customer"
    parameters:
      - email: string
      - personal_info: PersonalInfo
    business_logic:
      - validate_email_uniqueness
      - create_customer_profile
      - assign_customer_number
      - set_initial_tier
    events:
      - CustomerRegistered

  - name: add_address
    description: "Add shipping address"
    parameters:
      - address: Address
      - address_type: AddressType
    business_logic:
      - validate_address_format
      - check_address_limits
      - set_primary_if_first
    events:
      - AddressAdded

  - name: update_loyalty_tier
    description: "Update customer loyalty tier based on activity"
    business_logic:
      - calculate_total_spent
      - determine_appropriate_tier
      - apply_tier_benefits
    events:
      - LoyaltyTierUpdated

  - name: place_order
    description: "Place order against customer account"
    parameters:
      - order_data: OrderData
    business_logic:
      - check_credit_limit
      - validate_order_permissions
      - reserve_credit
    events:
      - OrderPlaced
```

## Entity Patterns

### OrderItem Entity
```yaml
name: OrderItem
description: "Order item entity - represents individual product in order"
version: "1.0"
type: entity

# Entity belongs to Order aggregate
aggregate_root: Order

business_methods:
  - name: update_quantity
    description: "Update item quantity"
    parameters:
      - new_quantity: integer
    preconditions:
      - new_quantity > 0
      - Order.state == "draft"
    business_logic:
      - check_inventory_availability
      - calculate_new_total
      - update_aggregate_total
    events:
      - OrderItemUpdated

  - name: apply_item_discount
    description: "Apply discount specific to this item"
    parameters:
      - discount: ItemDiscount
    business_logic:
      - validate_discount_applicability
      - calculate_discounted_price
      - update_line_total
    events:
      - ItemDiscountApplied
```

### Product Entity
```yaml
name: Product
description: "Product entity - represents product in catalog"
version: "1.0"
type: entity

business_methods:
  - name: reserve_inventory
    description: "Reserve inventory for order"
    parameters:
      - quantity: integer
      - order_id: uuid
    preconditions:
      - quantity <= available_quantity
    business_logic:
      - update_reserved_quantity
      - update_available_quantity
    events:
      - InventoryReserved

  - name: release_inventory
    description: "Release reserved inventory"
    parameters:
      - quantity: integer
      - order_id: uuid
    business_logic:
      - update_reserved_quantity
      - update_available_quantity
    events:
      - InventoryReleased
```

## Value Object Patterns

### Money Value Object
```yaml
name: Money
description: "Money value object - represents monetary amount with currency"
version: "1.0"
type: value_object

# Value objects are immutable
immutable: true

fields:
  amount:
    type: float
    precision: 15
    scale: 2
    min: 0
    required: true

  currency:
    type: enum
    values: [USD, EUR, GBP, JPY]
    required: true

# Value object operations (pure functions)
operations:
  - name: add
    description: "Add two money amounts"
    parameters:
      - other: Money
    conditions:
      - same_currency
    returns: Money

  - name: subtract
    description: "Subtract money amount"
    parameters:
      - other: Money
    conditions:
      - same_currency
      - amount >= other.amount
    returns: Money

  - name: multiply
    description: "Multiply by factor"
    parameters:
      - factor: float
    returns: Money

  - name: divide
    description: "Divide by divisor"
    parameters:
      - divisor: float
    conditions:
      - divisor != 0
    returns: Money
```

### Address Value Object
```yaml
name: Address
description: "Address value object - represents postal address"
version: "1.0"
type: value_object

immutable: true

fields:
  street_line1:
    type: string
    required: true
    max_length: 255

  street_line2:
    type: string
    nullable: true
    max_length: 255

  city:
    type: string
    required: true
    max_length: 100

  state:
    type: string
    required: true
    max_length: 50

  postal_code:
    type: string
    required: true
    max_length: 20

  country:
    type: string
    required: true
    max_length: 2  # ISO 3166-1 alpha-2

# Value object validations
validations:
  - name: valid_postal_code
    rule: postal_code_format_for_country
  - name: valid_state_for_country
    rule: state_exists_in_country
```

## Domain Service Patterns

### PricingService
```yaml
name: PricingService
description: "Domain service for complex pricing calculations"
version: "1.0"
type: domain_service

services:
  - name: calculate_order_total
    description: "Calculate total with discounts and taxes"
    parameters:
      - order: Order
      - customer: Customer
      - discount_code: string
    returns: PricingResult
    business_logic:
      - calculate_subtotal
      - apply_customer_discounts
      - apply_promotion_discounts
      - calculate_taxes
      - calculate_shipping
      - return_total_breakdown

  - name: determine_product_pricing
    description: "Determine pricing strategy for product"
    parameters:
      - product: Product
      - customer: Customer
      - quantity: integer
    returns: ProductPricing
    business_logic:
      - get_base_price
      - apply_quantity_discounts
      - apply_customer_tier_pricing
      - apply_time_based_pricing
      - return_final_pricing

  - name: validate_discount_code
    description: "Validate and apply discount code"
    parameters:
      - code: string
      - order: Order
      - customer: Customer
    returns: DiscountValidation
    business_logic:
      - check_code_existence
      - validate_code_active
      - check_usage_limits
      - validate_applicable_products
      - calculate_discount_amount
```

### InventoryService
```yaml
name: InventoryService
description: "Domain service for inventory management"
version: "1.0"
type: domain_service

services:
  - name: check_availability
    description: "Check if items are available"
    parameters:
      - items: array[OrderItem]
    returns: AvailabilityResult
    business_logic:
      - check_each_item_availability
      - aggregate_available_quantities
      - return_availability_status

  - name: reserve_items
    description: "Reserve inventory for order"
    parameters:
      - items: array[OrderItem]
      - order_id: uuid
    returns: ReservationResult
    business_logic:
      - validate_reservation_possible
      - create_reservation_records
      - update_available_quantities
      - return_reservation_confirmation

  - name: release_reservation
    description: "Release inventory reservation"
    parameters:
      - reservation_id: uuid
    business_logic:
      - find_reservation_records
      - restore_available_quantities
      - remove_reservation_records
```

## Domain Event Patterns

### Event Sourcing Pattern
```yaml
name: OrderEvent
description: "Base event for order aggregate"
version: "1.0"
type: domain_event

# Event structure
event_fields:
  aggregate_id:
    type: uuid
    required: true
  event_id:
    type: uuid
    auto_generate: true
  event_version:
    type: string
    default: "1.0"
  occurred_at:
    type: timestamp
    auto_now: true
  event_type:
    type: string
    required: true

# Specific event types
events:
  OrderCreated:
    description: "Order created event"
    fields:
      customer_id: uuid
      initial_items: array[OrderItemData]

  OrderConfirmed:
    description: "Order confirmed event"
    fields:
      payment_method: PaymentMethod
      payment_amount: Money
      confirmed_by: string

  OrderShipped:
    description: "Order shipped event"
    fields:
      shipping_address: Address
      tracking_number: string
      estimated_delivery: timestamp
```

### Event Pattern
```yaml
name: DomainEventPattern
description: "Pattern for defining domain events"
version: "1.0"

# Event naming conventions
naming:
  past_tense: true  # Use past tense for events
  descriptive: true # Include what happened

# Event data principles
data_principles:
  include_all_context: true
  immutable_data: true
  versioned_schema: true
  no_derived_data: true

# Event handlers
handler_patterns:
  - name: event_handler
    description: "Handle domain event"
    parameters:
      - event: DomainEvent
    effects:
      - update_read_model
      - trigger_saga
      - send_notification

  - name: saga_handler
    description: "Handle saga events"
    parameters:
      - event: IntegrationEvent
    effects:
      - continue_saga
      - compensate_saga
      - complete_saga
```

## Specification Patterns

### Customer Specification
```yaml
name: CustomerSpecification
description: "Business specifications for customer eligibility"
version: "1.0"
type: specification

specifications:
  - name: IsPremiumCustomer
    description: "Customer is premium tier"
    condition:
      type: field_comparison
      field: customer.tier
      operator: equals
      value: premium

  - name: HasSufficientCredit
    description: "Customer has sufficient credit for order"
    parameters:
      - order_amount: Money
    condition:
      type: expression
      expression: "customer.available_credit >= order_amount"

  - name: IsInGoodStanding
    description: "Customer account is in good standing"
    condition:
      type: composite
      operator: and
      rules:
        - field: customer.account_status
          operator: equals
          value: active
        - field: customer.overdue_payments
          operator: equals
          value: 0

# Composite specifications
composites:
  - name: CanPlaceOrder
    description: "Customer can place an order"
    specification:
      type: composite
      operator: and
      specifications:
        - IsInGoodStanding
        - HasSufficientCredit
        - (parameterized: order_amount)

  - name: IsVIPCustomer
    description: "Customer meets VIP criteria"
    specification:
      type: composite
      operator: and
      specifications:
        - IsPremiumCustomer
        - TotalOrdersGreaterThan
          threshold: 50
        - TotalSpentGreaterThan
          threshold: 10000
```