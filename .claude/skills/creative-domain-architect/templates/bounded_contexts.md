# Bounded Context Design Patterns

## Context Mapping Patterns

### Sales Context
```yaml
name: SalesContext
description: "Sales bounded context - handles orders, customers, and pricing"
version: "1.0"

# Core domain model
core_domain:
  - Order (Aggregate Root)
  - Customer (Aggregate Root)
  - Product (Entity)
  - Money (Value Object)
  - Address (Value Object)

# Subdomains classification
subdomains:
  core:
    - Order Management
    - Customer Management
    - Pricing
  supporting:
    - Product Catalog
    - Discount Management
  generic:
    - Notifications
    - Audit Logging

# Context relationships
context_mapping:
  # Shared Kernel with Customer context
  - type: SharedKernel
    name: CustomerSharedKernel
    target: CustomerContext
    shared_elements:
      - Customer
      - Address
      - PersonalInfo
    integration_mechanism: shared_database

  # Partnership with Inventory context
  - type: Partnership
    name: InventoryPartnership
    target: InventoryContext
    collaboration:
      - Sales checks inventory availability
      - Inventory provides reservation capabilities
    integration_mechanism: rest_api

  # Customer-Supplier relationship with Payment context
  - type: CustomerSupplier
    name: PaymentProvider
    customer: SalesContext
    supplier: PaymentContext
    contract:
      - Process payment requests
      - Provide payment status
      - Handle refunds
    integration_mechanism: message_queue

  # Conformist relationship with Shipping context
  - type: Conformist
    name: ShippingAdapter
    upstream: ShippingContext
    downstream: SalesContext
    adaptation:
      - Adapt shipping rates to domain model
      - Map shipping statuses to order states

# Anti-corruption layers
anti_corruption_layers:
  - name: LegacyProductAdapter
    target_system: LegacyERP
    purpose: Translate legacy product data to domain model
    implementation: adapter_pattern

  - name: ExternalTaxServiceAdapter
    target_system: ExternalTaxAPI
    purpose: Integrate external tax calculations
    implementation: facade_pattern

# Context boundaries
boundaries:
  public_apis:
    - CreateOrder
    - GetOrderDetails
    - UpdateOrderStatus
    - CancelOrder
    - GetCustomerOrders

  published_events:
    - OrderCreated
    - OrderConfirmed
    - OrderCancelled
    - OrderShipped
    - CustomerRegistered

  external_dependencies:
    - InventoryService
    - PaymentGateway
    - ShippingCalculator
    - TaxCalculator
```

### Inventory Context
```yaml
name: InventoryContext
description: "Inventory bounded context - manages product inventory and reservations"
version: "1.0"

core_domain:
  - Product (Aggregate Root)
  - InventoryItem (Entity)
  - StockLocation (Entity)
  - Reservation (Entity)
  - Quantity (Value Object)

subdomains:
  core:
    - Inventory Management
    - Stock Control
  supporting:
    - Location Management
    - Reorder Management
  generic:
    - Auditing
    - Reporting

context_mapping:
  # Partnership with Sales context
  - type: Partnership
    name: SalesPartnership
    target: SalesContext
    shared_responsibilities:
      - Inventory availability checking
      - Stock reservation management

  # Published Language for product information
  - type: PublishedLanguage
    name: ProductCatalog
    target: ProductContext
    shared_vocabulary:
      - Product
      - SKU
      - ProductCategory

# Ubiquitous Language
ubiquitous_language:
  terms:
    - Stock: Physical inventory count
    - Reserved: Items allocated to orders
    - Available: Stock minus reservations
    - ReorderPoint: Minimum stock level for reordering
    - LeadTime: Time to replenish stock
```

