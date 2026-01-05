//! Auth domain data access (Users, Sessions, Invitations, JoinRequests)

use sea_orm::{prelude::*, ActiveValue::Set, QueryOrder, sea_query::Expr};
use uuid::Uuid;
use chrono::{Utc, Duration};

use crate::infrastructure::store::TigerDataStore;
use crate::internal::DbStoreError;
use crate::primitives::{UserId, TenantId, SessionId, MagicLinkId, InvitationId, JoinRequestId, ConsentRecordId, AuditLogId, ThreadId, InviteRole};
use crate::types::auth::{
    User, CreateUserInput, UpdateUserInput,
    AuthSession, CreateSessionInput,
    MagicLink, MagicLinkPurpose, CreateMagicLinkInput,
    FamilyInvitation, InviteType, CreateEmailInviteInput, CreateCodeInviteInput,
    JoinRequest, JoinRequestStatus, CreateJoinRequestInput, ReviewJoinRequestInput,
    ConsentRecord, ConsentType, RecordConsentInput,
    AuditLogEntry, CreateAuditLogInput,
    UserMembership,
};
use crate::types::base::SystemEntityMeta;
use crate::components::Timestamps;
use crate::entities::db::auth::{user, session, magic_link, invitation, join_request, consent, audit};
use crate::entities::db::conversation::{tenant, tenant_member};

impl TigerDataStore {
    // ========================================================================
    // User Operations
    // ========================================================================

