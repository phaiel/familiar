/**
 * Temporal Client
 * 
 * Example client for starting workflows from external services.
 * In production, familiar-api will use similar code.
 * 
 * Usage:
 *   npm run start:client
 *   # or with args:
 *   npm run start:client -- --content "Hello, world!"
 */

import { Client, Connection } from '@temporalio/client';
import { loomWorkflow } from './workflows';
import type { WeaveRequest } from './workflows';
import { randomUUID } from 'crypto';

async function main() {
  const temporalAddress = process.env.TEMPORAL_ADDRESS || 'localhost:7233';
  const namespace = process.env.TEMPORAL_NAMESPACE || 'default';
  const taskQueue = process.env.TEMPORAL_TASK_QUEUE || 'fates-pipeline';

  console.log(`Connecting to Temporal at ${temporalAddress}...`);

  const connection = await Connection.connect({ 
    address: temporalAddress 
  });

  const client = new Client({
    connection,
    namespace,
  });

  // Example weave request
  const request: WeaveRequest = {
    course_id: randomUUID(),
    shuttle_id: randomUUID(),
    content: process.argv[3] || 'Hello, Familiar! This is a test message.',
    sender_id: 'test-user',
    channel_id: 'test-channel',
    tenant_id: 'test-tenant',
  };

  console.log('Starting loomWorkflow with request:', request);

  const handle = await client.workflow.start(loomWorkflow, {
    taskQueue,
    workflowId: `loom-${request.shuttle_id}`,
    args: [request],
  });

  console.log(`Started workflow: ${handle.workflowId}`);
  console.log(`Run ID: ${handle.firstExecutionRunId}`);
  console.log('Waiting for result...');

  const result = await handle.result();
  
  console.log('Workflow completed!');
  console.log('Result:', JSON.stringify(result, null, 2));

  await connection.close();
}

main().catch((err) => {
  console.error('Client failed:', err);
  process.exit(1);
});

