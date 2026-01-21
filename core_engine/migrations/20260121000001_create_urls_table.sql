-- GENESIS URL Shortener - Initial Schema
-- Creates the urls table for storing shortened URL mappings

CREATE TABLE IF NOT EXISTS urls (
    id VARCHAR(20) PRIMARY KEY,
    original_url TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_urls_created_at ON urls(created_at DESC);

-- Comment for documentation
COMMENT ON TABLE urls IS 'Stores URL shortener mappings: short_code -> original_url';
