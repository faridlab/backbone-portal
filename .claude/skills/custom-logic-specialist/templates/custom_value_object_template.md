# Custom Value Object Template

## Usage Instructions
1. Copy this template to `src/domain/value_objects/{value_object_name}.rs`
2. Replace placeholders with your specific value object logic
3. Add to `src/domain/value_objects/mod.rs`
4. Create tests in `src/domain/value_objects/tests/{value_object_name}_test.rs`

---

```rust
//! {ValueObjectName} Value Object
//!
//! Represents {domain_concept} with built-in validation and business logic.
//! This value object ensures type safety and business rule enforcement
//! throughout the domain layer.
//!
//! Business Rules:
//! - {business_rule_1}
//! - {business_rule_2}
//! - {business_rule_3}

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use anyhow::{Result, anyhow};
use regex::Regex;

/// {ValueObjectName} value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct {ValueObjectName}(
    #[serde(skip)]
    pub(crate) String
);

impl {ValueObjectName} {
    /// Create a new validated {value_object_name}
    ///
    /// # Arguments
    /// * `value` - The raw string value to validate and wrap
    ///
    /// # Returns
    /// * `Ok({ValueObjectName})` if validation passes
    /// * `Err(anyhow::Error)` if validation fails
    ///
    /// # Examples
    /// ```
    /// use your_module::domain::value_objects::{ValueObjectName};
    ///
    /// let valid = {ValueObjectName}::new("valid_value")?;
    /// assert_eq!(valid.to_string(), "valid_value");
    ///
    /// let invalid = {ValueObjectName}::new("invalid_value");
    /// assert!(invalid.is_err());
    /// ```
    pub fn new(value: &str) -> Result<Self> {
        // 1. Basic format validation
        if !Self::is_valid_format(value) {
            return Err(anyhow!("Invalid {value_object_name} format: {}", value));
        }

        // 2. Business rule validation
        Self::validate_business_rules(value)?;

        // 3. Transform/clean the value (optional)
        let cleaned_value = Self::clean_value(value);

        Ok(Self(cleaned_value))
    }

    /// Create {value_object_name} without validation (internal use only)
    ///
    /// # Safety
    /// This method bypasses validation and should only be used when
    /// the value is already known to be valid (e.g., from database).
    pub(crate) fn unsafe_new(value: String) -> Self {
        Self(value)
    }

    /// Get the raw string value
    pub fn value(&self) -> &str {
        &self.0
    }

    /// Get a reference to the inner value
    pub fn inner(&self) -> &String {
        &self.0
    }

    /// Consume and return the inner value
    pub fn into_inner(self) -> String {
        self.0
    }

    // =========================================================================
    // BUSINESS LOGIC METHODS
    // =========================================================================

    /// Check if {value_object_name} meets {business_condition}
    pub fn meets_{business_condition}(&self) -> bool {
        // Implement business-specific logic
        // Example: Check if email is corporate, if address is in service area, etc.
        todo!("Implement business condition check")
    }

    /// Extract {component} from {value_object_name}
    pub fn extract_{component}(&self) -> Option<&str> {
        // Extract a component of the value object
        // Example: Extract domain from email, area code from phone, etc.
        todo!("Implement component extraction")
    }

    /// Transform {value_object_name} to {representation}
    pub fn to_{representation}(&self) -> String {
        // Convert to different representation
        // Example: Format for display, normalize, etc.
        todo!("Implement transformation")
    }

    /// Compare with another {value_object_name} using {comparison_type}
    pub fn compare_{comparison_type}(&self, other: &Self) -> ComparisonResult {
        // Custom comparison logic
        // Example: Similarity scoring, distance calculation, etc.
        todo!("Implement comparison")
    }

    // =========================================================================
    // VALIDATION METHODS
    // =========================================================================

    /// Validate basic format using regex or other rules
    fn is_valid_format(value: &str) -> bool {
        // Implement format validation
        // Example: Email regex, phone number format, URL structure, etc.

        lazy_static::lazy_static! {
            static ref VALIDATION_REGEX: Regex = Regex::new(
                r"{validation_regex_pattern}"
            ).unwrap();
        }

        // Additional format checks
        if value.len() < {min_length} || value.len() > {max_length} {
            return false;
        }

        VALIDATION_REGEX.is_match(value)
    }

    /// Validate business-specific rules
    fn validate_business_rules(value: &str) -> Result<()> {
        // 1. {business_rule_1}
        if Self::violates_rule_1(value) {
            return Err(anyhow!("Violates business rule 1: {rule_1_description}"));
        }

        // 2. {business_rule_2}
        if Self::violates_rule_2(value) {
            return Err(anyhow!("Violates business rule 2: {rule_2_description}"));
        }

        // 3. {business_rule_3}
        if Self::violates_rule_3(value) {
            return Err(anyhow!("Violates business rule 3: {rule_3_description}"));
        }

        Ok(())
    }

    /// Clean and normalize the value
    fn clean_value(value: &str) -> String {
        // Apply cleaning/normalization rules
        // Example: Trim whitespace, convert to lowercase, remove formatting, etc.
        value.trim().to_lowercase()
    }

    // =========================================================================
    // PRIVATE BUSINESS RULE CHECKS
    // =========================================================================

    fn violates_rule_1(value: &str) -> bool {
        // Implement rule 1 check
        todo!("Implement rule 1 validation")
    }

    fn violates_rule_2(value: &str) -> bool {
        // Implement rule 2 check
        todo!("Implement rule 2 validation")
    }

    fn violates_rule_3(value: &str) -> bool {
        // Implement rule 3 check
        todo!("Implement rule 3 validation")
    }
}

