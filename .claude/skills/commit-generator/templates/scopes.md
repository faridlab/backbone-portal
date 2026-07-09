# Backbone Framework Commit Scopes

## Module Scopes
sapiens      - User management and authentication
postman      - Email and notifications
bucket     - File storage and management
backbone     - Core framework and shared components

## Component Scopes
auth         - Authentication and authorization
user         - User management features
email        - Email sending and templates
file         - File operations and storage
payment      - Payment processing
notification - System notifications
schema       - Schema definitions and generation
cli          - Command-line interface tools
api          - REST API endpoints
grpc         - gRPC services
database     - Database operations and migrations
cache        - Caching layer and strategies
security     - Security features and configurations

## Infrastructure Scopes
docker       - Docker configuration and images
ci           - Continuous integration pipelines
deployment   - Deployment configurations and scripts
monitoring   - Monitoring and observability
config       - Configuration management
deps         - Dependencies and package management
build        - Build system and tooling
test         - Testing infrastructure and utilities

## Documentation Scopes
readme       - README and getting started guides
api          - API documentation
schema       - Schema documentation
migration    - Migration guides and instructions
examples     - Code examples and tutorials

# Scope Selection Guide:
# 1. Use module scope when changing module-specific features
# 2. Use component scope when changing cross-cutting concerns
# 3. Use infrastructure scope when changing deployment/ops
# 4. Use framework scope for core Backbone changes
# 5. Be specific but not overly granular