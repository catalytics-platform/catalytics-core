-- Add migration script here
ALTER TABLE beta_applicants
ADD COLUMN IF NOT EXISTS referral_code TEXT UNIQUE NOT NULL DEFAULT '',
ADD COLUMN IF NOT EXISTS referred_by_id INTEGER REFERENCES beta_applicants(id);

-- Index for fast look-ups when someone uses a referral link
CREATE UNIQUE INDEX IF NOT EXISTS idx_beta_applicants_referral_code
    ON beta_applicants(referral_code);

-- Index for showing "who I referred" quickly
CREATE INDEX IF NOT EXISTS idx_beta_applicants_referred_by
    ON beta_applicants(referred_by_id);