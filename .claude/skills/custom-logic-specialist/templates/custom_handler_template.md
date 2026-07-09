# Custom Handler Extension Template

## Usage Instructions
1. Add this code to the `// <<< CUSTOM HANDLERS START >>>` section in generated handler files
2. Replace placeholders with your specific handler logic
3. Ensure required dependencies are imported in the handler file

---

## HTTP Handler Extension Template

```rust
// <<< CUSTOM HANDLERS START >>>

/// Custom HTTP handler for {business_operation}
///
/// Handles complex {business_operation} workflow with {workflow_type} logic.
/// This handler extends the generated CRUD operations with specialized business workflows.
#[axum::debug_handler]
pub async fn {operation_name}_handler(
    State(app_state): State<AppState>,
    Json(request): Json<{OperationName}Request>,
) -> Result<Json<{OperationName}Response>, ApiError> {
    // 1. Validate request
    let validated_request = request.validate()?;

    // 2. Execute custom business logic
    let result = app_state
        .custom_{entity}_service
        .{operation_method}(validated_request)
        .await
        .map_err(ApiError::from_business_error)?;

    // 3. Transform response
    let response = {OperationName}Response::from(result);

    Ok(Json(response))
}

/// Advanced search with complex filters and aggregations
#[axum::debug_handler]
pub async fn advanced_{entity}_search_handler(
    State(app_state): State<AppState>,
    Query(params): Query<AdvancedSearchQuery>,
) -> Result<Json<AdvancedSearchResponse>, ApiError> {
    // Validate search parameters
    let search_params = params.validate()?;

    // Execute advanced search with custom business logic
    let results = app_state
        .custom_{entity}_service
        .advanced_search(search_params)
        .await
        .map_err(ApiError::from_business_error)?;

    Ok(Json(AdvancedSearchResponse::from(results)))
}

/// Batch operations for {entity_name}
#[axum::debug_handler]
pub async fn batch_{entity}_operation_handler(
    State(app_state): State<AppState>,
    Json(request): Json<Batch{EntityName}OperationRequest>,
) -> Result<Json<Batch{EntityName}OperationResponse>, ApiError> {
    // Validate batch size and permissions
    if request.items.is_empty() || request.items.len() > MAX_BATCH_SIZE {
        return Err(ApiError::ValidationError(
            "Batch size must be between 1 and 1000".to_string(),
        ));
    }

    // Execute batch operation with transaction safety
    let results = app_state
        .transaction_service
        .execute_in_transaction(|tx| {
            Box::pin(async move {
                let mut results = Vec::new();

                for item in request.items {
                    let result = app_state
                        .custom_{entity}_service
                        .{operation_method}_with_tx(item, tx)
                        .await?;
                    results.push(result);
                }

                Ok(results)
            })
        })
        .await
        .map_err(ApiError::from_business_error)?;

    Ok(Json(Batch{EntityName}OperationResponse {
        results,
        total_processed: results.len(),
        errors: Vec::new(),
    }))
}

/// {EntityName} analytics and reporting endpoint
#[axum::debug_handler]
pub async fn {entity_name}_analytics_handler(
    State(app_state): State<AppState>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse>, ApiError> {
    // Validate analytics parameters
    let analytics_params = params.validate()?;

    // Check permissions for analytics access
    if !app_state.auth_service.has_permission(&params.user_id, "analytics:view") {
        return Err(ApiError::Forbidden("Insufficient permissions".to_string()));
    }

    // Generate analytics with custom business metrics
    let analytics = app_state
        .custom_{entity}_service
        .generate_analytics(analytics_params)
        .await
        .map_err(ApiError::from_business_error)?;

    Ok(Json(AnalyticsResponse::from(analytics)))
}

// Export handler for route configuration
pub use {
    {operation_name}_handler,
    advanced_{entity}_search_handler,
    batch_{entity}_operation_handler,
    {entity_name}_analytics_handler,
};

// <<< CUSTOM HANDLERS END >>>
```

## gRPC Handler Extension Template

