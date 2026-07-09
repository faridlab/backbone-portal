# Real-World Custom Logic Examples

This document provides practical, real-world examples of custom business logic implementation within the Backbone Framework. Each example demonstrates safe extension patterns and best practices.

## Example 1: E-Commerce Pricing Service

### Scenario
An e-commerce platform needs complex pricing logic with:
- Volume discounts
- Customer tier pricing
- Seasonal promotions
- Tax calculations by location

### Implementation

#### Custom Domain Service
```rust
// src/domain/services/pricing_service.rs
use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::domain::entities::{Product, Order, Customer};
use crate::domain::repositories::{ProductRepository, CustomerRepository};
use crate::domain::value_objects::{Money, Currency, CustomerTier};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingCalculation {
    pub base_price: Money,
    pub volume_discount: Money,
    pub customer_discount: Money,
    pub seasonal_discount: Money,
    pub tax_amount: Money,
    pub final_price: Money,
    pub applied_rules: Vec<String>,
}

pub struct PricingService {
    product_repo: Arc<dyn ProductRepository>,
    customer_repo: Arc<dyn CustomerRepository>,
}

impl PricingService {
    pub async fn calculate_order_total(
        &self,
        order: &Order,
        customer_id: &str,
        shipping_address: &Address,
    ) -> Result<PricingCalculation> {
        let customer = self.customer_repo.find_by_id(customer_id).await?
            .ok_or_else(|| anyhow::anyhow!("Customer not found"))?;

        let mut total_base = Money::zero();
        let mut total_volume_discount = Money::zero();
        let mut total_customer_discount = Money::zero();
        let mut total_seasonal_discount = Money::zero();

        // Calculate line items
        for item in &order.items {
            let product = self.product_repo.find_by_id(&item.product_id).await?
                .ok_or_else(|| anyhow::anyhow!("Product not found"))?;

            let line_calculation = self.calculate_line_item_price(
                &product,
                item.quantity,
                &customer,
            ).await?;

            total_base = total_base.add(&line_calculation.base_price);
            total_volume_discount = total_volume_discount.add(&line_calculation.volume_discount);
            total_customer_discount = total_customer_discount.add(&line_calculation.customer_discount);
            total_seasonal_discount = total_seasonal_discount.add(&line_calculation.seasonal_discount);
        }

        // Calculate tax based on shipping address
        let tax_rate = self.get_tax_rate(shipping_address)?;
        let subtotal_after_discounts = total_base
            .subtract(&total_volume_discount)
            .subtract(&total_customer_discount)
            .subtract(&total_seasonal_discount);

        let tax_amount = subtotal_after_discounts.multiply(tax_rate);
        let final_price = subtotal_after_discounts.add(&tax_amount);

        Ok(PricingCalculation {
            base_price: total_base,
            volume_discount: total_volume_discount,
            customer_discount: total_customer_discount,
            seasonal_discount: total_seasonal_discount,
            tax_amount,
            final_price,
            applied_rules: vec![
                "Volume discount applied".to_string(),
                format!("Customer tier: {:?}", customer.tier),
                "Seasonal promotion checked".to_string(),
                format!("Tax rate: {:.2}%", tax_rate * 100.0),
            ],
        })
    }

    async fn calculate_line_item_price(
        &self,
        product: &Product,
        quantity: u32,
        customer: &Customer,
    ) -> Result<PricingCalculation> {
        let base_price = product.base_price.multiply(quantity as f64);

        // Volume discount: 10% off for 10+ items, 20% off for 50+ items
        let volume_discount = match quantity {
            q if q >= 50 => base_price.multiply(0.20),
            q if q >= 10 => base_price.multiply(0.10),
            _ => Money::zero(),
        };

        // Customer tier discount
        let customer_discount = match customer.tier {
            CustomerTier::Platinum => base_price.multiply(0.15),
            CustomerTier::Gold => base_price.multiply(0.10),
            CustomerTier::Silver => base_price.multiply(0.05),
            CustomerTier::Bronze => Money::zero(),
        };

        // Seasonal promotion (Christmas season: 5% off)
        let seasonal_discount = if self.is_christmas_season() {
            base_price.multiply(0.05)
        } else {
            Money::zero()
        };

        let subtotal_after_discounts = base_price
            .subtract(&volume_discount)
            .subtract(&customer_discount)
            .subtract(&seasonal_discount);

        Ok(PricingCalculation {
            base_price,
            volume_discount,
            customer_discount,
            seasonal_discount,
            tax_amount: Money::zero(), // Calculated at order level
            final_price: subtotal_after_discounts,
            applied_rules: vec![
                format!("Quantity: {} (volume discount: {})", quantity, volume_discount),
                format!("Customer tier: {:?} (discount: {})", customer.tier, customer_discount),
                format!("Seasonal discount: {}", seasonal_discount),
            ],
        })
    }

    fn get_tax_rate(&self, address: &Address) -> Result<f64> {
        match address.country.as_str() {
            "US" => match address.state.as_str() {
                "CA" => Ok(0.0875), // California tax
                "NY" => Ok(0.08875), // New York tax
                "TX" => Ok(0.0625), // Texas tax
                _ => Ok(0.0), // Other states (simplified)
            },
            "CA" => Ok(0.13), // Canada (HST)
            "GB" => Ok(0.20), // UK VAT
            _ => Ok(0.0), // International (no tax for simplicity)
        }
    }

    fn is_christmas_season(&self) -> bool {
        let now = chrono::Utc::now();
        let year = now.year();
        let start = chrono::NaiveDate::from_ymd_opt(year, 11, 1)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| dt.and_utc())
            .unwrap();
        let end = chrono::NaiveDate::from_ymd_opt(year, 12, 31)
            .and_then(|d| d.and_hms_opt(23, 59, 59))
            .map(|dt| dt.and_utc())
            .unwrap();

        now >= start && now <= end
    }
}
```

