use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::str::FromStr;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "portal_token_status", rename_all = "snake_case")]
pub enum PortalTokenStatus {
    Active,
    Rotated,
    Revoked,
}

impl std::fmt::Display for PortalTokenStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Rotated => write!(f, "rotated"),
            Self::Revoked => write!(f, "revoked"),
        }
    }
}

impl FromStr for PortalTokenStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "rotated" => Ok(Self::Rotated),
            "revoked" => Ok(Self::Revoked),
            _ => Err(format!("Unknown PortalTokenStatus variant: {}", s)),
        }
    }
}

impl Default for PortalTokenStatus {
    fn default() -> Self {
        Self::Active
    }
}
