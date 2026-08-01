-- Migration to add total_trays, dirty_cat1, dirty_cat2, production_percentage, stock_in_trays and update egg_count_damaged
ALTER TABLE daily_production 
    ADD COLUMN IF NOT EXISTS total_trays DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN IF NOT EXISTS dirty_cat1 DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN IF NOT EXISTS dirty_cat2 DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN IF NOT EXISTS production_percentage DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN IF NOT EXISTS stock_in_trays DOUBLE PRECISION NOT NULL DEFAULT 0.0;

ALTER TABLE daily_production 
    ALTER COLUMN egg_count_damaged TYPE DOUBLE PRECISION USING egg_count_damaged::DOUBLE PRECISION;