### Payment Context
```yaml
name: PaymentContext
description: "Payment bounded context - handles payments, refunds, and payment methods"
version: "1.0"

core_domain:
  - Payment (Aggregate Root)
  - PaymentMethod (Entity)
  - Refund (Aggregate Root)
  - Transaction (Entity)
  - Amount (Value Object)

subdomains:
  core:
    - Payment Processing
    - Refund Management
  supporting:
    - Payment Method Management
    - Transaction History
  generic:
    - Audit Logging
    - Reporting

# Payment workflow
payment_workflow:
  initiate_payment:
    - ValidatePaymentDetails
    - CheckPaymentMethodValidity
    - CreatePaymentTransaction

  process_payment:
    - AuthorizePayment
    - CapturePayment
    - UpdatePaymentStatus

  handle_refund:
    - ValidateRefundRequest
    - ProcessRefund
    - UpdateRefundStatus
```

## Bounded Context Integration Patterns

### Integration Between Sales and Inventory
```yaml
name: SalesInventoryIntegration
description: "Integration patterns between Sales and Inventory contexts"
version: "1.0"

integration_style: EventDrivenArchitecture

# Shared events
shared_events:
  - OrderCreated (Sales → Inventory)
  - InventoryReserved (Inventory → Sales)
  - InventoryReleased (Inventory → Sales)
  - StockUpdated (Inventory → Sales)

# Event contracts
event_contracts:
  OrderCreatedEvent:
    schema:
      order_id: uuid
      customer_id: uuid
      items:
        - product_id: uuid
        - quantity: integer
      created_at: timestamp

  InventoryReservedEvent:
    schema:
      reservation_id: uuid
      order_id: uuid
      items:
        - product_id: uuid
        - quantity: reserved
        - available: integer
      expires_at: timestamp

# Message routing
message_routing:
  sales_to_inventory:
    exchange: sales.events
    routing_key: order.created
    handler: InventoryService.reserveItems

  inventory_to_sales:
    exchange: inventory.events
    routing_key: reservation.completed
    handler: OrderService.handleReservation
```

### Integration with External Systems
```yaml
name: ExternalSystemIntegration
description: "Patterns for integrating with external systems"
version: "1.0"

# Anti-corruption layer patterns
anti_corruption_patterns:

  # Facade Pattern
  facade_pattern:
    name: ShippingProviderFacade
    target_system: ExternalShippingAPI
    purpose: Simplify complex shipping API
    interface:
      - calculateShippingRate
      - createShipment
      - trackShipment
    implementation:
      - Translate domain models to external API format
      - Handle API rate limiting and errors
      - Provide consistent interface

  # Adapter Pattern
  adapter_pattern:
    name: LegacyProductAdapter
    target_system: LegacyERPSystem
    purpose: Adapt legacy product data
    interface:
      - getProductDetails
      - getProductPrice
      - updateProductStock
    implementation:
      - Map legacy data structures to domain models
      - Handle data transformation
      - Provide caching layer

  # Translation Layer
  translation_layer:
    name: TaxCalculationTranslator
    target_system: ExternalTaxService
    purpose: Translate tax calculations
    translations:
      - SalesOrder → TaxCalculationRequest
      - TaxCalculationResponse → TaxAmount
    implementation:
      - Map address formats
      - Handle currency conversions
      - Apply business rules
```

## Context Boundaries and Contracts

### API Contracts Between Contexts
```yaml
name: ContextAPICContracts
description: "API contracts for bounded context communication"
version: "1.0"

# Sales Context Public API
sales_api:
  version: "v1"
  endpoints:
    POST /api/v1/orders:
      description: Create new order
      request:
        customer_id: uuid
        items: array[OrderItem]
        shipping_address: Address
      response:
        order_id: uuid
        order_number: string
        status: string

    GET /api/v1/orders/{order_id}:
      description: Get order details
      response:
        order: OrderDetails
        items: array[OrderItemDetails]
        status: string

    POST /api/v1/orders/{order_id}/cancel:
      description: Cancel order
      request:
        reason: string
      response:
        success: boolean
        refund_amount: Money

# Inventory Context API
inventory_api:
  version: "v1"
  endpoints:
    GET /api/v1/inventory/{product_id}:
      description: Get product inventory
      response:
        product_id: uuid
        available: integer
        reserved: integer
        total: integer

    POST /api/v1/inventory/reserve:
      description: Reserve inventory
      request:
        order_id: uuid
        items: array[ReservationItem]
      response:
        reservation_id: uuid
        success: boolean
        expires_at: timestamp

    DELETE /api/v1/inventory/reserve/{reservation_id}:
      description: Release inventory reservation
      response:
        success: boolean
```

