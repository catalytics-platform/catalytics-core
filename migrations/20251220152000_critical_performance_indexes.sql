-- Add migration script here

-- 1. Critical: Covering index for beta_applicant_badges to optimize leaderboard view JOINs
-- This eliminates table lookups during the leaderboard view calculation
CREATE INDEX IF NOT EXISTS idx_beta_applicant_badges_covering
ON beta_applicant_badges (beta_applicant_id, badge_id)
INCLUDE (created_at);

-- 2. Critical: Index for badge scores to optimize SUM(b.score) in leaderboard view
CREATE INDEX IF NOT EXISTS idx_badges_score
ON badges (score);

-- 3. Critical: Composite index for badge condition lookups during badge awarding
-- Optimizes the WHERE clause in award_badge_if_eligible queries
CREATE INDEX IF NOT EXISTS idx_badge_conditions_eligibility
ON badge_conditions (progression_event_type_id, operation, required_count);

-- 4. Important: Composite index for progression lookups
-- Optimizes user progression queries in beta_applicant_progression
CREATE INDEX IF NOT EXISTS idx_beta_applicant_progressions_lookup
ON beta_applicant_progressions (beta_applicant_id, progression_event_type_id);

-- Comments explaining the performance impact
COMMENT ON INDEX idx_beta_applicant_badges_covering IS 'Covering index for leaderboard view JOINs - eliminates table lookups';
COMMENT ON INDEX idx_badges_score IS 'Optimizes SUM(score) aggregations in leaderboard calculations';
COMMENT ON INDEX idx_badge_conditions_eligibility IS 'Optimizes badge eligibility checks during badge awarding';
COMMENT ON INDEX idx_beta_applicant_progressions_lookup IS 'Optimizes user progression lookups for badge and leaderboard features';