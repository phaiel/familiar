/**
 * Loom Workflow - The Fates Pipeline Orchestrator
 * 
 * This workflow orchestrates the Fates stages (Gate, Morta, Decima, Nona)
 * running as Rust activities in familiar-daemon.
 * 
 * Stage-by-Stage Benefits:
 * 1. Visual Progress - See each stage in Temporal UI
 * 2. Granular Retries - If Decima fails, only Decima retries
 * 3. Interleave Logic - Insert approval steps between Rust activities
 */

import { proxyActivities } from '@temporalio/workflow';
import type { 
  FatesActivities, 
  FatesInput, 
  GateOutput, 
  MortaOutput, 
  DecimaOutput, 
  NonaOutput,
  PipelineOutput 
} from '../types';

// Proxy to Rust activities running in familiar-daemon
const fates = proxyActivities<FatesActivities>({
  taskQueue: 'fates-pipeline',
  startToCloseTimeout: '30 seconds',
  retry: {
    maximumAttempts: 3,
    initialInterval: '1s',
    backoffCoefficient: 2,
  },
});

/** Weave request - input to start the loom workflow */
export interface WeaveRequest {
  course_id: string;
  shuttle_id: string;
  content: string;
  sender_id?: string;
  channel_id?: string;
  tenant_id?: string;
}

/** Course response - output from the loom workflow */
export interface CourseResponse {
  status: string;
  stages: {
    gate: GateOutput;
    morta: MortaOutput;
    decima: DecimaOutput;
    nona: NonaOutput;
  };
}

/**
 * Main Loom Workflow
 * 
 * Orchestrates the Fates pipeline stage-by-stage.
 * Each stage is a separate activity call to the hot Rust worker.
 * 
 * @param request - The weave request containing user message
 * @returns Complete pipeline output with all stage results
 */
export async function loomWorkflow(request: WeaveRequest): Promise<CourseResponse> {
  // Stage 1: Gate - Classification and Routing
  // Determines which path the message should take
  const gate = await fates.FatesGate(request as FatesInput);
  
  // Stage 2: Morta - Content Segmentation
  // Breaks content into meaningful segments
  const morta = await fates.FatesMorta(gate);
  
  // Stage 3: Decima - Entity Extraction
  // Extracts entities, intents, and metadata
  // If this fails (e.g., LLM timeout), Temporal retries ONLY this stage
  const decima = await fates.FatesDecima(morta);
  
  // Stage 4: Nona - Response Generation
  // Generates the final response
  const nona = await fates.FatesNona(decima);
  
  return {
    status: 'pipeline_complete',
    stages: { gate, morta, decima, nona }
  };
}

/**
 * Fast Loom Workflow (Single Activity)
 * 
 * For cases where you want the entire pipeline as one atomic unit.
 * Less visibility but simpler retry semantics.
 */
export async function loomWorkflowFast(request: WeaveRequest): Promise<PipelineOutput> {
  return await fates.FatesPipeline(request as FatesInput);
}