#### Custom Command Handler Extension
```rust
// In generated src/application/commands/order_commands.rs
// <<< CUSTOM COMMANDS START >>>

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateOrderPricingCommand {
    pub order_id: String,
    pub customer_id: String,
    pub shipping_address: Address,
    pub currency: Option<Currency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderPricingResponse {
    pub order_id: String,
    pub pricing: PricingCalculation,
    pub currency: Currency,
    pub valid_until: DateTime<Utc>,
}

impl CalculateOrderPricingCommand {
    pub async fn handle(
        &self,
        order_service: &OrderService,
        pricing_service: &PricingService,
    ) -> Result<OrderPricingResponse> {
        // Get order
        let order = order_service.find_by_id(&self.order_id).await?
            .ok_or_else(|| anyhow::anyhow!("Order not found"))?;

        // Calculate pricing
        let mut pricing = pricing_service
            .calculate_order_total(&order, &self.customer_id, &self.shipping_address)
            .await?;

        // Apply currency conversion if needed
        if let Some(target_currency) = self.currency {
            pricing = self.convert_to_currency(pricing, target_currency).await?;
        }

        Ok(OrderPricingResponse {
            order_id: self.order_id.clone(),
            pricing,
            currency: self.currency.unwrap_or(Currency::USD),
            valid_until: chrono::Utc::now() + chrono::Duration::hours(1), // Prices valid for 1 hour
        })
    }

    async fn convert_to_currency(
        &self,
        pricing: PricingCalculation,
        target_currency: Currency,
    ) -> Result<PricingCalculation> {
        // Currency conversion logic
        let exchange_rate = self.get_exchange_rate(Currency::USD, target_currency).await?;

        Ok(PricingCalculation {
            base_price: pricing.base_price.convert(target_currency, exchange_rate),
            volume_discount: pricing.volume_discount.convert(target_currency, exchange_rate),
            customer_discount: pricing.customer_discount.convert(target_currency, exchange_rate),
            seasonal_discount: pricing.seasonal_discount.convert(target_currency, exchange_rate),
            tax_amount: pricing.tax_amount.convert(target_currency, exchange_rate),
            final_price: pricing.final_price.convert(target_currency, exchange_rate),
            applied_rules: pricing.applied_rules,
        })
    }

    async fn get_exchange_rate(&self, from: Currency, to: Currency) -> Result<f64> {
        // In real implementation, this would call a currency exchange API
        match (from, to) {
            (Currency::USD, Currency::EUR) => Ok(0.85),
            (Currency::USD, Currency::GBP) => Ok(0.73),
            (Currency::USD, Currency::JPY) => Ok(110.5),
            (Currency::USD, Currency::USD) => Ok(1.0),
            _ => Err(anyhow::anyhow!("Unsupported currency conversion")),
        }
    }
}

// <<< CUSTOM COMMANDS END >>>
```

