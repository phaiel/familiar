-- TigerData Initialization Script
-- 
-- This script runs automatically when the TimescaleDB container starts for the first time.
-- It creates the required extensions before migrations run.
--
-- Extensions:
-- - uuid-ossp: UUID generation (uuid_generate_v4)
-- - pgvector: Vector similarity search (backup to Qdrant)
-- - timescaledb: Time-series hypertables (already included in image)

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable vector operations (for embeddings)
CREATE EXTENSION IF NOT EXISTS vector;

-- TimescaleDB is already enabled in the timescale/timescaledb-ha image
-- but we ensure it's available
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

-- Log successful initialization
DO $$
BEGIN
    RAISE NOTICE 'TigerData initialized with extensions: uuid-ossp, vector, timescaledb';
END $$;

