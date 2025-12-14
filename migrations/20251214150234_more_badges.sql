-- Add migration script here
DELETE FROM badge_group_conjunctions
WHERE badge_id IN (2, 3, 4, 5, 6, 7);

INSERT INTO badge_groups (id, title, description) VALUES
    (2, 'CATICS Holder Badges', 'Rewards for holding and supporting with $CATICS'),
    (3, 'Jupiter Staker Badges', 'Cosmic rewards for staking $JUP'),
    (4, 'Referral Badges', 'Grow the clowder – earn badges for bringing in new cats'),
    (5, 'Season 0 Badges', 'Coming soon – exclusive rewards for the earliest miners and players');

INSERT INTO badges (id, title, description, score) VALUES
    (8, 'Bronze Early Supporter', 'Hold 100 $CATICS or more', 1),

    (9, 'Curious Feline', 'Stake at least 10 $JUP', 1),
    (10, 'Galactic Cat', 'Stake at least 1.000 $JUP', 1),
    (11, 'Cosmic Feline', 'Stake at least 10.000 $JUP', 1),

    (12, 'First Recruiter', 'Successfully refer 1 friend', 1),
    (13, 'Pack Leader', 'Successfully refer 5 friends', 1),
    (14, 'Feline Gatherer', 'Successfully refer 10 friends', 1),
    (15, 'Cat Wrangler', 'Successfully refer 20 friends', 1),
    (16, 'Cat Herder', 'Successfully refer 50 friends', 1);

INSERT INTO badge_group_conjunctions (badge_id, badge_group_id, sort_order) VALUES
    (8, 2, 10),
    (2, 2, 20),
    (3, 2, 30),
    (4, 2, 40),

    (5, 5, 10),
    (6, 5, 20),

    (9, 3, 10),
    (7, 3, 20),
    (10, 3, 30),
    (11, 3, 40),

    (12, 4, 10),
    (13, 4, 20),
    (14, 4, 30),
    (15, 4, 40),
    (16, 4, 50);

INSERT INTO progression_event_types (id, event_type) VALUES
    (6, 'referral_created');

INSERT INTO badge_conditions (id, badge_id, progression_event_type_id, operation, required_count) VALUES
    (8, 8, 4, 'gte', 100),

    (9, 9, 5, 'gte', 10),
    (10, 10, 5, 'gte', 1_000),
    (11, 11, 5, 'gte', 10_000),

    (12, 12, 6, 'gte', 1),
    (13, 13, 6, 'gte', 5),
    (14, 14, 6, 'gte', 10),
    (15, 15, 6, 'gte', 20),
    (16, 16, 6, 'gte', 50);


UPDATE badges SET score = 100 WHERE id = 1;

UPDATE badges SET score = 20 WHERE id = 8;
UPDATE badges SET score = 50 WHERE id = 2;
UPDATE badges SET score = 150 WHERE id = 3;
UPDATE badges SET score = 500 WHERE id = 4;

UPDATE badges SET score = 10 WHERE id = 9;
UPDATE badges SET score = 40 WHERE id = 7;
UPDATE badges SET score = 120 WHERE id = 10;
UPDATE badges SET score = 400 WHERE id = 11;

UPDATE badges SET score = 15 WHERE id = 12;
UPDATE badges SET score = 50 WHERE id = 13;
UPDATE badges SET score = 120 WHERE id = 14;
UPDATE badges SET score = 250 WHERE id = 15;
UPDATE badges SET score = 600 WHERE id = 16;