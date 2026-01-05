/**
 * Temporal Workflow Worker
 * 
 * This worker runs TypeScript workflows (the "brain").
 * Activities run on the Rust worker (familiar-daemon).
 * 
 * Usage:
 *   npm run start:worker
 *   # or with env vars:
 *   TEMPORAL_ADDRESS=localhost:7233 npm run start:worker
 */

import { Worker, NativeConnection } from '@temporalio/worker';

async function main() {
  const temporalAddress = process.env.TEMPORAL_ADDRESS || 'localhost:7233';
  const namespace = process.env.TEMPORAL_NAMESPACE || 'default';
  const taskQueue = process.env.TEMPORAL_TASK_QUEUE || 'fates-pipeline';

  console.log(`Connecting to Temporal at ${temporalAddress}...`);
  
  const connection = await NativeConnection.connect({ 
    address: temporalAddress 
  });

  console.log(`Creating worker for namespace "${namespace}", queue "${taskQueue}"...`);

  const worker = await Worker.create({
    connection,
    namespace,
    taskQueue,
    workflowsPath: require.resolve('./workflows'),
    // No activities registered here - they run on the Rust worker (familiar-daemon)
  });

  console.log('🧠 TypeScript Workflow Worker online');
  console.log(`   Namespace: ${namespace}`);
  console.log(`   Task Queue: ${taskQueue}`);
  console.log('   Workflows: loomWorkflow, loomWorkflowFast');
  console.log('');
  console.log('Waiting for Rust activity worker (familiar-daemon) to register activities...');

  await worker.run();
}

main().catch((err) => {
  console.error('Worker failed:', err);
  process.exit(1);
});





