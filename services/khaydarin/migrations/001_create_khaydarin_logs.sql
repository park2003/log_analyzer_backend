-- Migration for Khaydarin service logs table
-- As specified in DESIGN.md

CREATE TABLE IF NOT EXISTS khaydarin_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id VARCHAR(255) UNIQUE NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_prompt TEXT NOT NULL,
    llm_chain_trace JSONB,
    structured_plan JSONB,
    status VARCHAR(50) NOT NULL, -- 'SUCCESS', 'LLM_ERROR', 'PARSING_ERROR'
    processing_time_ms INT
);

CREATE INDEX idx_khaydarin_logs_user_id ON khaydarin_logs(user_id);
CREATE INDEX idx_khaydarin_logs_received_at ON khaydarin_logs(received_at);