    pub async fn create_user(&self, input: CreateUserInput) -> Result<User, DbStoreError> {
        let now = Utc::now();
        let id = UserId::new();
        
        let model = user::ActiveModel {
            id: Set(id),
            email: Set(input.email.clone()),
            email_verified: Set(false),
            password_hash: Set(input.password_hash.clone()),
            name: Set(input.name.clone()),
            avatar_url: Set(input.avatar_url.clone()),
            primary_tenant_id: Set(input.primary_tenant_id),
            settings: Set(serde_json::json!({})),
            gdpr_consents: Set(serde_json::json!({})),
            deletion_requested_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        
        user::Entity::insert(model).exec(&self.db).await?;
        
        Ok(User {
            meta: SystemEntityMeta {
                id: id.as_uuid().into(),
                timestamps: Timestamps { created_at: now, updated_at: now },
            },
            email: input.email,
            email_verified: false,
            name: input.name,
            avatar_url: input.avatar_url,
            primary_tenant_id: input.primary_tenant_id,
            settings: serde_json::json!({}),
            gdpr_consents: serde_json::json!({}),
            deletion_requested_at: None,
        })
    }

    pub async fn get_user(&self, user_id: UserId) -> Result<Option<User>, DbStoreError> {
        let result = user::Entity::find_by_id(user_id).one(&self.db).await?;
        
        Ok(result.map(|m| User {
            meta: SystemEntityMeta {
                id: m.id.as_uuid().into(),
                timestamps: Timestamps { created_at: m.created_at, updated_at: m.updated_at },
            },
            email: m.email,
            email_verified: m.email_verified,
            name: m.name,
            avatar_url: m.avatar_url,
            primary_tenant_id: m.primary_tenant_id,
            settings: m.settings,
            gdpr_consents: m.gdpr_consents,
            deletion_requested_at: m.deletion_requested_at,
        }))
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DbStoreError> {
        let result = user::Entity::find().filter(user::Column::Email.eq(email)).one(&self.db).await?;
        
        Ok(result.map(|m| User {
            meta: SystemEntityMeta {
                id: m.id.into(),
                timestamps: Timestamps { created_at: m.created_at, updated_at: m.updated_at },
            },
            email: m.email,
            email_verified: m.email_verified,
            name: m.name,
            avatar_url: m.avatar_url,
            primary_tenant_id: m.primary_tenant_id.map(|t| t.into()),
            settings: m.settings,
            gdpr_consents: m.gdpr_consents,
            deletion_requested_at: m.deletion_requested_at,
        }))
    }

    pub async fn get_user_password_hash(&self, user_id: Uuid) -> Result<Option<String>, DbStoreError> {
        let result = user::Entity::find_by_id(user_id).one(&self.db).await?;
        Ok(result.and_then(|m| m.password_hash))
    }

    pub async fn update_user(&self, user_id: Uuid, input: UpdateUserInput) -> Result<(), DbStoreError> {
        let now = Utc::now();
        let mut model: user::ActiveModel = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbStoreError::not_found("User", user_id.to_string()))?
            .into();
        
        if let Some(name) = input.name { model.name = Set(name); }
        if let Some(avatar) = input.avatar_url { model.avatar_url = Set(Some(avatar)); }
        if let Some(tenant) = input.primary_tenant_id { model.primary_tenant_id = Set(Some(tenant)); }
        if let Some(settings) = input.settings { model.settings = Set(settings); }
        model.updated_at = Set(now);
        
        model.update(&self.db).await?;
        Ok(())
    }

    pub async fn verify_user_email(&self, user_id: Uuid) -> Result<(), DbStoreError> {
        let now = Utc::now();
        user::Entity::update_many()
            .col_expr(user::Column::EmailVerified, Expr::value(true))
            .col_expr(user::Column::UpdatedAt, Expr::value(now))
            .filter(user::Column::Id.eq(user_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn update_user_password(&self, user_id: Uuid, password_hash: &str) -> Result<(), DbStoreError> {
        let now = Utc::now();
        user::Entity::update_many()
            .col_expr(user::Column::PasswordHash, Expr::value(Some(password_hash)))
            .col_expr(user::Column::UpdatedAt, Expr::value(now))
            .filter(user::Column::Id.eq(user_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    // ========================================================================
    // Session Operations
    // ========================================================================

    pub async fn create_session(&self, input: CreateSessionInput) -> Result<AuthSession, DbStoreError> {
        let now = Utc::now();
        let id = SessionId::new();
        let expires_at = now + Duration::hours(input.expires_in_hours);
        
        let model = session::ActiveModel {
            id: Set(id),
            user_id: Set(input.user_id),
            token_hash: Set(input.token_hash.clone()),
            user_agent: Set(input.user_agent.clone()),
            ip_address: Set(input.ip_address.clone()),
            expires_at: Set(expires_at),
            created_at: Set(now),
            version: Set(0), // Initial version for optimistic locking
        };
        
        session::Entity::insert(model).exec(&self.db).await?;
        
        Ok(AuthSession {
            id: id.as_uuid().into(),
            user_id: input.user_id,
            token_hash: input.token_hash,
            user_agent: input.user_agent,
            ip_address: input.ip_address,
            expires_at,
            created_at: now,
        })
    }

    pub async fn validate_session(&self, token_hash: &str) -> Result<Option<UserId>, DbStoreError> {
        let now = Utc::now();
        let result = session::Entity::find()
            .filter(session::Column::TokenHash.eq(token_hash))
            .filter(session::Column::ExpiresAt.gt(now))
            .one(&self.db)
            .await?;
        
        Ok(result.map(|m| m.user_id))
    }

    pub async fn delete_session(&self, session_id: SessionId) -> Result<(), DbStoreError> {
        session::Entity::delete_by_id(session_id).exec(&self.db).await?;
        Ok(())
    }

    pub async fn delete_user_sessions(&self, user_id: UserId) -> Result<(), DbStoreError> {
        session::Entity::delete_many()
            .filter(session::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    // ========================================================================
    // Magic Link Operations
    // ========================================================================

    pub async fn create_magic_link(&self, input: CreateMagicLinkInput, token_hash: String) -> Result<MagicLink, DbStoreError> {
        let now = Utc::now();
        let id = MagicLinkId::new();
        let expires_at = now + Duration::minutes(input.expires_in_minutes);
        
        let model = magic_link::ActiveModel {
            id: Set(id),
            email: Set(input.email.clone()),
            token_hash: Set(token_hash.clone()),
            purpose: Set(match input.purpose {
                MagicLinkPurpose::Login => magic_link::MagicLinkPurpose::Login,
                MagicLinkPurpose::Signup => magic_link::MagicLinkPurpose::Signup,
                MagicLinkPurpose::VerifyEmail => magic_link::MagicLinkPurpose::VerifyEmail,
                MagicLinkPurpose::PasswordReset => magic_link::MagicLinkPurpose::PasswordReset,
            }),
            metadata: Set(input.metadata.clone().unwrap_or_default()),
            expires_at: Set(expires_at),
            used_at: Set(None),
            created_at: Set(now),
        };
        
        magic_link::Entity::insert(model).exec(&self.db).await?;
        
        Ok(MagicLink {
            id: id.into(),
            email: input.email,
            purpose: input.purpose,
            metadata: input.metadata.unwrap_or_default(),
            expires_at,
            used_at: None,
            created_at: now,
        })
    }

    pub async fn get_magic_link_by_token(&self, token_hash: &str) -> Result<Option<MagicLink>, DbStoreError> {
        let result = magic_link::Entity::find()
            .filter(magic_link::Column::TokenHash.eq(token_hash))
            .one(&self.db)
            .await?;
        
        Ok(result.map(|m| MagicLink {
            id: m.id.into(),
            email: m.email,
            purpose: match m.purpose {
                magic_link::MagicLinkPurpose::Login => MagicLinkPurpose::Login,
                magic_link::MagicLinkPurpose::Signup => MagicLinkPurpose::Signup,
                magic_link::MagicLinkPurpose::VerifyEmail => MagicLinkPurpose::VerifyEmail,
                magic_link::MagicLinkPurpose::PasswordReset => MagicLinkPurpose::PasswordReset,
            },
            metadata: m.metadata,
            expires_at: m.expires_at,
            used_at: m.used_at,
            created_at: m.created_at,
        }))
    }

    pub async fn consume_magic_link(&self, link_id: Uuid) -> Result<(), DbStoreError> {
        let now = Utc::now();
        magic_link::Entity::update_many()
            .col_expr(magic_link::Column::UsedAt, Expr::value(Some(now)))
            .filter(magic_link::Column::Id.eq(link_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    // ========================================================================
    // Consent Operations
    // ========================================================================

    pub async fn record_consent(&self, user_id: UserId, input: RecordConsentInput, ip_address: Option<String>, user_agent: Option<String>) -> Result<ConsentRecord, DbStoreError> {
        let now = Utc::now();
        let id = ConsentRecordId::new();
        
        let model = consent::ActiveModel {
            id: Set(id),
            user_id: Set(user_id),
            consent_type: Set(match input.consent_type {
                ConsentType::TermsOfService => consent::ConsentType::TermsOfService,
                ConsentType::PrivacyPolicy => consent::ConsentType::PrivacyPolicy,
                ConsentType::MarketingEmails => consent::ConsentType::MarketingEmails,
                ConsentType::AiProcessing => consent::ConsentType::AiProcessing,
                ConsentType::DataSharing => consent::ConsentType::DataSharing,
                ConsentType::Analytics => consent::ConsentType::Analytics,
            }),
            granted: Set(input.granted),
            version: Set(input.version.clone()),
            ip_address: Set(ip_address.clone()),
            user_agent: Set(user_agent.clone()),
            created_at: Set(now),
        };
        
        consent::Entity::insert(model).exec(&self.db).await?;
        
        Ok(ConsentRecord {
            id: id.as_uuid(),
            user_id,
            consent_type: input.consent_type,
            granted: input.granted,
            version: input.version,
            ip_address,
            user_agent,
            created_at: now,
        })
    }

    pub async fn get_consent_records(&self, user_id: UserId) -> Result<Vec<ConsentRecord>, DbStoreError> {
        let results: Vec<consent::Model> = consent::Entity::find()
            .filter(consent::Column::UserId.eq(user_id))
            .order_by_desc(consent::Column::CreatedAt)
            .all(&self.db)
            .await?;
        
        Ok(results.into_iter().map(|m| ConsentRecord {
            id: m.id.as_uuid(),
            user_id: m.user_id,
            consent_type: match m.consent_type {
                consent::ConsentType::TermsOfService => ConsentType::TermsOfService,
                consent::ConsentType::PrivacyPolicy => ConsentType::PrivacyPolicy,
                consent::ConsentType::MarketingEmails => ConsentType::MarketingEmails,
                consent::ConsentType::AiProcessing => ConsentType::AiProcessing,
                consent::ConsentType::DataSharing => ConsentType::DataSharing,
                consent::ConsentType::Analytics => ConsentType::Analytics,
            },
            granted: m.granted,
            version: m.version,
            ip_address: m.ip_address,
            user_agent: m.user_agent,
            created_at: m.created_at,
        }).collect())
    }

    // ========================================================================
    // Audit Log Operations
    // ========================================================================

    pub async fn create_audit_log(&self, input: CreateAuditLogInput) -> Result<AuditLogEntry, DbStoreError> {
        let now = Utc::now();
        let id = AuditLogId::new();
        
        let model = audit::ActiveModel {
            id: Set(id),
            user_id: Set(input.user_id),
            user_email: Set(input.user_email.clone()),
            action: Set(input.action.clone()),
            resource_type: Set(input.resource_type.clone()),
            resource_id: Set(input.resource_id),
            ip_address: Set(input.ip_address.clone()),
            user_agent: Set(input.user_agent.clone()),
            metadata: Set(input.metadata.clone().unwrap_or_default()),
            success: Set(input.success),
            error_message: Set(input.error_message.clone()),
            created_at: Set(now),
        };
        
        audit::Entity::insert(model).exec(&self.db).await?;
        
        Ok(AuditLogEntry {
            id: id.as_uuid(),
            user_id: input.user_id,
            user_email: input.user_email,
            action: input.action,
            resource_type: input.resource_type,
            resource_id: input.resource_id,
            ip_address: input.ip_address,
            user_agent: input.user_agent,
            metadata: input.metadata.unwrap_or_default(),
            success: input.success,
            error_message: input.error_message,
            created_at: now,
        })
    }

    // ========================================================================
    // Invitation Operations
    // ========================================================================

    pub async fn create_email_invitation(&self, input: CreateEmailInviteInput, invited_by: UserId) -> Result<FamilyInvitation, DbStoreError> {
        let now = Utc::now();
        let id = InvitationId::new();
        let expires_at = Some(now + Duration::days(input.expires_in_days));
        
        let model = invitation::ActiveModel {
            id: Set(id),
            tenant_id: Set(input.tenant_id),
            invited_by: Set(Some(invited_by)),
            invite_type: Set(invitation::InviteType::Email),
            email: Set(Some(input.email.clone())),
            invite_code: Set(None),
            role: Set(match input.role.unwrap_or(InviteRole::Member) {
                InviteRole::Admin => invitation::InviteRole::Admin,
                InviteRole::Member => invitation::InviteRole::Member,
                InviteRole::Guest => invitation::InviteRole::Guest,
            }),
            max_uses: Set(Some(1)),
            use_count: Set(0),
            expires_at: Set(expires_at),
            created_at: Set(now),
        };
        
        invitation::Entity::insert(model).exec(&self.db).await?;
        
        Ok(FamilyInvitation {
            id,
            tenant_id: input.tenant_id,
            invited_by: Some(invited_by),
            invite_type: InviteType::Email,
            email: Some(input.email),
            invite_code: None,
            role: input.role.unwrap_or(InviteRole::Member),
            max_uses: 1,
            use_count: 0,
            expires_at,
            created_at: now,
        })
    }

    pub async fn create_code_invitation(&self, input: CreateCodeInviteInput, invited_by: UserId) -> Result<FamilyInvitation, DbStoreError> {
        let now = Utc::now();
        let id = InvitationId::new();
        let code = generate_invite_code();
        let expires_at = input.expires_in_days.map(|days| now + Duration::days(days));
        
        let model = invitation::ActiveModel {
            id: Set(id),
            tenant_id: Set(input.tenant_id),
            invited_by: Set(Some(invited_by)),
            invite_type: Set(invitation::InviteType::Code),
            email: Set(None),
            invite_code: Set(Some(code.clone())),
            role: Set(match input.role.unwrap_or(InviteRole::Member) {
                InviteRole::Admin => invitation::InviteRole::Admin,
                InviteRole::Member => invitation::InviteRole::Member,
                InviteRole::Guest => invitation::InviteRole::Guest,
            }),
            max_uses: Set(Some(input.max_uses)),
            use_count: Set(0),
            expires_at: Set(expires_at),
            created_at: Set(now),
        };
        
        invitation::Entity::insert(model).exec(&self.db).await?;
        
        Ok(FamilyInvitation {
            id,
            tenant_id: input.tenant_id,
            invited_by: Some(invited_by),
            invite_type: InviteType::Code,
            email: None,
            invite_code: Some(code),
            role: input.role.unwrap_or(InviteRole::Member),
            max_uses: input.max_uses,
            use_count: 0,
            expires_at,
            created_at: now,
        })
    }

    pub async fn get_invitation_by_code(&self, code: &str) -> Result<Option<FamilyInvitation>, DbStoreError> {
        let result = invitation::Entity::find()
            .filter(invitation::Column::InviteCode.eq(code))
            .one(&self.db)
            .await?;
        
        Ok(result.map(|m| map_invitation(m)))
    }

    pub async fn get_invitation_by_email(&self, email: &str, tenant_id: TenantId) -> Result<Option<FamilyInvitation>, DbStoreError> {
        let result = invitation::Entity::find()
            .filter(invitation::Column::Email.eq(email))
            .filter(invitation::Column::TenantId.eq(tenant_id))
            .one(&self.db)
            .await?;
        
        Ok(result.map(|m| map_invitation(m)))
    }

    pub async fn use_invitation(&self, invitation_id: InvitationId) -> Result<(), DbStoreError> {
        invitation::Entity::update_many()
            .col_expr(invitation::Column::UseCount, Expr::col(invitation::Column::UseCount).add(1))
            .filter(invitation::Column::Id.eq(invitation_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn is_invitation_valid(&self, invitation_id: InvitationId) -> Result<bool, DbStoreError> {
        let now = Utc::now();
        let result = invitation::Entity::find_by_id(invitation_id).one(&self.db).await?;
        
        match result {
            None => Ok(false),
            Some(inv) => {
                if inv.expires_at.map(|e| e < now).unwrap_or(false) { return Ok(false); }
                if inv.use_count >= inv.max_uses.unwrap_or(1) { return Ok(false); }
                Ok(true)
            }
        }
    }

    // ========================================================================
    // Join Request Operations
    // ========================================================================

    pub async fn create_join_request(&self, user_id: UserId, input: CreateJoinRequestInput) -> Result<JoinRequest, DbStoreError> {
        let now = Utc::now();
        let id = JoinRequestId::new();
        
        let model = join_request::ActiveModel {
            id: Set(id),
            user_id: Set(user_id),
            tenant_id: Set(input.tenant_id),
            message: Set(input.message.clone()),
            status: Set(join_request::JoinRequestStatus::Pending),
            reviewed_by: Set(None),
            reviewed_at: Set(None),
            review_note: Set(None),
            created_at: Set(now),
        };
        
        join_request::Entity::insert(model).exec(&self.db).await?;
        
        Ok(JoinRequest {
            id: id.as_uuid(),
            user_id,
            tenant_id: input.tenant_id,
            message: input.message,
            status: JoinRequestStatus::Pending,
            reviewed_by: None,
            reviewed_at: None,
            review_note: None,
            created_at: now,
        })
    }

    pub async fn get_pending_join_requests(&self, tenant_id: TenantId) -> Result<Vec<JoinRequest>, DbStoreError> {
        let results: Vec<join_request::Model> = join_request::Entity::find()
            .filter(join_request::Column::TenantId.eq(tenant_id))
            .filter(join_request::Column::Status.eq(join_request::JoinRequestStatus::Pending))
            .order_by_desc(join_request::Column::CreatedAt)
            .all(&self.db)
            .await?;
        
        Ok(results.into_iter().map(|m| JoinRequest {
            id: m.id.as_uuid(),
            user_id: m.user_id,
            tenant_id: m.tenant_id,
            message: m.message,
            status: match m.status {
                join_request::JoinRequestStatus::Pending => JoinRequestStatus::Pending,
                join_request::JoinRequestStatus::Approved => JoinRequestStatus::Approved,
                join_request::JoinRequestStatus::Rejected => JoinRequestStatus::Rejected,
            },
            reviewed_by: m.reviewed_by,
            reviewed_at: m.reviewed_at,
            review_note: m.review_note,
            created_at: m.created_at,
        }).collect())
    }

    pub async fn review_join_request(&self, request_id: JoinRequestId, reviewer_id: UserId, input: ReviewJoinRequestInput) -> Result<JoinRequest, DbStoreError> {
        let now = Utc::now();
        
        join_request::Entity::update_many()
            .col_expr(join_request::Column::Status, Expr::value(match input.approved {
                true => "approved",
                false => "rejected",
            }))
            .col_expr(join_request::Column::ReviewedBy, Expr::value(Some(reviewer_id)))
            .col_expr(join_request::Column::ReviewedAt, Expr::value(Some(now)))
            .col_expr(join_request::Column::ReviewNote, Expr::value(input.note.clone()))
            .filter(join_request::Column::Id.eq(request_id))
            .exec(&self.db)
            .await?;
        
        let result = join_request::Entity::find_by_id(request_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbStoreError::not_found("JoinRequest", request_id.to_string()))?;
        
        Ok(JoinRequest {
            id: result.id.as_uuid(),
            user_id: result.user_id,
            tenant_id: result.tenant_id,
            message: result.message,
            status: match result.status {
                join_request::JoinRequestStatus::Pending => JoinRequestStatus::Pending,
                join_request::JoinRequestStatus::Approved => JoinRequestStatus::Approved,
                join_request::JoinRequestStatus::Rejected => JoinRequestStatus::Rejected,
            },
            reviewed_by: result.reviewed_by,
            reviewed_at: result.reviewed_at,
            review_note: result.review_note,
            created_at: result.created_at,
        })
    }

    // ========================================================================
    // Membership Operations
    // ========================================================================

    pub async fn add_user_to_tenant(&self, user_id: UserId, tenant_id: TenantId, role: InviteRole) -> Result<(), DbStoreError> {
        let now = Utc::now();
        let thread_id = ThreadId::new();
        
        let user = user::Entity::find_by_id(user_id).one(&self.db).await?
            .ok_or_else(|| DbStoreError::not_found("User", user_id.to_string()))?;
        
        let model = tenant_member::ActiveModel {
            id: Set(thread_id),
            user_id: Set(user_id),
            tenant_id: Set(tenant_id),
            name: Set(user.name),
            email: Set(Some(user.email)),
            avatar_url: Set(user.avatar_url),
            role: Set(match role {
                InviteRole::Admin => tenant_member::MemberRole::Admin,
                InviteRole::Member => tenant_member::MemberRole::Member,
                InviteRole::Guest => tenant_member::MemberRole::Guest,
            }),
            settings: Set(serde_json::json!({})),
            created_at: Set(now),
            updated_at: Set(now),
        };
        
        tenant_member::Entity::insert(model).exec(&self.db).await?;
        Ok(())
    }

    pub async fn get_user_memberships(&self, user_id: UserId) -> Result<Vec<UserMembership>, DbStoreError> {
        let user = user::Entity::find_by_id(user_id).one(&self.db).await?
            .ok_or_else(|| DbStoreError::not_found("User", user_id.to_string()))?;
        
        let results = tenant_member::Entity::find()
            .filter(tenant_member::Column::Email.eq(&user.email))
            .all(&self.db)
            .await?;
        
        let tenant_ids: Vec<TenantId> = results.iter().map(|m| m.tenant_id).collect();
        let tenants: std::collections::HashMap<TenantId, String> = tenant::Entity::find()
            .filter(tenant::Column::Id.is_in(tenant_ids))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|t| (t.id, t.name))
            .collect();
        
        Ok(results.into_iter().map(|m| UserMembership {
            tenant_id: m.tenant_id,
            tenant_name: tenants.get(&m.tenant_id).cloned().unwrap_or_default(),
            role: match m.role {
                tenant_member::MemberRole::Admin => InviteRole::Admin,
                tenant_member::MemberRole::Member => InviteRole::Member,
                tenant_member::MemberRole::Guest => InviteRole::Guest,
            },
            is_primary: user.primary_tenant_id == Some(m.tenant_id),
            joined_at: m.created_at,
        }).collect())
    }

    pub async fn is_user_member(&self, user_id: UserId, tenant_id: TenantId) -> Result<bool, DbStoreError> {
        let user = user::Entity::find_by_id(user_id).one(&self.db).await?;
        match user {
            None => Ok(false),
            Some(u) => {
                let count = tenant_member::Entity::find()
                    .filter(tenant_member::Column::TenantId.eq(tenant_id))
                    .filter(tenant_member::Column::Email.eq(&u.email))
                    .count(&self.db)
                    .await?;
                Ok(count > 0)
            }
        }
    }

    pub async fn is_user_admin(&self, user_id: UserId, tenant_id: TenantId) -> Result<bool, DbStoreError> {
        let user = user::Entity::find_by_id(user_id).one(&self.db).await?;
        match user {
            None => Ok(false),
            Some(u) => {
                let count = tenant_member::Entity::find()
                    .filter(tenant_member::Column::TenantId.eq(tenant_id))
                    .filter(tenant_member::Column::Email.eq(&u.email))
                    .filter(tenant_member::Column::Role.eq(tenant_member::MemberRole::Admin))
                    .count(&self.db)
                    .await?;
                Ok(count > 0)
            }
        }
    }
}

fn generate_invite_code() -> String {
    use rand::Rng;
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    let mut rng = rand::thread_rng();
    (0..8).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
}

fn map_invitation(m: invitation::Model) -> FamilyInvitation {
    FamilyInvitation {
        id: m.id,
        tenant_id: m.tenant_id,
        invited_by: m.invited_by,
        invite_type: match m.invite_type {
            invitation::InviteType::Email => InviteType::Email,
            invitation::InviteType::Code => InviteType::Code,
        },
        email: m.email,
        invite_code: m.invite_code,
        role: match m.role {
            invitation::InviteRole::Admin => InviteRole::Admin,
            invitation::InviteRole::Member => InviteRole::Member,
            invitation::InviteRole::Guest => InviteRole::Guest,
        },
        max_uses: m.max_uses.unwrap_or(1),
        use_count: m.use_count,
        expires_at: m.expires_at,
        created_at: m.created_at,
    }
}
