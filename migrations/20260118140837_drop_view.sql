-- Add migration script here
-- Remove unused beta_applicants_leaderboard view
-- This view was used during the migration to leaderboard_entries table
-- but is no longer needed since the application now uses:
-- 1. leaderboard_entries table for fast queries
-- 2. Direct table JOINs for real-time calculations

DROP VIEW IF EXISTS beta_applicants_leaderboard;
