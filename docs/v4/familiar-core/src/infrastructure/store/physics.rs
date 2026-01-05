//! Physics domain data access (Pulse, Moment, Components)
//!
//! Uses SeaORM ActiveModel for type-safe persistence. If a field is renamed
//! in the Rust struct or schema, the compiler will catch mismatches here.
//!
//! Note: Quantum state uses raw SQL for pgvector compatibility (SeaORM
//! doesn't natively support the VECTOR type).

use sea_orm::{ActiveModelTrait, ConnectionTrait, Set, Statement, DbBackend, TransactionTrait};
use uuid::Uuid;
use chrono::Utc;

use crate::infrastructure::store::TigerDataStore;
use crate::internal::DbStoreError;
use crate::legacy::entities::{Pulse, Moment};
use crate::primitives::{UUID, EntityId, TenantId};

// SeaORM entities for type-safe persistence
use crate::entities::db::physics::{
    entity_registry, field_excitation, content,
};

impl TigerDataStore {
    // ========================================================================
    // Entity Save Operations (Pulse/Moment)
    // ========================================================================

    pub async fn save_pulse(&self, pulse: &Pulse) -> Result<(), DbStoreError> {
        let txn = self.db.begin().await?;
        let now = Utc::now();
        // Convert UUID wrapper to typed primitives for SeaORM entities
        let entity_id = EntityId::from(pulse.identity.id.as_uuid());
        let tenant_id = TenantId::from(pulse.identity.tenant_id.as_uuid());

        // 1. Entity Registry (SeaORM ActiveModel)
        let entity_model = entity_registry::ActiveModel {
            id: Set(entity_id),
            tenant_id: Set(tenant_id),
            entity_type: Set("Pulse".to_string()),
            created_at: Set(now),
        };
        entity_model.insert(&txn).await?;

        // 2. Physics (Field Excitation) - SeaORM ActiveModel
        // Types align directly: QuantizedCoord::value() -> i64, NormalizedFloat::value() -> f64
        let p = &pulse.physics;
        let excitation_model = field_excitation::ActiveModel {
            time: Set(now),
            entity_id: Set(entity_id),
            pos_valence: Set(p.position[0].value()),
            pos_arousal: Set(p.position[1].value()),
            pos_epistemic: Set(p.position[2].value()),
            vel_valence: Set(p.velocity[0].value()),
            vel_arousal: Set(p.velocity[1].value()),
            vel_epistemic: Set(p.velocity[2].value()),
            amplitude: Set(p.amplitude.value()),
            energy: Set(p.energy.value()),
            temperature: Set(p.temperature.value()),
        };
        excitation_model.insert(&txn).await?;

        // 3. Content (SeaORM ActiveModel)
        let content_model = content::ActiveModel {
            entity_id: Set(entity_id),
            text_content: Set(pulse.content.text.clone()),
            metadata: Set(serde_json::to_value(&pulse.content.metadata).unwrap_or_default()),
        };
        content_model.insert(&txn).await?;

        txn.commit().await?;
        Ok(())
    }

    pub async fn save_moment(&self, moment: &Moment) -> Result<(), DbStoreError> {
        let txn = self.db.begin().await?;
        let now = Utc::now();
        // Convert UUID wrapper to typed primitives for SeaORM entities
        let entity_id = EntityId::from(moment.identity.id.as_uuid());
        let tenant_id = TenantId::from(moment.identity.tenant_id.as_uuid());

        // 1. Entity Registry (SeaORM ActiveModel)
        let entity_model = entity_registry::ActiveModel {
            id: Set(entity_id),
            tenant_id: Set(tenant_id),
            entity_type: Set("Moment".to_string()),
            created_at: Set(now),
        };
        entity_model.insert(&txn).await?;

        // 2. Physics (Field Excitation) - SeaORM ActiveModel
        // Types align directly: QuantizedCoord::value() -> i64, NormalizedFloat::value() -> f64
        let p = &moment.physics;
        let excitation_model = field_excitation::ActiveModel {
            time: Set(now),
            entity_id: Set(entity_id),
            pos_valence: Set(p.position[0].value()),
            pos_arousal: Set(p.position[1].value()),
            pos_epistemic: Set(p.position[2].value()),
            vel_valence: Set(p.velocity[0].value()),
            vel_arousal: Set(p.velocity[1].value()),
            vel_epistemic: Set(p.velocity[2].value()),
            amplitude: Set(p.amplitude.value()),
            energy: Set(p.energy.value()),
            temperature: Set(p.temperature.value()),
        };
        excitation_model.insert(&txn).await?;

        // 3. Quantum State (raw SQL for pgvector - SeaORM doesn't support VECTOR type)
        let amplitudes: Vec<f32> = moment.quantum.amplitudes.iter()
            .flat_map(|(re, im)| vec![*re as f32, *im as f32])
            .collect();
        let vector_str = format!("[{}]", amplitudes.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(","));
        
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO comp_quantum_states (entity_id, amplitudes, coherence, frequency) VALUES ($1, $2::vector, $3, $4)",
            vec![
                entity_id.as_uuid().into(), 
                vector_str.into(), 
                moment.quantum.coherence.value().into(),
                moment.quantum.frequency.into()
            ],
        )).await?;

        // 4. Content (SeaORM ActiveModel)
        let content_model = content::ActiveModel {
            entity_id: Set(entity_id),
            text_content: Set(moment.content.text.clone()),
            metadata: Set(serde_json::to_value(&moment.content.metadata).unwrap_or_default()),
        };
        content_model.insert(&txn).await?;

        txn.commit().await?;
        Ok(())
    }

    pub async fn load_pulse(&self, _entity_id: &UUID) -> Result<Option<Pulse>, DbStoreError> {
        // Implementation for loading complex entities would go here
        // For now, return None as a placeholder (requires mapping logic)
        Ok(None)
    }

    /// Find entities similar to a query vector using pgvector cosine distance
    /// 
    /// Note: This uses raw SQL because SeaORM doesn't support pgvector natively.
    pub async fn find_similar_by_vector(
        &self,
        query_vector: Vec<f32>,
        limit: i64,
    ) -> Result<Vec<(Uuid, f64)>, DbStoreError> {
        let vector_str = format!("[{}]", query_vector.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(","));
        
        let sql = r#"
            SELECT entity_id, 1 - (amplitudes <=> $1::vector) as similarity
            FROM comp_quantum_states
            ORDER BY amplitudes <=> $1::vector
            LIMIT $2
        "#;

        let rows = self.db.query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![vector_str.into(), limit.into()],
        )).await?;
        
        Ok(rows.into_iter().map(|r| {
            let id: Uuid = r.try_get("", "entity_id").unwrap_or_default();
            let sim: f64 = r.try_get("", "similarity").unwrap_or_default();
            (id, sim)
        }).collect())
    }
}