// =========================================================================
// TRAIT IMPLEMENTATIONS
// =========================================================================

impl fmt::Display for {ValueObjectName} {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for {ValueObjectName} {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

impl From<{ValueObjectName}> for String {
    fn from(value: {ValueObjectName}) -> Self {
        value.0
    }
}

impl TryFrom<String> for {ValueObjectName} {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        Self::new(&value)
    }
}

impl<'a> TryFrom<&'a str> for {ValueObjectName} {
    type Error = anyhow::Error;

    fn try_from(value: &'a str) -> Result<Self> {
        Self::new(value)
    }
}

// =========================================================================
// SUPPORTING TYPES
// =========================================================================

/// Result of comparison operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonResult {
    Equal,
    Similar(f32),  // With similarity score
    Different,
    Related,       // Related but different (e.g., same domain)
}

/// Builder for creating {value_object_name} with options
#[derive(Debug, Clone)]
pub struct {ValueObjectName}Builder {
    value: Option<String>,
    strict_validation: bool,
    allow_transformation: bool,
}

impl {ValueObjectName}Builder {
    pub fn new() -> Self {
        Self {
            value: None,
            strict_validation: true,
            allow_transformation: false,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn strict_validation(mut self, strict: bool) -> Self {
        self.strict_validation = strict;
        self
    }

    pub fn allow_transformation(mut self, allow: bool) -> Self {
        self.allow_transformation = allow;
        self
    }

    pub fn build(self) -> Result<{ValueObjectName}> {
        let value = self.value.ok_or_else(|| anyhow!("Value is required"))?;

        if self.allow_transformation {
            // Apply transformations before validation
            let transformed = Self::apply_transformations(&value)?;
            {ValueObjectName}::new(&transformed)
        } else {
            {ValueObjectName}::new(&value)
        }
    }

    fn apply_transformations(value: &str) -> Result<String> {
        // Apply business-approved transformations
        // Example: Add missing components, normalize format, etc.
        todo!("Implement transformations")
    }
}

impl Default for {ValueObjectName}Builder {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// CONSTANTS AND HELPER FUNCTIONS
// =========================================================================

/// Maximum allowed length for {value_object_name}
pub const MAX_LENGTH: usize = {max_length};

/// Minimum allowed length for {value_object_name}
pub const MIN_LENGTH: usize = {min_length};

/// List of blocked values/patterns
pub const BLOCKED_PATTERNS: &[&str] = &[
    "{pattern_1}",
    "{pattern_2}",
    "{pattern_3}",
];

/// Check if a value is in the blocked list
pub fn is_blocked_value(value: &str) -> bool {
    BLOCKED_PATTERNS.iter().any(|&pattern| {
        value.to_lowercase().contains(&pattern.to_lowercase())
    })
}

/// Extract {useful_component} from {value_object_name} string
pub fn extract_{component}_from_string(value: &str) -> Option<String> {
    // Extract component from raw string
    // This can be used without creating the full value object
    todo!("Implement component extraction")
}
```

## Registration in Module Files

### Add to `src/domain/value_objects/mod.rs`
```rust
pub mod {value_object_name};
pub use {value_object_name}::*;
```

## Testing Template

See `templates/custom_value_object_test_template.md` for testing patterns.

## Usage Examples

### Basic Usage
```rust
use your_module::domain::value_objects::{ValueObjectName};

// Create with validation
let email = {ValueObjectName}::new("user@example.com")?;
assert_eq!(email.value(), "user@example.com");

// Parse from string
let email: {ValueObjectName} = "user@example.com".parse()?;

// Use builder pattern
let email = {ValueObjectName}Builder::new()
    .value("USER@EXAMPLE.COM")
    .allow_transformation(true)
    .build()?;
```

### Integration with Entities
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    #[serde(with = "serde_with::As::<serde_with::FromInto<{ValueObjectName}>>")]
    pub email: {ValueObjectName},  // Automatic conversion
    pub created_at: DateTime<Utc>,
}
```