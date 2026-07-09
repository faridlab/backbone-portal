# Custom Service Test Template

## Usage Instructions
1. Copy this template to `src/domain/services/tests/{service_name}_service_test.rs`
2. Replace placeholders with your specific service logic
3. Add to `src/domain/services/tests/mod.rs`

---

```rust
//! Tests for {EntityName} Domain Service
//!
//! Comprehensive test suite covering:
//! - Business logic validation
//! - Edge cases and error conditions
//! - Integration with repositories
//! - Performance characteristics
//! - Security and permissions

use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use mockall::predicate::*;
use mockall::Mock;

use anyhow::Result;

// Domain imports
use crate::domain::entities::{{EntityName}};
use crate::domain::repositories::mocks::Mock{EntityName}Repository;
use crate::domain::services::{{
    {EntityName}Service,
    {EntityName}DomainService,
}};
use crate::domain::value_objects::{related_value_objects};
use crate::domain::events::{related_events};

// Test utilities
use crate::tests::factories::{EntityNameFactory, TestContext};

// =============================================================================
// TEST FIXTURES AND UTILITIES
// =============================================================================

/// Test fixture for {EntityName} service tests
struct {EntityName}ServiceTestFixture {
    pub mock_repo: Mock{EntityName}Repository,
    pub service: Arc<dyn {EntityName}Service>,
    pub test_context: TestContext,
}

impl {EntityName}ServiceTestFixture {
    fn new() -> Self {
        let mock_repo = Mock{EntityName}Repository::new();
        let service = Arc::new({EntityName}DomainService::new(
            Arc::new(mock_repo.clone()),
        ));

        Self {
            mock_repo,
            service,
            test_context: TestContext::new(),
        }
    }

    fn with_dependency(mut self, dependency: Arc<{OtherServiceType}>) -> Self {
        // If service has optional dependencies, set them up here
        // service = service.with_dependency(dependency);
        self
    }

    async fn setup_mock_{entity_name}(
        &mut self,
        entity: {EntityName},
    ) -> Result<()> {
        self.mock_repo
            .expect_find_by_id()
            .with(eq(entity.id.to_string()))
            .times(1)
            .returning(move |_| Ok(Some(entity.clone())));

        self.mock_repo
            .expect_update()
            .withf(move |id, updated_entity| {
                id == &entity.id.to_string() &&
                updated_entity.id == entity.id
            })
            .times(1)
            .returning(move |_, entity| Ok(Some(entity.clone())));

        Ok(())
    }
}

// =============================================================================
// UNIT TESTS
// =============================================================================

#[tokio::test]
async fn test_{operation_method_name}_success() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();
    let test_entity = EntityNameFactory::create_{entity_name}();

    fixture.setup_mock_{entity_name}(test_entity.clone()).await?;

    let request = {BusinessOperation}Request {
        {field_1}: "{test_value_1}".to_string(),
        {field_2}: {test_value_2},
        business_context: serde_json::json!({
            "user_id": fixture.test_context.user_id,
            "timestamp": Utc::now(),
        }),
    };

    // Act
    let result = fixture.service.{operation_method_name}(request).await?;

    // Assert
    assert!(result.success, "Operation should succeed");
    assert_eq!(result.entity_id, Some(test_entity.id));
    assert!(result.error_message.is_none(), "Should have no errors");
    assert!(!result.business_data.is_null(), "Should return business data");

    Ok(())
}

#[tokio::test]
async fn test_{operation_method_name}_entity_not_found() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();
    let non_existent_id = Uuid::new_v4();

    fixture.mock_repo
        .expect_find_by_id()
        .with(eq(non_existent_id.to_string()))
        .times(1)
        .returning(|_| Ok(None));

    let request = {BusinessOperation}Request {
        {entity_id_field}: non_existent_id.to_string(),
        {field_1}: "{test_value}".to_string(),
        business_context: serde_json::json!({}),
    };

    // Act
    let result = fixture.service.{operation_method_name}(request).await;

    // Assert
    assert!(result.is_err(), "Should return error for non-existent entity");
    let error = result.unwrap_err();
    assert!(error.to_string().contains("not found"));

    Ok(())
}

#[tokio::test]
async fn test_{operation_method_name}_business_rule_violation() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();
    let test_entity = EntityNameFactory::create_{entity_name}_with_status(
        "{status_that_violates_rules}"
    );

    fixture.setup_mock_{entity_name}(test_entity.clone()).await?;

    let request = {BusinessOperation}Request {
        {entity_id_field}: test_entity.id.to_string(),
        {field_1}: "{value_that_triggers_violation}".to_string(),
        business_context: serde_json::json!({}),
    };

    // Act
    let result = fixture.service.{operation_method_name}(request).await?;

    // Assert
    assert!(!result.success, "Operation should fail due to business rule violation");
    assert_eq!(result.entity_id, Some(test_entity.id));
    assert!(result.error_message.is_some(), "Should have error message");
    assert!(result.warnings.len() > 0, "Should have warnings about rule violation");

    Ok(())
}

#[tokio::test]
async fn test_validate_{validation_rule}_success() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();
    let test_entity = EntityNameFactory::create_{entity_name}_with_valid_properties();

    let validation_context = ValidationContext {
        user_id: Some(fixture.test_context.user_id),
        timestamp: Utc::now(),
        permissions: vec!["{required_permission}".to_string()],
        business_context: serde_json::json!({}),
    };

    // Act
    let result = fixture.service
        .validate_{validation_rule}(&test_entity, &validation_context)
        .await?;

    // Assert
    assert!(result, "Validation should pass for valid entity");

    Ok(())
}

#[tokio::test]
async fn test_validate_{validation_rule}_failure() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();
    let test_entity = EntityNameFactory::create_{entity_name}_with_invalid_properties();

    let validation_context = ValidationContext {
        user_id: Some(fixture.test_context.user_id),
        timestamp: Utc::now(),
        permissions: vec![], // No permissions
        business_context: serde_json::json!({}),
    };

    // Act
    let result = fixture.service
        .validate_{validation_rule}(&test_entity, &validation_context)
        .await?;

    // Assert
    assert!(!result, "Validation should fail for invalid entity");

    Ok(())
}

#[tokio::test]
async fn test_calculate_{calculation_type}_basic() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();
    let test_entity = EntityNameFactory::create_{entity_name}_with_calculation_data();

    let parameters = CalculationParameters {
        calculation_type: "{calculation_type}".to_string(),
        variables: [
            ("param_1".to_string(), 100.0),
            ("param_2".to_string(), 0.5),
        ].into_iter().collect(),
        options: CalculationOptions {
            include_taxes: true,
            apply_discounts: false,
            rounding_precision: 2,
        },
    };

    // Act
    let result = fixture.service
        .calculate_{calculation_type}(&test_entity, &parameters)
        .await?;

    // Assert
    assert!(result.amount > dec!(0.0), "Calculated amount should be positive");
    // Add specific calculation assertions based on your business logic

    Ok(())
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[tokio::test]
async fn test_{operation_method_name}_with_side_effects() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();

    // Mock event emission
    let mock_event_emitter = MockEventEmitter::new();
    // Setup event expectations...

    let service_with_events = {EntityName}DomainService::new(
        Arc::new(fixture.mock_repo.clone()),
    ).with_event_emitter(Arc::new(mock_event_emitter));

    let test_entity = EntityNameFactory::create_{entity_name}();
    fixture.setup_mock_{entity_name}(test_entity.clone()).await?;

    let request = {BusinessOperation}Request {
        {entity_id_field}: test_entity.id.to_string(),
        {field_1}: "{trigger_event_value}".to_string(),
        business_context: serde_json::json!({
            "trigger_events": true
        }),
    };

    // Act
    let result = service_with_events.{operation_method_name}(request).await?;

    // Assert
    assert!(result.success, "Operation should succeed");
    // Verify events were emitted...

    Ok(())
}

#[tokio::test]
async fn test_{operation_method_name}_transaction_rollback() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();

    // Mock repository to fail on update
    fixture.mock_repo
        .expect_update()
        .times(1)
        .returning(|_, _| Err(anyhow::anyhow!("Simulated database error")));

    let test_entity = EntityNameFactory::create_{entity_name}();

    fixture.mock_repo
        .expect_find_by_id()
        .with(eq(test_entity.id.to_string()))
        .times(1)
        .returning(move |_| Ok(Some(test_entity.clone())));

    let request = {BusinessOperation}Request {
        {entity_id_field}: test_entity.id.to_string(),
        {field_1}: "{test_value}".to_string(),
        business_context: serde_json::json!({}),
    };

    // Act
    let result = fixture.service.{operation_method_name}(request).await;

    // Assert
    assert!(result.is_err(), "Operation should fail when database update fails");
    // Verify transaction was rolled back...

    Ok(())
}

// =============================================================================
// PERFORMANCE TESTS
// =============================================================================

#[tokio::test]
async fn test_{operation_method_name}_performance() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();

    // Setup mock for performance testing
    fixture.mock_repo
        .expect_find_by_id()
        .times(1000)
        .returning(move |_| {
            Ok(Some(EntityNameFactory::create_{entity_name}()))
        });

    fixture.mock_repo
        .expect_update()
        .times(1000)
        .returning(|_, entity| Ok(Some(entity)));

    let start_time = std::time::Instant::now();

    // Act
    for i in 0..1000 {
        let request = {BusinessOperation}Request {
            {entity_id_field}: format!("test-{}", i),
            {field_1}: format!("value-{}", i),
            business_context: serde_json::json!({}),
        };

        fixture.service.{operation_method_name}(request).await?;
    }

    let duration = start_time.elapsed();

    // Assert
    assert!(
        duration.as_millis() < 5000, // Should complete within 5 seconds
        "Performance test failed: took {}ms",
        duration.as_millis()
    );

    Ok(())
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[tokio::test]
async fn test_{operation_method_name}_with_empty_request() -> Result<()> {
    // Arrange
    let fixture = {EntityName}ServiceTestFixture::new();

    let request = {BusinessOperation}Request {
        {entity_id_field}: "".to_string(),
        {field_1}: "".to_string(),
        business_context: serde_json::Value::Null,
    };

    // Act
    let result = fixture.service.{operation_method_name}(request).await;

    // Assert
    assert!(result.is_err(), "Should fail with empty request");

    Ok(())
}

#[tokio::test]
async fn test_{operation_method_name}_with_extremely_large_data() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();
    let test_entity = EntityNameFactory::create_{entity_name}();

    fixture.setup_mock_{entity_name}(test_entity).await?;

    let large_data = "x".repeat(1_000_000); // 1MB of data
    let request = {BusinessOperation}Request {
        {entity_id_field}: test_entity.id.to_string(),
        {field_1}: large_data,
        business_context: serde_json::json!({}),
    };

    // Act
    let result = fixture.service.{operation_method_name}(request).await;

    // Assert
    match result {
        Ok(_) => panic!("Should fail with extremely large data"),
        Err(e) => assert!(e.to_string().contains("too large") ||
                          e.to_string().contains("limit")),
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_{operation_method_name}() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();
    let test_entity = EntityNameFactory::create_{entity_name}();

    // Setup mock to handle concurrent requests
    fixture.mock_repo
        .expect_find_by_id()
        .times(10)
        .returning(move |_| Ok(Some(test_entity.clone())));

    fixture.mock_repo
        .expect_update()
        .times(10)
        .returning(|_, entity| Ok(Some(entity)));

    // Act - Execute 10 concurrent operations
    let mut handles = Vec::new();
    for i in 0..10 {
        let service = fixture.service.clone();
        let entity_id = test_entity.id.to_string();

        let handle = tokio::spawn(async move {
            let request = {BusinessOperation}Request {
                {entity_id_field}: entity_id,
                {field_1}: format!("concurrent-{}", i),
                business_context: serde_json::json!({}),
            };

            service.{operation_method_name}(request).await
        });

        handles.push(handle);
    }

    // Collect results
    let results: Vec<Result<_>> = futures::future::join_all(handles)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect();

    // Assert
    let success_count = results.iter().filter(|r| {
        matches!(r, Ok(result) if result.success)
    }).count();

    assert!(success_count > 0, "At least some operations should succeed");

    Ok(())
}

// =============================================================================
// SECURITY TESTS
// =============================================================================

#[tokio::test]
async fn test_{operation_method_name}_authorization() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();
    let test_entity = EntityNameFactory::create_{entity_name}();

    fixture.setup_mock_{entity_name}(test_entity.clone()).await?;

    let unauthorized_request = {BusinessOperation}Request {
        {entity_id_field}: test_entity.id.to_string(),
        {field_1}: "{test_value}".to_string(),
        business_context: serde_json::json!({
            "user_id": Uuid::new_v4(), // Different user
            "permissions": [] // No permissions
        }),
    };

    // Act
    let result = fixture.service.{operation_method_name}(unauthorized_request).await;

    // Assert
    assert!(result.is_err(), "Should fail without proper authorization");

    Ok(())
}

#[tokio::test]
async fn test_{operation_method_name}_input_sanitization() -> Result<()> {
    // Arrange
    let mut fixture = {EntityName}ServiceTestFixture::new();
    let test_entity = EntityNameFactory::create_{entity_name}();

    fixture.setup_mock_{entity_name}(test_entity).await?;

    let malicious_input = "<script>alert('xss')</script>";
    let request = {BusinessOperation}Request {
        {entity_id_field}: test_entity.id.to_string(),
        {field_1}: malicious_input.to_string(),
        business_context: serde_json::json!({}),
    };

    // Act
    let result = fixture.service.{operation_method_name}(request).await?;

    // Assert
    assert!(result.success, "Should handle malicious input gracefully");
    // Verify malicious content was sanitized in the result
    let result_data = result.business_data.to_string();
    assert!(!result_data.contains("<script>"), "Malicious content should be sanitized");

    Ok(())
}
```