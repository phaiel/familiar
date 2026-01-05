//! Impl module for payload types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for Payload

// Methods: type_name, is_command, is_event, is_trace
impl Payload { # [doc = " Get the message type name for routing"] pub fn type_name (& self) -> & 'static str { match self { Payload :: CourseStart { .. } => "course.command.start" , Payload :: CourseContinue { .. } => "course.command.continue" , Payload :: CourseCancel { .. } => "course.command.cancel" , Payload :: CourseRetry { .. } => "course.command.retry" , Payload :: CourseStarted { .. } => "course.event.started" , Payload :: CourseSegmented { .. } => "course.event.segmented" , Payload :: CourseClassified { .. } => "course.event.classified" , Payload :: CourseCompleted { .. } => "course.event.completed" , Payload :: CourseFailed { .. } => "course.event.failed" , Payload :: CourseCancelled { .. } => "course.event.cancelled" , Payload :: CourseRetrying { .. } => "course.event.retrying" , Payload :: Signup { .. } => "onboarding.command.signup" , Payload :: CreateFamily { .. } => "onboarding.command.create_family" , Payload :: AcceptInvitation { .. } => "onboarding.command.accept_invitation" , Payload :: SignupCompleted { .. } => "onboarding.event.signup_completed" , Payload :: FamilyCreated { .. } => "onboarding.event.family_created" , Payload :: InvitationAccepted { .. } => "onboarding.event.invitation_accepted" , Payload :: OnboardingFailed { .. } => "onboarding.event.failed" , Payload :: Trace { .. } => "trace" , } } # [doc = " Check if this is a command payload"] pub fn is_command (& self) -> bool { matches ! (self , Payload :: CourseStart { .. } | Payload :: CourseContinue { .. } | Payload :: CourseCancel { .. } | Payload :: CourseRetry { .. } | Payload :: Signup { .. } | Payload :: CreateFamily { .. } | Payload :: AcceptInvitation { .. }) } # [doc = " Check if this is an event payload"] pub fn is_event (& self) -> bool { matches ! (self , Payload :: CourseStarted { .. } | Payload :: CourseSegmented { .. } | Payload :: CourseClassified { .. } | Payload :: CourseCompleted { .. } | Payload :: CourseFailed { .. } | Payload :: CourseCancelled { .. } | Payload :: CourseRetrying { .. } | Payload :: SignupCompleted { .. } | Payload :: FamilyCreated { .. } | Payload :: InvitationAccepted { .. } | Payload :: OnboardingFailed { .. }) } # [doc = " Check if this is a trace payload"] pub fn is_trace (& self) -> bool { matches ! (self , Payload :: Trace { .. }) } }

