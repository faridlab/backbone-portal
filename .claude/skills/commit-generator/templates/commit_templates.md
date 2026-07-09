# Backbone Framework Commit Message Templates
# Use these templates for consistent commit messages

## Feature Templates
feat({module}): Add {feature_name}
feat({module}): Implement {functionality} with {key_aspect}
feat({module}): Add {entity_type} with {capabilities}

# Examples:
feat(sapiens): Add user profile with preferences and avatar
feat(postman): Implement email template rendering engine
feat(bucket): Add file versioning and sharing
feat(backbone): Add generic caching layer

## Bug Fix Templates
fix({module}): Resolve {issue_description}
fix({module}): Fix {component} {problem_type}
fix({module}): Correct {behavior} for {scenario}

# Examples:
fix(auth): Resolve login validation for edge case emails
fix(database): Fix migration rollback on schema changes
fix(api): Correct response status codes for authentication errors
fix(email): Fix SMTP timeout for large attachments

## Refactoring Templates
refactor({module}): Extract {component} to {target_location}
refactor({module}): Simplify {process} logic
refactor({module}): Consolidate {related_components}

# Examples:
refactor(user): Extract validation to dedicated service layer
refactor(payment): Simplify transaction handling logic
refactor(tests): Consolidate user test utilities

## Performance Templates
perf({module}): Optimize {operation} with {optimization}
perf({module}): Improve {metric} performance
perf({module}): Add {caching_strategy} for {data_type}

# Examples:
perf(database): Optimize user queries with composite indexes
perf(email): Batch email sending for better throughput
perf(file): Optimize image thumbnail generation

## Documentation Templates
docs({section}): Add {document_type} for {topic}
docs({section}): Update {documentation} with {new_content}
docs({section}): Document {feature_or_process}

# Examples:
docs(readme): Update setup instructions for new developers
docs(api): Document authentication endpoints and examples
docs(schema): Document new user entity validation rules
docs(migration): Add guide for user entity migration

## Testing Templates
test({module}): Add {test_type} for {feature}
test({module}): Cover {edge_cases} with {test_approach}
test({module}): Add integration tests for {workflow}

# Examples:
test(auth): Add comprehensive MFA authentication tests
test(api): Add integration tests for payment workflow
test(user): Cover validation edge cases with unit tests

## Maintenance Templates
chore({area}): Update {dependency_or_tool} to {version}
chore({area}): Add {configuration} for {feature}
chore({area}): Improve {process} with {enhancement}

# Examples:
chore(deps): Update actix-web to version 4.5
chore(build): Improve Docker build speed with layer caching
chore(ci): Add automated testing for pull requests

## Schema-Specific Templates
feat({module}): Add {entity_type} entity with {key_features}
feat({module}): Add {relationship_type} between {entities}
fix({module}): Resolve {entity} validation error
refactor({module}): Simplify {entity} model structure

# Examples:
feat(sapiens): Add UserProfile entity with preferences and avatar
feat(bucket): Add many-to-many relationship between files and folders
fix(user): Resolve User entity email validation constraint

## Breaking Change Templates
feat!({module}): {major_change_description}
feat!({module}): Redesign {system} with {new_approach}

# Examples:
feat!(auth): Redesign authentication system with JWT
feat!(user): Restructure user entity with new validation system

## Multi-Module Change Templates
feat: Add {feature} across {modules}
fix: Resolve {issue} in {affected_modules}
refactor: Standardize {pattern} across modules

# Examples:
feat: Add unified error handling across all modules
fix: Resolve database transaction handling in sapiens and bucket
refactor: Standardize repository patterns across all modules

## Issue Reference Patterns
{commit_message}

Fixes #{issue_number}
Closes #{issue_number}
Resolves #{issue_number}

# Examples:
feat(sapiens): Add multi-factor authentication

Fixes #123