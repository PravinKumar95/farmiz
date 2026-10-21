-- Migration to add flock_size to daily_production
ALTER TABLE daily_production 
    ADD COLUMN IF NOT EXISTS flock_size INTEGER NOT NULL DEFAULT 0;
