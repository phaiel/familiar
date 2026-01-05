//! Component Traits
//!
//! Defines the trait system for the hybrid entity-component architecture.
//! Entities are composed of components and implement these traits to
//! provide uniform access to their component data.
//!
//! ## Trait Hierarchy
//!
//! ```text
//! Component (marker trait)
//!   ├── HasIdentity - entities with unique identity
//!   ├── HasPhysics - entities with physics state (VAE position)
//!   ├── HasContent - entities with content payload
//!   ├── HasTemporal - entities with temporal binding
//!   └── HasClassifications - entities with weighted classifications
//! ```

use super::identity::Identity;
use super::content::ContentPayload;
use super::field_excitation::FieldExcitation;
use super::quantum_state::QuantumState;
use super::weighted_classification::WeightedClassification;
use crate::primitives::UUID;

/// Marker trait for all components.
/// 
/// Components are data-only structs that can be composed into entities.
/// They must be Send + Sync for parallel simulation.
pub trait Component: Sized + Send + Sync + 'static {}

/// Entities with a unique identity.
/// 
/// All persistent entities must have an identity for:
/// - Database storage
/// - Cross-reference between entities
pub trait HasIdentity {
    /// Get the entity's identity
    fn identity(&self) -> &Identity;
    
    /// Get the entity's UUID (convenience method)
    fn id(&self) -> UUID {
        self.identity().id
    }
    
    /// Get the tenant ID
    fn tenant_id(&self) -> UUID {
        self.identity().tenant_id
    }
}

/// Entities with physics state in VAE space.
/// 
/// Physics-enabled entities participate in the simulation:
/// - Position in 3D VAE space (valence, arousal, epistemic)
/// - Forces and energy
/// - Temperature (volatility)
pub trait HasPhysics {
    /// Get immutable reference to physics state
    fn physics(&self) -> &FieldExcitation;
    
    /// Get mutable reference to physics state
    fn physics_mut(&mut self) -> &mut FieldExcitation;
    
    /// Get the entity's position as quantized coordinates
    fn position(&self) -> [i64; 3] {
        let p = self.physics();
        [p.position[0].value(), p.position[1].value(), p.position[2].value()]
    }
    
    /// Get the entity's significance (amplitude/mass)
    fn significance(&self) -> f64 {
        self.physics().amplitude.value()
    }
}

/// Entities with quantum state (superposition, coherence).
/// 
/// Used for entities that can exist in multiple states:
/// - Classification probabilities
/// - Observation collapse
pub trait HasQuantum {
    /// Get immutable reference to quantum state
    fn quantum(&self) -> &QuantumState;
    
    /// Get mutable reference to quantum state
    fn quantum_mut(&mut self) -> &mut QuantumState;
    
    /// Get the coherence level (0.0 = decoherent, 1.0 = pure state)
    fn coherence(&self) -> f64 {
        self.quantum().coherence.value()
    }
}

/// Entities with content payload.
/// 
/// Content is the actual data the entity represents:
/// - Text content (user messages, thoughts)
/// - Metadata
pub trait HasContent {
    /// Get immutable reference to content
    fn content(&self) -> &ContentPayload;
    
    /// Get mutable reference to content
    fn content_mut(&mut self) -> &mut ContentPayload;
    
    /// Get the text content
    fn text(&self) -> &str {
        &self.content().text
    }
}

/// Entities with temporal binding.
/// 
/// Temporal binding captures when events occurred:
/// - Absolute timestamps
/// - Relative markers ("yesterday", "last week")
pub trait HasTemporal {
    /// Get the timestamp when this entity was observed
    fn observed_at(&self) -> chrono::DateTime<chrono::Utc>;
    
    /// Get the temporal marker (if any)
    fn temporal_marker(&self) -> Option<&str>;
    
    /// Check if this is a recent entity (within last hour)
    fn is_recent(&self) -> bool {
        let age = chrono::Utc::now() - self.observed_at();
        age.num_hours() < 1
    }
}

/// Entities with weighted classifications.
/// 
/// Classifications determine what type of entity this is:
/// - Entity type probabilities (Moment, Thread, Bond, etc.)
/// - Intent classification
pub trait HasClassifications {
    /// Get the classifications
    fn classifications(&self) -> &[WeightedClassification];
    
    /// Get the primary (highest weight) classification
    fn primary_classification(&self) -> Option<&WeightedClassification> {
        self.classifications()
            .iter()
            .max_by(|a, b| a.weight.value().partial_cmp(&b.weight.value()).unwrap_or(std::cmp::Ordering::Equal))
    }
}

// Implement Component marker for all component types
impl Component for Identity {}
impl Component for ContentPayload {}
impl Component for FieldExcitation {}
impl Component for QuantumState {}
impl Component for WeightedClassification {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::UUID;

    // Example entity for testing trait implementation
    struct TestMoment {
        identity: Identity,
        content: ContentPayload,
    }

    impl HasIdentity for TestMoment {
        fn identity(&self) -> &Identity {
            &self.identity
        }
    }

    impl HasContent for TestMoment {
        fn content(&self) -> &ContentPayload {
            &self.content
        }
        
        fn content_mut(&mut self) -> &mut ContentPayload {
            &mut self.content
        }
    }

    #[test]
    fn test_has_identity() {
        let moment = TestMoment {
            identity: Identity::new(UUID::new()),
            content: ContentPayload {
                text: "test".to_string(),
                metadata: Default::default(),
            },
        };
        
        // Verify trait methods work
        let _id = moment.id();
        let _tenant = moment.tenant_id();
    }
}