```rust
// <<< CUSTOM GRPC HANDLERS START >>>

/// Custom gRPC service implementation for advanced {entity_name} operations
pub struct Custom{EntityName}GrpcService {
    app_state: Arc<AppState>,
}

impl Custom{EntityName}GrpcService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

#[tonic::async_trait]
impl custom_{entity_name}_grpc_server::Custom{EntityName}Grpc for Custom{EntityName}GrpcService {
    /// Advanced {operation_name} with complex business logic
    async fn {operation_name}(
        &self,
        request: Request<grpc::{OperationName}Request>,
    ) -> Result<Response<grpc::{OperationName}Response>, tonic::Status> {
        let request = request.into_inner();

        // Validate and convert gRPC request to domain request
        let domain_request = {OperationName}Request::try_from(request)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;

        // Execute custom business logic
        let result = self.app_state
            .custom_{entity}_service
            .{operation_method}(domain_request)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        // Convert domain result to gRPC response
        let grpc_response = grpc::{OperationName}Response::from(result);

        Ok(Response::new(grpc_response))
    }

    /// Stream {entity_name} updates for real-time notifications
    async fn stream_{entity_name}_updates(
        &self,
        request: Request<grpc::{EntityName}StreamRequest>,
    ) -> Result<Response<tonic::codec::Streaming<grpc::{EntityName}Update>>, tonic::Status> {
        let request = request.into_inner();

        // Create update stream with custom filtering
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // Subscribe to {entity_name} updates
        let update_stream = self.app_state
            .event_stream
            .subscribe_to_{entity_name}_updates(request.filter_criteria)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        // Spawn task to convert domain events to gRPC messages
        tokio::spawn(async move {
            let mut stream = update_stream;
            while let Some(update) = stream.next().await {
                let grpc_update = grpc::{EntityName}Update::from(update);
                if tx.send(grpc_update).await.is_err() {
                    break; // Client disconnected
                }
            }
        });

        let grpc_stream = tonic::codec::Streaming::new(rx);
        Ok(Response::new(grpc_stream))
    }

    /// Batch {operation_name} with transaction guarantees
    async fn batch_{operation_name}(
        &self,
        request: Request<grpc::Batch{OperationName}Request>,
    ) -> Result<Response<grpc::Batch{OperationName}Response>, tonic::Status> {
        let request = request.into_inner();

        // Validate batch size
        if request.items.is_empty() || request.items.len() > MAX_BATCH_SIZE {
            return Err(tonic::Status::invalid_argument(
                "Invalid batch size".to_string(),
            ));
        }

        // Execute batch operation
        let results = self.app_state
            .transaction_service
            .execute_in_transaction(|tx| {
                Box::pin(async move {
                    let mut results = Vec::new();

                    for item in request.items {
                        let domain_item = {OperationName}Request::try_from(item)?;
                        let result = self.app_state
                            .custom_{entity}_service
                            .{operation_method}_with_tx(domain_item, tx)
                            .await?;
                        results.push(grpc::{OperationName}Result::from(result));
                    }

                    Ok(results)
                })
            })
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(Response::new(grpc::Batch{OperationName}Response {
            results,
            total_count: results.len() as u32,
            errors: Vec::new(),
        }))
    }
}

// <<< CUSTOM GRPC HANDLERS END >>>
```

## Request/Response Types Template

```rust
// <<< CUSTOM TYPES START >>>

/// Custom request for {operation_name} operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {OperationName}Request {
    pub {entity_id_field}: String,
    pub operation_data: serde_json::Value,
    pub business_context: Option<BusinessContext>,
    pub validation_options: Option<ValidationOptions>,
}

impl {OperationName}Request {
    /// Validate request and convert to domain request
    pub fn validate(self) -> Result<Domain{OperationName}Request, ValidationError> {
        // Validate required fields
        if self.{entity_id_field}.is_empty() {
            return Err(ValidationError::MissingField("entity_id".to_string()));
        }

        // Validate business context if provided
        if let Some(ctx) = &self.business_context {
            ctx.validate()?;
        }

        Ok(Domain{OperationName}Request {
            entity_id: self.{entity_id_field},
            operation_data: self.operation_data,
            business_context: self.business_context,
            validation_options: self.validation_options,
        })
    }
}

/// Custom response for {operation_name} operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {OperationName}Response {
    pub success: bool,
    pub entity_data: Option<serde_json::Value>,
    pub business_result: Option<BusinessResult>,
    pub warnings: Vec<String>,
    pub metadata: ResponseMetadata,
}

impl From<Domain{OperationName}Result> for {OperationName}Response {
    fn from(domain_result: Domain{OperationName}Result) -> Self {
        Self {
            success: domain_result.success,
            entity_data: domain_result.entity_data,
            business_result: domain_result.business_result.map(Into::into),
            warnings: domain_result.warnings,
            metadata: ResponseMetadata {
                timestamp: chrono::Utc::now(),
                operation_id: domain_result.operation_id,
                processing_time_ms: domain_result.processing_time_ms,
            },
        }
    }
}

/// Advanced search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSearchQuery {
    #[serde(flatten)]
    pub base_query: BaseSearchQuery,
    pub custom_filters: Vec<CustomFilter>,
    pub aggregations: Vec<AggregationRequest>,
    pub sort_options: Vec<SortOption>,
    pub pagination: PaginationOptions,
    pub include_metadata: bool,
}

/// Batch operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch{EntityName}OperationRequest {
    pub items: Vec<{OperationName}Request>,
    pub operation_type: BatchOperationType,
    pub error_handling: ErrorHandlingStrategy,
    pub transaction_mode: TransactionMode,
}

/// Batch operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch{EntityName}OperationResponse {
    pub results: Vec<Batch{OperationName}Result>,
    pub total_processed: usize,
    pub errors: Vec<BatchOperationError>,
    pub transaction_id: Option<String>,
}

/// Analytics query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub metrics: Vec<String>,
    pub dimensions: Vec<String>,
    pub filters: Vec<AnalyticsFilter>,
    pub time_range: TimeRange,
    pub group_by: Option<String>,
    pub user_id: String,
}

// <<< CUSTOM TYPES END >>>
```