## Example 2: Healthcare Patient Eligibility Service

### Scenario
A healthcare system needs to check patient eligibility for procedures based on:
- Insurance coverage
- Medical necessity
- Prior authorization requirements
- Network provider status

### Implementation

#### Custom Value Object for Insurance Coverage
```rust
// src/domain/value_objects/insurance_coverage.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsuranceCoverage {
    pub plan_id: String,
    pub provider_name: String,
    pub coverage_type: CoverageType,
    pub coverage_percentage: f32,
    pub deductible: Money,
    pub copay: Money,
    pub out_of_pocket_max: Money,
    pub covered_procedures: Vec<String>,
    pub excluded_procedures: Vec<String>,
    pub prior_auth_required: Vec<String>,
    pub network_status: NetworkStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoverageType {
    HMO,
    PPO,
    EPO,
    POS,
    Medicaid,
    Medicare,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NetworkStatus {
    InNetwork,
    OutOfNetwork,
    NotCovered,
}

impl InsuranceCoverage {
    pub fn new(
        plan_id: &str,
        provider_name: &str,
        coverage_type: CoverageType,
    ) -> Result<Self> {
        if plan_id.is_empty() {
            return Err(anyhow::anyhow!("Plan ID is required"));
        }

        Ok(Self {
            plan_id: plan_id.to_string(),
            provider_name: provider_name.to_string(),
            coverage_type,
            coverage_percentage: 0.0,
            deductible: Money::zero(),
            copay: Money::zero(),
            out_of_pocket_max: Money::zero(),
            covered_procedures: Vec::new(),
            excluded_procedures: Vec::new(),
            prior_auth_required: Vec::new(),
            network_status: NetworkStatus::InNetwork,
        })
    }

    pub fn covers_procedure(&self, procedure_code: &str) -> bool {
        // Check if explicitly covered
        if self.covered_procedures.contains(&procedure_code.to_string()) {
            return true;
        }

        // Check if explicitly excluded
        if self.excluded_procedures.contains(&procedure_code.to_string()) {
            return false;
        }

        // Check if in network for PPO/EPO/POS plans
        match self.coverage_type {
            CoverageType::PPO | CoverageType::EPO | CoverageType::POS => {
                matches!(self.network_status, NetworkStatus::InNetwork)
            }
            CoverageType::HMO => {
                // HMO requires in-network providers
                matches!(self.network_status, NetworkStatus::InNetwork)
            }
            CoverageType::Medicaid | CoverageType::Medicare => {
                // Government plans have different rules
                self.is_medically_necessary(procedure_code)
            }
        }
    }

    pub fn requires_prior_auth(&self, procedure_code: &str) -> bool {
        self.prior_auth_required.contains(&procedure_code.to_string())
    }

    pub fn calculate_patient_responsibility(
        &self,
        procedure_cost: Money,
    ) -> Money {
        // Apply deductible first
        let after_deductible = if self.deductible < procedure_cost {
            procedure_cost.subtract(&self.deductible)
        } else {
            Money::zero()
        };

        // Apply copay
        let after_copay = if self.copay < after_deductible {
            after_deductible.subtract(&self.copay)
        } else {
            Money::zero()
        };

        // Apply coinsurance
        let patient_responsibility = after_copay.multiply(1.0 - self.coverage_percentage / 100.0);

        // Ensure doesn't exceed out-of-pocket max
        let total_responsibility = patient_responsibility.add(&self.copay);
        if total_responsibility > self.out_of_pocket_max {
            self.out_of_pocket_max
        } else {
            total_responsibility
        }
    }

    fn is_medically_necessary(&self, procedure_code: &str) -> bool {
        // Simplified medical necessity check
        // In real implementation, this would check clinical guidelines
        !procedure_code.starts_with("COS") // Cosmetic procedures
    }
}
```

