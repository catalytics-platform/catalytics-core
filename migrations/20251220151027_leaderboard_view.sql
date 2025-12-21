-- Add migration script here
CREATE VIEW beta_applicants_leaderboard AS
SELECT 
    ba.id,
    ba.public_key,
    ba.created_at,
    COALESCE(SUM(b.score), 0)::INTEGER as total_score
FROM beta_applicants ba
LEFT JOIN beta_applicant_badges bab ON ba.id = bab.beta_applicant_id  
LEFT JOIN badges b ON bab.badge_id = b.id
GROUP BY ba.id, ba.public_key, ba.created_at
ORDER BY total_score DESC, ba.created_at;