## Route Registration Template

```rust
// Add to your router configuration module

// <<< CUSTOM ROUTES START >>>

/// Register custom {entity_name} routes
pub fn register_custom_{entity_name}_routes(
    router: Router<Arc<AppState>>,
) -> Router<Arc<AppState>> {
    router
        // Custom operation routes
        .route(
            "/api/v1/{entity_name}/{operation_name}",
            post({operation_name}_handler),
        )
        .route(
            "/api/v1/{entity_name}/advanced-search",
            get(advanced_{entity}_search_handler),
        )
        .route(
            "/api/v1/{entity_name}/batch-operation",
            post(batch_{entity}_operation_handler),
        )
        .route(
            "/api/v1/{entity_name}/analytics",
            get({entity_name}_analytics_handler),
        )
        // Additional custom routes...
        .route(
            "/api/v1/{entity_name}/export",
            get(export_{entity_name}_handler),
        )
        .route(
            "/api/v1/{entity_name}/import",
            post(import_{entity_name}_handler),
        )
        .route(
            "/api/v1/{entity_name}/validate",
            post(validate_{entity_name}_handler),
        )
}

// <<< CUSTOM ROUTES END >>>
```

## Error Handling Template

```rust
// <<< CUSTOM ERROR HANDLING START >>>

/// Custom API errors for {entity_name} operations
#[derive(Debug, thiserror::Error)]
pub enum Custom{EntityName}Error {
    #[error("Business validation failed: {details}")]
    BusinessValidationError { details: String },

    #[error("Operation not permitted: {reason}")]
    OperationNotPermitted { reason: String },

    #[error("Concurrent modification detected")]
    ConcurrentModification,

    #[error("Batch operation limit exceeded: {current}/{max}")]
    BatchSizeExceeded { current: usize, max: usize },

    #[error("Analytics processing failed: {reason}")]
    AnalyticsError { reason: String },

    #[error("Export failed: {reason}")]
    ExportError { reason: String },
}

impl From<Custom{EntityName}Error> for ApiError {
    fn from(err: Custom{EntityName}Error) -> Self {
        match err {
            Custom{EntityName}Error::BusinessValidationError { details } => {
                ApiError::ValidationError(details)
            }
            Custom{EntityName}Error::OperationNotPermitted { reason } => {
                ApiError::Forbidden(reason)
            }
            Custom{EntityName}Error::ConcurrentModification => {
                ApiError::Conflict("Concurrent modification detected".to_string())
            }
            Custom{EntityName}Error::BatchSizeExceeded { current, max } => {
                ApiError::ValidationError(format!(
                    "Batch size {} exceeds maximum of {}",
                    current, max
                ))
            }
            Custom{EntityName}Error::AnalyticsError { reason } => {
                ApiError::InternalServerError(format!("Analytics error: {}", reason))
            }
            Custom{EntityName}Error::ExportError { reason } => {
                ApiError::InternalServerError(format!("Export error: {}", reason))
            }
        }
    }
}

// <<< CUSTOM ERROR HANDLING END >>>
```