#### Custom Domain Service
```rust
// src/domain/services/eligibility_service.rs
use std::sync::Arc;
use anyhow::Result;

use crate::domain::entities::{Patient, Procedure, Provider};
use crate::domain::repositories::{PatientRepository, ProcedureRepository};
use crate::domain::value_objects::{InsuranceCoverage, Money};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityResult {
    pub patient_id: String,
    pub procedure_id: String,
    pub is_eligible: bool,
    pub coverage_details: CoverageDetails,
    pub requirements: Vec<EligibilityRequirement>,
    pub estimated_cost: CostBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageDetails {
    pub is_covered: bool,
    pub coverage_percentage: f32,
    pub requires_prior_auth: bool,
    pub network_status: NetworkStatus,
    pub deductible_remaining: Money,
    pub out_of_pocket_remaining: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityRequirement {
    pub requirement_type: RequirementType,
    pub description: String,
    pub is_satisfied: bool,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementType {
    PriorAuthorization,
    MedicalNecessity,
    Referral,
    PreCertification,
    ClinicalDocumentation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub total_cost: Money,
    pub insurance_pays: Money,
    pub patient_responsibility: Money,
    pub breakdown: Vec<CostComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostComponent {
    pub component_type: String,
    pub amount: Money,
    pub description: String,
}

pub struct EligibilityService {
    patient_repo: Arc<dyn PatientRepository>,
    procedure_repo: Arc<dyn ProcedureRepository>,
}

impl EligibilityService {
    pub async fn check_eligibility(
        &self,
        patient_id: &str,
        procedure_id: &str,
        provider_id: &str,
    ) -> Result<EligibilityResult> {
        // Get entities
        let patient = self.patient_repo.find_by_id(patient_id).await?
            .ok_or_else(|| anyhow::anyhow!("Patient not found"))?;

        let procedure = self.procedure_repo.find_by_id(procedure_id).await?
            .ok_or_else(|| anyhow::anyhow!("Procedure not found"))?;

        // Check insurance coverage
        let coverage_details = self.check_coverage(&patient, &procedure, provider_id).await?;

        // Check requirements
        let requirements = self.check_requirements(&patient, &procedure).await?;

        // Calculate costs
        let estimated_cost = self.estimate_costs(&patient, &procedure, &coverage_details).await?;

        // Determine overall eligibility
        let is_eligible = coverage_details.is_covered &&
            requirements.iter().all(|r| r.is_satisfied);

        Ok(EligibilityResult {
            patient_id: patient_id.to_string(),
            procedure_id: procedure_id.to_string(),
            is_eligible,
            coverage_details,
            requirements,
            estimated_cost,
        })
    }

    async fn check_coverage(
        &self,
        patient: &Patient,
        procedure: &Procedure,
        provider_id: &str,
    ) -> Result<CoverageDetails> {
        let insurance = &patient.insurance_coverage;

        let is_covered = insurance.covers_procedure(&procedure.code);
        let requires_prior_auth = insurance.requires_prior_auth(&procedure.code);

        // Check network status (simplified)
        let network_status = self.check_provider_network(provider_id, &insurance.plan_id).await?;

        Ok(CoverageDetails {
            is_covered,
            coverage_percentage: insurance.coverage_percentage,
            requires_prior_auth,
            network_status,
            deductible_remaining: insurance.deductible, // Simplified
            out_of_pocket_remaining: insurance.out_of_pocket_max, // Simplified
        })
    }

    async fn check_requirements(
        &self,
        patient: &Patient,
        procedure: &Procedure,
    ) -> Result<Vec<EligibilityRequirement>> {
        let mut requirements = Vec::new();

        // Check prior authorization
        if patient.insurance_coverage.requires_prior_auth(&procedure.code) {
            let auth_status = self.check_prior_auth_status(
                &patient.id,
                &procedure.code,
            ).await?;

            requirements.push(EligibilityRequirement {
                requirement_type: RequirementType::PriorAuthorization,
                description: "Prior authorization required".to_string(),
                is_satisfied: auth_status.is_approved,
                details: Some(auth_status.details),
            });
        }

        // Check medical necessity
        let medical_necessity = self.check_medical_necessity(
            &patient.id,
            &procedure.code,
            &patient.medical_history,
        ).await?;

        requirements.push(EligibilityRequirement {
            requirement_type: RequirementType::MedicalNecessity,
            description: "Medical necessity must be established".to_string(),
            is_satisfied: medical_necessity.is_met,
            details: medical_necessity.justification,
        });

        // Check age requirements
        if procedure.min_age.is_some() || procedure.max_age.is_some() {
            let age_requirement = self.check_age_requirements(patient, procedure);
            requirements.push(age_requirement);
        }

        Ok(requirements)
    }

    async fn estimate_costs(
        &self,
        patient: &Patient,
        procedure: &Procedure,
        coverage: &CoverageDetails,
    ) -> Result<CostBreakdown> {
        let total_cost = procedure.base_cost;

        let insurance_pays = if coverage.is_covered {
            let patient_responsibility = patient
                .insurance_coverage
                .calculate_patient_responsibility(total_cost);
            total_cost.subtract(&patient_responsibility)
        } else {
            Money::zero()
        };

        let patient_responsibility = total_cost.subtract(&insurance_pays);

        let breakdown = vec![
            CostComponent {
                component_type: "Base Cost".to_string(),
                amount: total_cost,
                description: format!("{} procedure cost", procedure.name),
            },
            CostComponent {
                component_type: "Insurance Payment".to_string(),
                amount: insurance_pays,
                description: format!("{}% coverage", coverage.coverage_percentage),
            },
            CostComponent {
                component_type: "Patient Responsibility".to_string(),
                amount: patient_responsibility,
                description: "Deductible, copay, and coinsurance".to_string(),
            },
        ];

        Ok(CostBreakdown {
            total_cost,
            insurance_pays,
            patient_responsibility,
            breakdown,
        })
    }

    async fn check_provider_network(
        &self,
        provider_id: &str,
        plan_id: &str,
    ) -> Result<NetworkStatus> {
        // In real implementation, this would query insurance provider networks
        // For this example, we'll simulate network lookup
        if provider_id.starts_with("IN_") {
            Ok(NetworkStatus::InNetwork)
        } else if provider_id.starts_with("OUT_") {
            Ok(NetworkStatus::OutOfNetwork)
        } else {
            Ok(NetworkStatus::NotCovered)
        }
    }

    async fn check_prior_auth_status(
        &self,
        patient_id: &str,
        procedure_code: &str,
    ) -> Result<PriorAuthStatus> {
        // Check if prior authorization has been obtained
        // In real implementation, this would query authorization systems
        Ok(PriorAuthStatus {
            is_approved: true, // Simplified
            auth_number: Some("PA-123456".to_string()),
            expiration_date: Some(chrono::Utc::now() + chrono::Duration::days(30)),
            details: Some("Prior authorization approved".to_string()),
        })
    }

    async fn check_medical_necessity(
        &self,
        patient_id: &str,
        procedure_code: &str,
        medical_history: &MedicalHistory,
    ) -> Result<MedicalNecessityResult> {
        // Complex medical necessity logic based on clinical guidelines
        // This would typically involve expert systems or AI models

        let is_met = match procedure_code.as_str() {
            "MRI_BRAIN" => medical_history.has_symptoms(&["headache", "dizziness", "vision_changes"]),
            "CT_CHEST" => medical_history.has_diagnosis(&["pneumonia", "lung_cancer", "chest_pain"]),
            "COLONOSCOPY" => {
                let age = medical_history.age;
                age >= 45 || medical_history.has_family_history("colon_cancer")
            }
            _ => true, // Default to medically necessary
        };

        Ok(MedicalNecessityResult {
            is_met,
            justification: Some(if is_met {
                "Procedure meets medical necessity criteria".to_string()
            } else {
                "Insufficient medical justification".to_string()
            }),
        })
    }

    fn check_age_requirements(
        &self,
        patient: &Patient,
        procedure: &Procedure,
    ) -> EligibilityRequirement {
        let patient_age = patient.age();
        let meets_min = procedure.min_age.map_or(true, |min| patient_age >= min);
        let meets_max = procedure.max_age.map_or(true, |max| patient_age <= max);
        let is_satisfied = meets_min && meets_max;

        let description = format!(
            "Age requirement: {}-{} years",
            procedure.min_age.unwrap_or(0),
            procedure.max_age.unwrap_or(999)
        );

        EligibilityRequirement {
            requirement_type: RequirementType::ClinicalDocumentation,
            description,
            is_satisfied,
            details: Some(format!("Patient age: {}", patient_age)),
        }
    }
}

// Supporting types
#[derive(Debug, Clone)]
pub struct PriorAuthStatus {
    pub is_approved: bool,
    pub auth_number: Option<String>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub details: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MedicalNecessityResult {
    pub is_met: bool,
    pub justification: Option<String>,
}
```

