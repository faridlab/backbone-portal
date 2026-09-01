use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::str::FromStr;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "portal_audit_event", rename_all = "snake_case")]
pub enum PortalAuditEvent {
    InviteMinted,
    InviteRevoked,
    InviteRedeemed,
    SignupRefusedPolicyClosed,
    SignupCreated,
    PolicyChanged,
    BearerMinted,
    BearerRotated,
    BearerRevoked,
    RecipientHmacMinted,
    LoginSucceeded,
    LoginRefused,
    LifecycleAccessRevoked,
    OnboardingLinked,
    OnboardingNoInvitation,
    CredentialPortNotComposed,
}

impl std::fmt::Display for PortalAuditEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InviteMinted => write!(f, "invite_minted"),
            Self::InviteRevoked => write!(f, "invite_revoked"),
            Self::InviteRedeemed => write!(f, "invite_redeemed"),
            Self::SignupRefusedPolicyClosed => write!(f, "signup_refused_policy_closed"),
            Self::SignupCreated => write!(f, "signup_created"),
            Self::PolicyChanged => write!(f, "policy_changed"),
            Self::BearerMinted => write!(f, "bearer_minted"),
            Self::BearerRotated => write!(f, "bearer_rotated"),
            Self::BearerRevoked => write!(f, "bearer_revoked"),
            Self::RecipientHmacMinted => write!(f, "recipient_hmac_minted"),
            Self::LoginSucceeded => write!(f, "login_succeeded"),
            Self::LoginRefused => write!(f, "login_refused"),
            Self::LifecycleAccessRevoked => write!(f, "lifecycle_access_revoked"),
            Self::OnboardingLinked => write!(f, "onboarding_linked"),
            Self::OnboardingNoInvitation => write!(f, "onboarding_no_invitation"),
            Self::CredentialPortNotComposed => write!(f, "credential_port_not_composed"),
        }
    }
}

impl FromStr for PortalAuditEvent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "invite_minted" => Ok(Self::InviteMinted),
            "invite_revoked" => Ok(Self::InviteRevoked),
            "invite_redeemed" => Ok(Self::InviteRedeemed),
            "signup_refused_policy_closed" => Ok(Self::SignupRefusedPolicyClosed),
            "signup_created" => Ok(Self::SignupCreated),
            "policy_changed" => Ok(Self::PolicyChanged),
            "bearer_minted" => Ok(Self::BearerMinted),
            "bearer_rotated" => Ok(Self::BearerRotated),
            "bearer_revoked" => Ok(Self::BearerRevoked),
            "recipient_hmac_minted" => Ok(Self::RecipientHmacMinted),
            "login_succeeded" => Ok(Self::LoginSucceeded),
            "login_refused" => Ok(Self::LoginRefused),
            "lifecycle_access_revoked" => Ok(Self::LifecycleAccessRevoked),
            "onboarding_linked" => Ok(Self::OnboardingLinked),
            "onboarding_no_invitation" => Ok(Self::OnboardingNoInvitation),
            "credential_port_not_composed" => Ok(Self::CredentialPortNotComposed),
            _ => Err(format!("Unknown PortalAuditEvent variant: {}", s)),
        }
    }
}

impl Default for PortalAuditEvent {
    fn default() -> Self {
        Self::InviteMinted
    }
}
