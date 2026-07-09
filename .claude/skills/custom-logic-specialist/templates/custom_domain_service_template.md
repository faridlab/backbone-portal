# Custom Domain Service Template

## Usage Instructions
1. Copy this template to `src/domain/services/{service_name}_service.rs`
2. Replace placeholders with your specific domain logic
3. Add to `src/domain/mod.rs` and `src/domain/services/mod.rs`
4. Create tests in `src/domain/services/tests/{service_name}_service_test.rs`

---

```rust
//! {EntityName} Domain Service
//!
//! Custom business logic for {domain_description}.
//! This service contains complex business rules that don't fit in generated services.
//!
//! Responsibilities:
//! - {business_responsibility_1}
//! - {business_responsibility_2}
//! - {business_responsibility_3}

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Domain imports
use crate::domain::entities::{{related_entities}}};
use crate::domain::repositories::{{related_repositories}};
use crate::domain::value_objects::{related_value_objects};
use crate::domain::events::{related_events};

// Custom imports
// use crate::domain::services::other_custom_service::OtherCustomService;

/// Custom business result for {entity_name} operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {EntityName}BusinessResult {
    pub success: bool,
    pub entity_id: Option<Uuid>,
    pub business_data: serde_json::Value,
    pub warnings: Vec<String>,
    pub error_message: Option<String>,
}

/// Request DTO for {business_operation}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {BusinessOperation}Request {
    pub {field_1}: {type_1},
    pub {field_2}: {type_2},
    pub business_context: serde_json::Value,
}

/// {EntityName} Domain Service trait
#[async_trait]
pub trait {EntityName}Service: Send + Sync {
    /// Perform {business_operation} with complex business rules
    async fn {operation_method_name}(
        &self,
        request: {BusinessOperation}Request,
    ) -> Result<{EntityName}BusinessResult>;

    /// Validate {business_rule}
    async fn validate_{validation_rule}(
        &self,
        entity: &{EntityName},
        context: &ValidationContext,
    ) -> Result<bool>;

    /// Calculate {calculation_type} based on business rules
    async fn calculate_{calculation_type}(
        &self,
        entity: &{EntityName},
        parameters: &CalculationParameters,
    ) -> Result<{ResultType}>;
}

/// {EntityName} Domain Service implementation
pub struct {EntityName}DomainService {
    {entity_name}_repo: Arc<dyn {EntityName}Repository>,
    {other_dependency}_service: Option<Arc<{OtherServiceType}>>,
}

impl {EntityName}DomainService {
    /// Create new service instance
    pub fn new(
        {entity_name}_repo: Arc<dyn {EntityName}Repository>,
    ) -> Self {
        Self {
            {entity_name}_repo,
            {other_dependency}_service: None,
        }
    }

    /// Builder pattern for optional dependencies
    pub fn with_{other_dependency}_service(
        mut self,
        service: Arc<{OtherServiceType}>,
    ) -> Self {
        self.{other_dependency}_service = Some(service);
        self
    }

    // =========================================================================
    // PRIVATE HELPER METHODS
    // =========================================================================

    /// Check if {business_condition} is met
    async fn check_{business_condition}(
        &self,
        entity: &{EntityName},
    ) -> Result<bool> {
        // Implement complex business rule
        // Example: Check eligibility, permissions, business constraints
        todo!("Implement business condition check")
    }

    /// Apply {business_rule} to entity
    async fn apply_{business_rule}(
        &self,
        entity: &mut {EntityName},
        parameters: &BusinessRuleParameters,
    ) -> Result<()> {
        // Implement business rule application
        // Example: Apply discounts, calculate fees, update status
        todo!("Implement business rule application")
    }

    /// Emit {business_event} after operation
    async fn emit_{event_name}(
        &self,
        event_data: {EventDataType},
    ) -> Result<()> {
        // Emit domain event for other services to react to
        // Integration with event system
        todo!("Implement event emission")
    }
}

#[async_trait]
impl {EntityName}Service for {EntityName}DomainService {
    async fn {operation_method_name}(
        &self,
        request: {BusinessOperation}Request,
    ) -> Result<{EntityName}BusinessResult> {
        // 1. Validate request
        self.validate_request(&request)?;

        // 2. Load required data
        let entity = self.{entity_name}_repo
            .find_by_id(&request.{entity_id_field})
            .await?
            .ok_or_else(|| anyhow::anyhow!("{EntityName} not found"))?;

        // 3. Check business rules
        if !self.check_{business_condition}(&entity).await? {
            return Ok({EntityName}BusinessResult {
                success: false,
                entity_id: Some(entity.id),
                business_data: serde_json::Value::Null,
                warnings: vec!["Business condition not met".to_string()],
                error_message: Some("Operation not allowed by business rules".to_string()),
            });
        }

        // 4. Execute business logic
        let mut updated_entity = entity.clone();
        self.apply_{business_rule}(&mut updated_entity, &request.business_context.into())
            .await?;

        // 5. Save changes
        let saved_entity = self.{entity_name}_repo
            .update(&updated_entity.id, &updated_entity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to save {entity_name}"))?;

        // 6. Emit events
        self.emit_{event_name}({EventDataType}::from(&saved_entity))
            .await?;

        // 7. Return result
        Ok({EntityName}BusinessResult {
            success: true,
            entity_id: Some(saved_entity.id),
            business_data: serde_json::to_value(&saved_entity)?,
            warnings: Vec::new(),
            error_message: None,
        })
    }

    async fn validate_{validation_rule}(
        &self,
        entity: &{EntityName},
        context: &ValidationContext,
    ) -> Result<bool> {
        // Implement complex validation logic
        // Consider multiple factors, external services, business constraints
        todo!("Implement validation rule")
    }

    async fn calculate_{calculation_type}(
        &self,
        entity: &{EntityName},
        parameters: &CalculationParameters,
    ) -> Result<{ResultType}> {
        // Implement complex calculation
        // May involve multiple entities, external data, complex formulas
        todo!("Implement calculation")
    }
}

// =========================================================================
// SUPPORTING TYPES
// =========================================================================

/// Validation context for business rules
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub user_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub permissions: Vec<String>,
    pub business_context: serde_json::Value,
}

/// Parameters for business rule application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRuleParameters {
    pub rule_type: String,
    pub parameters: serde_json::Value,
    pub override_flags: HashMap<String, bool>,
}

/// Parameters for calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationParameters {
    pub calculation_type: String,
    pub variables: HashMap<String, f64>,
    pub options: CalculationOptions,
}

/// Options for calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationOptions {
    pub include_taxes: bool,
    pub apply_discounts: bool,
    pub rounding_precision: u8,
}

/// Custom validation error types
#[derive(Debug, thiserror::Error)]
pub enum {EntityName}BusinessError {
    #[error("Validation failed: {message}")]
    ValidationError { message: String },

    #[error("Business rule violation: {rule}")]
    BusinessRuleViolation { rule: String },

    #[error("Calculation error: {details}")]
    CalculationError { details: String },

    #[error("Entity not found: {id}")]
    EntityNotFound { id: Uuid },

    #[error("Insufficient permissions: {required}")]
    InsufficientPermissions { required: Vec<String> },
}
```

## Registration in Module Files

### 1. Add to `src/domain/services/mod.rs`
```rust
pub mod {service_name}_service;
pub use {service_name}_service::*;
```

### 2. Add to `src/domain/mod.rs` (if exported)
```rust
pub mod services;
```

## Testing Template

See `templates/custom_service_test_template.md` for testing patterns.