## Example 3: Financial Fraud Detection Service

### Scenario
A financial services platform needs to detect potentially fraudulent transactions using:
- Pattern recognition
- Behavioral analysis
- Risk scoring
- Real-time monitoring

### Implementation

#### Custom Domain Service
```rust
// src/domain/services/fraud_detection_service.rs
use std::sync::Arc;
use anyhow::Result;

use crate::domain::entities::{Transaction, Customer, Device};
use crate::domain::repositories::{TransactionRepository, CustomerRepository};
use crate::domain::value_objects::{Money, RiskScore, TransactionType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudDetectionResult {
    pub transaction_id: String,
    pub risk_score: RiskScore,
    pub risk_level: RiskLevel,
    pub detected_patterns: Vec<FraudPattern>,
    pub recommended_actions: Vec<RecommendedAction>,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudPattern {
    pub pattern_type: PatternType,
    pub description: String,
    pub confidence: f32,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternType {
    UnusualLocation,
    VelocityThreshold,
    AmountAnomaly,
    DeviceFingerprint,
    BehavioralPattern,
    KnownFraudulentActivity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub action_type: ActionType,
    pub description: String,
    pub priority: u8,
    pub automated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionType {
    Allow,
    FlagForReview,
    RequireAdditionalVerification,
    BlockTransaction,
    FreezeAccount,
    NotifySecurity,
    ReportToCompliance,
}

pub struct FraudDetectionService {
    transaction_repo: Arc<dyn TransactionRepository>,
    customer_repo: Arc<dyn CustomerRepository>,
}

impl FraudDetectionService {
    pub async fn analyze_transaction(
        &self,
        transaction: &Transaction,
        device_info: Option<&DeviceInfo>,
        location_info: Option<&LocationInfo>,
    ) -> Result<FraudDetectionResult> {
        let mut patterns = Vec::new();
        let mut total_score = 0.0;

        // Get customer history
        let customer = self.customer_repo.find_by_id(&transaction.customer_id).await?
            .ok_or_else(|| anyhow::anyhow!("Customer not found"))?;

        // Check various fraud patterns
        if let Some(location) = location_info {
            let location_pattern = self.check_unusual_location(&customer, transaction, location).await?;
            if location_pattern.is_some() {
                total_score += location_pattern.as_ref().unwrap().confidence * 0.3;
                patterns.push(location_pattern.unwrap());
            }
        }

        let velocity_pattern = self.check_transaction_velocity(&customer, transaction).await?;
        if velocity_pattern.is_some() {
            total_score += velocity_pattern.as_ref().unwrap().confidence * 0.25;
            patterns.push(velocity_pattern.unwrap());
        }

        let amount_pattern = self.check_amount_anomaly(&customer, transaction).await?;
        if amount_pattern.is_some() {
            total_score += amount_pattern.as_ref().unwrap().confidence * 0.2;
            patterns.push(amount_pattern.unwrap());
        }

        if let Some(device) = device_info {
            let device_pattern = self.check_device_risk(&customer, transaction, device).await?;
            if device_pattern.is_some() {
                total_score += device_pattern.as_ref().unwrap().confidence * 0.15;
                patterns.push(device_pattern.unwrap());
            }
        }

        let behavioral_pattern = self.check_behavioral_anomalies(&customer, transaction).await?;
        if behavioral_pattern.is_some() {
            total_score += behavioral_pattern.as_ref().unwrap().confidence * 0.1;
            patterns.push(behavioral_pattern.unwrap());
        }

        // Calculate final risk score and level
        let risk_score = RiskScore::new(total_score)?;
        let risk_level = self.determine_risk_level(risk_score.value);
        let recommended_actions = self.get_recommended_actions(risk_level, &patterns);

        Ok(FraudDetectionResult {
            transaction_id: transaction.id.clone(),
            risk_score,
            risk_level,
            detected_patterns: patterns,
            recommended_actions,
            confidence: total_score,
        })
    }

    async fn check_unusual_location(
        &self,
        customer: &Customer,
        transaction: &Transaction,
        location: &LocationInfo,
    ) -> Result<Option<FraudPattern>> {
        // Get recent transactions for location comparison
        let recent_transactions = self.transaction_repo
            .find_recent_by_customer(&customer.id, 30).await?; // Last 30 days

        if recent_transactions.is_empty() {
            return Ok(None); // No history for comparison
        }

        // Calculate average distance from usual locations
        let usual_locations: Vec<&LocationInfo> = recent_transactions
            .iter()
            .filter_map(|t| t.location_info.as_ref())
            .collect();

        if usual_locations.is_empty() {
            return Ok(None);
        }

        let min_distance = usual_locations
            .iter()
            .map(|usual_loc| calculate_distance(location, usual_loc))
            .fold(f64::INFINITY, f64::min);

        // If transaction is very far from usual locations, flag as unusual
        if min_distance > 500.0 { // 500km threshold
            Ok(Some(FraudPattern {
                pattern_type: PatternType::UnusualLocation,
                description: format!(
                    "Transaction location {:.1}km from usual locations",
                    min_distance
                ),
                confidence: (min_distance / 1000.0).min(1.0), // Scale confidence by distance
                severity: if min_distance > 2000.0 {
                    Severity::Critical
                } else if min_distance > 1000.0 {
                    Severity::High
                } else {
                    Severity::Medium
                },
            }))
        } else {
            Ok(None)
        }
    }

    async fn check_transaction_velocity(
        &self,
        customer: &Customer,
        transaction: &Transaction,
    ) -> Result<Option<FraudPattern>> {
        // Check number of transactions in last hour
        let recent_hour = self.transaction_repo
            .find_recent_by_customer_and_timeframe(
                &customer.id,
                chrono::Utc::now() - chrono::Duration::hours(1),
            ).await?;

        let hour_count = recent_hour.len();

        // Check total amount in last hour
        let hour_total: Money = recent_hour
            .iter()
            .fold(Money::zero(), |acc, t| acc.add(&t.amount));

        // Get customer's normal patterns
        let avg_hourly_count = customer.behavioral_patterns.avg_hourly_transactions;
        let avg_hourly_amount = customer.behavioral_patterns.avg_hourly_amount;

        let count_multiplier = (hour_count as f64 / avg_hourly_count as f64).max(1.0);
        let amount_multiplier = hour_total.divide(&avg_hourly_amount).max(1.0);

        let velocity_score = (count_multiplier + amount_multiplier) / 2.0;

        if velocity_score > 5.0 { // 5x normal activity
            Ok(Some(FraudPattern {
                pattern_type: PatternType::VelocityThreshold,
                description: format!(
                    "Unusual transaction velocity: {} transactions, {} in last hour",
                    hour_count,
                    hour_total
                ),
                confidence: (velocity_score / 10.0).min(1.0),
                severity: if velocity_score > 10.0 {
                    Severity::Critical
                } else if velocity_score > 7.0 {
                    Severity::High
                } else {
                    Severity::Medium
                },
            }))
        } else {
            Ok(None)
        }
    }

    async fn check_amount_anomaly(
        &self,
        customer: &Customer,
        transaction: &Transaction,
    ) -> Result<Option<FraudPattern>> {
        let avg_amount = customer.behavioral_patterns.avg_transaction_amount;
        let amount_multiplier = transaction.amount.divide(&avg_amount);

        // Check if amount is significantly higher than usual
        if amount_multiplier > 10.0 { // 10x normal amount
            Ok(Some(FraudPattern {
                pattern_type: PatternType::AmountAnomaly,
                description: format!(
                    "Transaction amount {} is {}x higher than average {}",
                    transaction.amount,
                    amount_multiplier,
                    avg_amount
                ),
                confidence: (amount_multiplier / 20.0).min(1.0),
                severity: if amount_multiplier > 50.0 {
                    Severity::Critical
                } else if amount_multiplier > 25.0 {
                    Severity::High
                } else {
                    Severity::Medium
                },
            }))
        } else {
            Ok(None)
        }
    }

    async fn check_device_risk(
        &self,
        customer: &Customer,
        transaction: &Transaction,
        device: &DeviceInfo,
    ) -> Result<Option<FraudPattern>> {
        // Check if device is known for this customer
        let known_devices = self.transaction_repo
            .find_devices_used_by_customer(&customer.id).await?;

        let is_known_device = known_devices.iter().any(|d| d.fingerprint == device.fingerprint);

        if !is_known_device {
            // Check if device is in known fraud list
            let is_fraud_device = self.is_known_fraud_device(device).await?;

            Ok(Some(FraudPattern {
                pattern_type: PatternType::DeviceFingerprint,
                description: if is_fraud_device {
                    "Device associated with known fraudulent activity".to_string()
                } else {
                    "New device not previously used by customer".to_string()
                },
                confidence: if is_fraud_device { 0.9 } else { 0.5 },
                severity: if is_fraud_device { Severity::Critical } else { Severity::Medium },
            }))
        } else {
            Ok(None)
        }
    }

    async fn check_behavioral_anomalies(
        &self,
        customer: &Customer,
        transaction: &Transaction,
    ) -> Result<Option<FraudPattern>> {
        // Check transaction type frequency
        let recent_same_type = self.transaction_repo
            .find_recent_by_customer_and_type(
                &customer.id,
                &transaction.transaction_type,
                chrono::Utc::now() - chrono::Duration::days(7),
            ).await?;

        let type_frequency = customer.behavioral_patterns
            .transaction_type_frequencies
            .get(&transaction.transaction_type)
            .copied()
            .unwrap_or(0.0);

        let actual_frequency = (recent_same_type.len() as f64) / 7.0; // per day

        if actual_frequency > type_frequency * 5.0 {
            Ok(Some(FraudPattern {
                pattern_type: PatternType::BehavioralPattern,
                description: format!(
                    "Unusual frequency of {} transactions: {:.1}/day vs usual {:.1}/day",
                    format!("{:?}", transaction.transaction_type).to_lowercase(),
                    actual_frequency,
                    type_frequency
                ),
                confidence: (actual_frequency / (type_frequency * 10.0)).min(1.0),
                severity: Severity::Medium,
            }))
        } else {
            Ok(None)
        }
    }

    fn determine_risk_level(&self, risk_score: f64) -> RiskLevel {
        match risk_score {
            s if s >= 0.8 => RiskLevel::Critical,
            s if s >= 0.6 => RiskLevel::High,
            s if s >= 0.3 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }

    fn get_recommended_actions(
        &self,
        risk_level: RiskLevel,
        patterns: &[FraudPattern],
    ) -> Vec<RecommendedAction> {
        match risk_level {
            RiskLevel::Critical => vec![
                RecommendedAction {
                    action_type: ActionType::BlockTransaction,
                    description: "Block transaction immediately".to_string(),
                    priority: 1,
                    automated: true,
                },
                RecommendedAction {
                    action_type: ActionType::FreezeAccount,
                    description: "Freeze customer account for security review".to_string(),
                    priority: 2,
                    automated: false,
                },
                RecommendedAction {
                    action_type: ActionType::NotifySecurity,
                    description: "Notify security team immediately".to_string(),
                    priority: 1,
                    automated: true,
                },
            ],
            RiskLevel::High => vec![
                RecommendedAction {
                    action_type: ActionType::RequireAdditionalVerification,
                    description: "Require additional verification".to_string(),
                    priority: 1,
                    automated: true,
                },
                RecommendedAction {
                    action_type: ActionType::FlagForReview,
                    description: "Flag for manual review".to_string(),
                    priority: 2,
                    automated: true,
                },
            ],
            RiskLevel::Medium => vec![
                RecommendedAction {
                    action_type: ActionType::FlagForReview,
                    description: "Flag for review within 24 hours".to_string(),
                    priority: 2,
                    automated: true,
                },
            ],
            RiskLevel::Low => vec![
                RecommendedAction {
                    action_type: ActionType::Allow,
                    description: "Allow transaction with monitoring".to_string(),
                    priority: 3,
                    automated: true,
                },
            ],
        }
    }

    async fn is_known_fraud_device(&self, device: &DeviceInfo) -> Result<bool> {
        // In real implementation, this would check against fraud databases
        // For example, device intelligence services, shared fraud lists, etc.

        // Simulate check against known fraud patterns
        let suspicious_patterns = vec![
            "emulator",
            "rooted",
            "jailbreak",
            "vpn",
            "tor",
        ];

        Ok(suspicious_patterns.iter().any(|pattern| {
            device.user_agent.to_lowercase().contains(pattern) ||
            device.device_info.to_lowercase().contains(pattern)
        }))
    }
}

// Helper functions
fn calculate_distance(loc1: &LocationInfo, loc2: &LocationInfo) -> f64 {
    // Haversine formula for calculating distance between two coordinates
    let earth_radius = 6371.0; // Earth's radius in kilometers

    let lat1_rad = loc1.latitude.to_radians();
    let lat2_rad = loc2.latitude.to_radians();
    let delta_lat = (loc2.latitude - loc1.latitude).to_radians();
    let delta_lon = (loc2.longitude - loc1.longitude).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2) +
        lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    earth_radius * c
}

// Supporting types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub fingerprint: String,
    pub user_agent: String,
    pub device_info: String,
    pub ip_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub latitude: f64,
    pub longitude: f64,
    pub country: String,
    pub city: String,
}
```