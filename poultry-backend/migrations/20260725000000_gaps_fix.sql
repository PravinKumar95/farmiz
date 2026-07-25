-- Add user_id for multi-tenancy to existing tables
ALTER TABLE parties ADD COLUMN IF NOT EXISTS user_id VARCHAR(255);
ALTER TABLE employees ADD COLUMN IF NOT EXISTS user_id VARCHAR(255);

ALTER TABLE egg_sales 
    ADD COLUMN IF NOT EXISTS user_id VARCHAR(255),
    ADD COLUMN IF NOT EXISTS party_id UUID REFERENCES parties(id) ON DELETE SET NULL;

ALTER TABLE broken_egg_sales 
    ADD COLUMN IF NOT EXISTS user_id VARCHAR(255),
    ADD COLUMN IF NOT EXISTS party_id UUID REFERENCES parties(id) ON DELETE SET NULL;

ALTER TABLE material_purchases 
    ADD COLUMN IF NOT EXISTS user_id VARCHAR(255),
    ADD COLUMN IF NOT EXISTS party_id UUID REFERENCES parties(id) ON DELETE SET NULL;

ALTER TABLE feed_batches 
    ADD COLUMN IF NOT EXISTS user_id VARCHAR(255);

ALTER TABLE labor_records 
    ADD COLUMN IF NOT EXISTS user_id VARCHAR(255),
    ADD COLUMN IF NOT EXISTS employee_id UUID REFERENCES employees(id) ON DELETE SET NULL;

-- Create Daily Production & Flock Health Log table
CREATE TABLE IF NOT EXISTS daily_production (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date VARCHAR(50) NOT NULL,
    shed_name VARCHAR(100) NOT NULL,
    egg_count_good INTEGER NOT NULL DEFAULT 0,
    egg_count_damaged INTEGER NOT NULL DEFAULT 0,
    mortality_count INTEGER NOT NULL DEFAULT 0,
    cull_count INTEGER NOT NULL DEFAULT 0,
    feed_consumed_kg DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    notes TEXT,
    user_id VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
