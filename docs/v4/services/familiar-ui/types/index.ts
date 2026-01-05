/**
 * Generated Types - Single Source of Truth
 * 
 * Re-exports types from familiar-core bindings (ts-rs generated from Rust schemas).
 * 
 * Generated types are in:
 *   familiar-core/generated/typescript/
 * 
 * To regenerate: run `cargo test --lib` in familiar-core to update ts-rs bindings.
 */

// ============================================================================
// UI Component Types (from generated)
// ============================================================================
export type { ToolCallStatus } from "@familiar-core/ToolCallStatus"
export type { UIToolCall } from "@familiar-core/UIToolCall"
export type { UIThinkingStep } from "@familiar-core/UIThinkingStep"
export type { UIHeddleSegment } from "@familiar-core/UIHeddleSegment"
export type { UIClassification } from "@familiar-core/UIClassification"
export type { UIPhysicsResult } from "@familiar-core/UIPhysicsResult"
export type { UIHeddleResult } from "@familiar-core/UIHeddleResult"
export type { UIThreadItem } from "@familiar-core/UIThreadItem"
export type { UIChannelMessage } from "@familiar-core/UIChannelMessage"
export type { UIChannel } from "@familiar-core/UIChannel"

// ============================================================================
// Agentic Flow Types (from generated)
// ============================================================================
export type { AgenticFlowResponse } from "@familiar-core/AgenticFlowResponse"
export type { ConversationHistoryItem } from "@familiar-core/ConversationHistoryItem"

// ============================================================================
// Block Kit Types (from generated)
// ============================================================================
export type { Block } from "@familiar-core/Block"
export type { BlockMessage } from "@familiar-core/BlockMessage"
export type { TextObject } from "@familiar-core/TextObject"
export type { PlainTextObject } from "@familiar-core/PlainTextObject"
export type { MarkdownTextObject } from "@familiar-core/MarkdownTextObject"
export type { SectionBlock } from "@familiar-core/SectionBlock"
export type { HeaderBlock } from "@familiar-core/HeaderBlock"
export type { ImageBlock } from "@familiar-core/ImageBlock"
export type { DividerBlock } from "@familiar-core/DividerBlock"
export type { ActionsBlock } from "@familiar-core/ActionsBlock"
export type { ContextBlock } from "@familiar-core/ContextBlock"
export type { AccordionBlock } from "@familiar-core/AccordionBlock"
export type { AccordionState } from "@familiar-core/AccordionState"
export type { BlockElement } from "@familiar-core/BlockElement"
export type { ContextElement } from "@familiar-core/ContextElement"
export type { ImageElement } from "@familiar-core/ImageElement"

// ============================================================================
// Message Types (from generated)
// ============================================================================
export type { MessageIntent } from "@familiar-core/MessageIntent"
export type { MessageRole } from "@familiar-core/MessageRole"

// ============================================================================
// Conversation Types (from generated)
// ============================================================================
export type { ConversationTurn } from "@familiar-core/ConversationTurn"

// ============================================================================
// Entity Types (from generated)
// ============================================================================
export type { HeddleEntityType } from "@familiar-core/HeddleEntityType"

// ============================================================================
// Classification and Physics (from generated)
// ============================================================================
export type { WeaveUnitClassification } from "@familiar-core/WeaveUnitClassification"
export type { PhysicsHint } from "@familiar-core/PhysicsHint"
export type { NormalizedFloat } from "@familiar-core/NormalizedFloat"

// ============================================================================
// Weave Types (from generated)
// ============================================================================
export type { WeaveUnit } from "@familiar-core/WeaveUnit"
export type { WeaveRequest } from "@familiar-core/WeaveRequest"

// ============================================================================
// Auth Types (from generated)
// ============================================================================
export type { User } from "@familiar-core/User"
export type { UserMembership } from "@familiar-core/UserMembership"
export type { SessionCreated } from "@familiar-core/SessionCreated"
export type { AuthResponse } from "@familiar-core/AuthResponse"
export type { CurrentUser } from "@familiar-core/CurrentUser"
export type { InvitationInfo } from "@familiar-core/InvitationInfo"

// Type aliases for backwards compatibility (re-exports with rename)
export { type SessionCreated as Session } from "@familiar-core/SessionCreated"
export { type CurrentUser as CurrentUserResponse } from "@familiar-core/CurrentUser"

// ============================================================================
// Tenant Types (from generated)
// ============================================================================
export type { Tenant } from "@familiar-core/Tenant"
export type { TenantMember } from "@familiar-core/TenantMember"
export type { MemberRole } from "@familiar-core/MemberRole"

// ============================================================================
// Channel Types (from generated)
// ============================================================================
export type { Channel } from "@familiar-core/Channel"
export type { ChannelType } from "@familiar-core/ChannelType"

// ============================================================================
// Message Types (from generated)
// ============================================================================
export type { Message } from "@familiar-core/Message"