### Event-Driven Integration Contracts
```yaml
name: EventDrivenContracts
description: "Event contracts for context integration"
version: "1.0"

# Event schemas
event_schemas:

  OrderCreated:
    source: SalesContext
    description: Order created in sales system
    schema:
      event_id: uuid
      order_id: uuid
      customer_id: uuid
      order_number: string
      total_amount: Money
      currency: string
      items: array[OrderItemEvent]
      shipping_address: Address
      created_at: timestamp

  InventoryReserved:
    source: InventoryContext
    description: Inventory reserved for order
    schema:
      event_id: uuid
      reservation_id: uuid
      order_id: uuid
      success: boolean
      items: array[ReservationItemEvent]
      expires_at: timestamp
      reserved_at: timestamp

  PaymentProcessed:
    source: PaymentContext
    description: Payment processed for order
    schema:
      event_id: uuid
      payment_id: uuid
      order_id: uuid
      amount: Money
      status: PaymentStatus
      transaction_id: string
      processed_at: timestamp

# Event routing
event_routing:
  sales.events:
    OrderCreated → inventory.inventory.reservation
    OrderCancelled → inventory.inventory.release

  inventory.events:
    InventoryReserved → sales.order.reservation_completed
    InventoryUnavailable → sales.order.inventory_failed

  payment.events:
    PaymentProcessed → sales.order.payment_completed
    PaymentFailed → sales.order.payment_failed
```

## Testing Across Boundaries

### Integration Testing Strategy
```yaml
name: CrossBoundaryTesting
description: "Testing strategies for bounded context integration"
version: "1.0"

testing_approaches:

  # Contract Testing
  contract_testing:
    description: Test API contracts between contexts
    tools:
      - Pact for consumer-driven contracts
      - Postman for API testing
    coverage:
      - Request/response schemas
      - Error handling
      - Rate limiting
      - Authentication

  # Event Testing
  event_testing:
    description: Test event contracts and processing
    tools:
      - TestContainers for message brokers
      - Event schema validation
    coverage:
      - Event schema validation
      - Event processing
      - Error scenarios
      - Idempotency

  # End-to-End Testing
  e2e_testing:
    description: Test complete workflows across contexts
    scenarios:
      - Order creation to fulfillment
      - Customer registration with first order
      - Inventory reservation and release
      - Payment processing and refund
    tools:
      - Docker Compose for environment
      - Test data factories
```

## Evolution Strategies

### Context Evolution Patterns
```yaml
name: ContextEvolution
description: "Strategies for evolving bounded contexts"
version: "1.0"

evolution_patterns:

  # Context Splitting
  context_splitting:
    trigger: Domain becomes too complex
    strategy:
      - Identify subdomains
      - Define new boundaries
      - Create migration plan
      - Implement anti-corruption layers
    example:
      - Split large Sales context into Order and Customer contexts

  # Context Merging
  context_merging:
    trigger: Overlapping responsibilities
    strategy:
      - Analyze shared models
      - Define unified ubiquitous language
      - Plan migration
      - Consolidate APIs
    example:
      - Merge overlapping Shipping and Logistics contexts

  # Context Retirement
  context_retirement:
    trigger: Domain becomes obsolete
    strategy:
      - Migrate data
      - Redirect APIs
      - Decommission services
      - Archive documentation
```