/**
 * Fates Pipeline Types
 * 
 * These types match the Rust structs in familiar-core.
 * Use snake_case to match Rust serde serialization.
 * 
 * TODO: Auto-generate from ts-rs bindings via `cargo xtask sync-temporal`
 */

/** Input to the Fates pipeline */
export interface FatesInput {
  course_id: string;
  shuttle_id: string;
  content: string;
  sender_id?: string;
  channel_id?: string;
  tenant_id?: string;
}

/** Output from Gate stage - classification and routing */
export interface GateOutput {
  classification: string;
  next_stage: string;
  input: FatesInput;
  confidence?: number;
}

/** Output from Morta stage - content segmentation */
export interface MortaOutput {
  segments: ContentSegment[];
  gate_output: GateOutput;
}

export interface ContentSegment {
  segment_type: string;
  content: string;
  start_pos: number;
  end_pos: number;
}

/** Output from Decima stage - entity extraction */
export interface DecimaOutput {
  entities: ExtractedEntity[];
  morta_output: MortaOutput;
}

export interface ExtractedEntity {
  entity_type: string;
  value: string;
  confidence: number;
  segment_index: number;
}

/** Output from Nona stage - response generation */
export interface NonaOutput {
  response: string;
  intent: string;
  payload?: Record<string, unknown>;
  decima_output: DecimaOutput;
}

/** Complete pipeline output */
export interface PipelineOutput {
  status: string;
  stages: {
    gate: GateOutput;
    morta: MortaOutput;
    decima: DecimaOutput;
    nona: NonaOutput;
  };
}

/** Activity interfaces - must match Rust activity names exactly */
export interface FatesActivities {
  FatesGate(input: FatesInput): Promise<GateOutput>;
  FatesMorta(input: GateOutput): Promise<MortaOutput>;
  FatesDecima(input: MortaOutput): Promise<DecimaOutput>;
  FatesNona(input: DecimaOutput): Promise<NonaOutput>;
  FatesPipeline(input: FatesInput): Promise<PipelineOutput>;
}





