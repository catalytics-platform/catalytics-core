-- Add migration script here
CREATE INDEX IF NOT EXISTS idx_badge_group_conjunctions_badge_id
    ON badge_group_conjunctions (badge_id);