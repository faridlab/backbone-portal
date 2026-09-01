use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::str::FromStr;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "portal_user_status", rename_all = "snake_case")]
pub enum PortalUserStatus {
    Invited,
    Active,
    Revoked,
    Archived,
}

impl std::fmt::Display for PortalUserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invited => write!(f, "invited"),
            Self::Active => write!(f, "active"),
            Self::Revoked => write!(f, "revoked"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

impl FromStr for PortalUserStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "invited" => Ok(Self::Invited),
            "active" => Ok(Self::Active),
            "revoked" => Ok(Self::Revoked),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("Unknown PortalUserStatus variant: {}", s)),
        }
    }
}

impl Default for PortalUserStatus {
    fn default() -> Self {
        Self::Invited
    }
}
