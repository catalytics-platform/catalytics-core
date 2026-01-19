#!/usr/bin/env python3
"""
Script to sync badges for all beta applicants from the database.
Reads public_key from beta_applicants table and calls the badges sync API.
"""

import os
import time
import requests
import psycopg2
from typing import List


def get_database_connection():
    """Establish connection to PostgreSQL database using environment variable."""
    database_url = os.getenv('DATABASE_URL')
    if not database_url:
        raise ValueError("DATABASE_URL environment variable is required")
    
    try:
        conn = psycopg2.connect(database_url)
        return conn
    except psycopg2.Error as e:
        print(f"Error connecting to database: {e}")
        return None


def get_public_keys() -> List[str]:
    """Fetch all public_key values from the beta_applicants table."""
    conn = get_database_connection()
    if not conn:
        return []
    
    cursor = None
    try:
        cursor = conn.cursor()
        cursor.execute("SELECT public_key FROM beta_applicants WHERE public_key IS NOT NULL")
        
        results = cursor.fetchall()
        public_keys = [row[0] for row in results if row[0]]
        
        print(f"Found {len(public_keys)} public keys in beta_applicants table")
        return public_keys
        
    except psycopg2.Error as e:
        print(f"Error fetching public keys: {e}")
        return []
    finally:
        if cursor:
            cursor.close()
        conn.close()


def sync_badges_for_public_key(public_key: str, api_base_url: str) -> bool:
    """Call the badges sync API for a given public key."""
    url = f"{api_base_url}/api/badges/sync?publicKey={public_key}"

    try:
        response = requests.get(url, timeout=30)
        
        if response.status_code == 200:
            print(f"✓ Successfully synced badges for {public_key}")
            return True
        else:
            print(f"✗ Failed to sync badges for {public_key}. Status: {response.status_code}")
            return False
            
    except requests.exceptions.RequestException as e:
        print(f"✗ Error calling API for {public_key}: {e}")
        return False


def get_leaderboard_stats() -> dict:
    """Get basic statistics about the leaderboard for logging."""
    conn = get_database_connection()
    if not conn:
        return {}
    
    cursor = None
    try:
        cursor = conn.cursor()
        cursor.execute("""
            SELECT 
                COUNT(*) as total_entries,
                COALESCE(MAX(total_score), 0) as max_score,
                COALESCE(MIN(total_score), 0) as min_score,
                COUNT(CASE WHEN previous_rank IS NOT NULL THEN 1 END) as entries_with_history
            FROM leaderboard_entries
        """)
        
        result = cursor.fetchone()
        return {
            'total_entries': result[0] if result else 0,
            'max_score': result[1] if result else 0,
            'min_score': result[2] if result else 0,
            'entries_with_history': result[3] if result else 0
        }
        
    except psycopg2.Error as e:
        print(f"Error getting leaderboard stats: {e}")
        return {}
    finally:
        if cursor:
            cursor.close()
        conn.close()


def refresh_leaderboard_entries() -> bool:
    """
    Update the leaderboard_entries table with current badge scores and rankings.
    Preserves rank history by moving current rank to previous_rank.
    """
    conn = get_database_connection()
    if not conn:
        return False
    
    cursor = None
    try:
        cursor = conn.cursor()
        print("Calculating current rankings from badge data...")
        
        # Step 1: Update existing entries
        update_existing_query = """
        UPDATE leaderboard_entries 
        SET 
            previous_rank = rank,
            rank = updates.new_rank,
            total_score = total_score + updates.new_total_score,
            updated_at = NOW()
        FROM (
            WITH current_rankings AS (
                SELECT 
                    ba.id as beta_applicant_id,
                    ba.public_key,
                    COALESCE(SUM(b.score), 0)::INTEGER as new_total_score,
                    ROW_NUMBER() OVER (ORDER BY COALESCE(SUM(b.score), 0) DESC, ba.created_at ASC)::INTEGER as new_rank
                FROM beta_applicants ba
                LEFT JOIN beta_applicant_badges bab ON ba.id = bab.beta_applicant_id  
                LEFT JOIN badges b ON bab.badge_id = b.id
                GROUP BY ba.id, ba.public_key, ba.created_at
            )
            SELECT 
                cr.beta_applicant_id,
                cr.new_rank,
                cr.new_total_score
            FROM current_rankings cr
            WHERE EXISTS (
                SELECT 1 FROM leaderboard_entries le 
                WHERE le.beta_applicant_id = cr.beta_applicant_id
            )
        ) AS updates
        WHERE leaderboard_entries.beta_applicant_id = updates.beta_applicant_id;
        """
        
        cursor.execute(update_existing_query)
        updated_count = cursor.rowcount
        print(f"Updated {updated_count} existing leaderboard entries")
        
        # Step 2: Insert new users
        insert_new_query = """
        INSERT INTO leaderboard_entries 
        (beta_applicant_id, public_key, total_score, rank, previous_rank, created_at, updated_at)
        SELECT 
            cr.beta_applicant_id,
            cr.public_key,
            cr.new_total_score,
            cr.new_rank,
            NULL as previous_rank,
            NOW() as created_at,
            NOW() as updated_at
        FROM (
            WITH current_rankings AS (
                SELECT 
                    ba.id as beta_applicant_id,
                    ba.public_key,
                    COALESCE(SUM(b.score), 0)::INTEGER as new_total_score,
                    ROW_NUMBER() OVER (ORDER BY COALESCE(SUM(b.score), 0) DESC, ba.created_at ASC)::INTEGER as new_rank
                FROM beta_applicants ba
                LEFT JOIN beta_applicant_badges bab ON ba.id = bab.beta_applicant_id  
                LEFT JOIN badges b ON bab.badge_id = b.id
                GROUP BY ba.id, ba.public_key, ba.created_at
            )
            SELECT 
                cr.beta_applicant_id,
                cr.public_key,
                cr.new_total_score,
                cr.new_rank
            FROM current_rankings cr
        ) cr
        WHERE NOT EXISTS (
            SELECT 1 FROM leaderboard_entries le 
            WHERE le.beta_applicant_id = cr.beta_applicant_id
        );
        """
        
        cursor.execute(insert_new_query)
        inserted_count = cursor.rowcount
        print(f"Inserted {inserted_count} new leaderboard entries")
        
        # Commit the transaction
        conn.commit()
        
        total_count = updated_count + inserted_count
        print(f"✓ Leaderboard refresh completed: {updated_count} updated, {inserted_count} new entries (total: {total_count})")
        
        return True
        
    except psycopg2.Error as e:
        print(f"✗ Error refreshing leaderboard: {e}")
        if conn:
            conn.rollback()
        return False
    finally:
        if cursor:
            cursor.close()
        conn.close()


def main():
    """Main function to orchestrate the badge sync and leaderboard refresh process."""
    print("Starting badge sync and leaderboard refresh process...")
    print("=" * 50)
    
    # Get API base URL from environment
    api_base_url = os.getenv('CATALYTICS_API_BASE_URL')
    if not api_base_url:
        raise ValueError("CATALYTICS_API_BASE_URL environment variable is required")
    
    print(f"Using API base URL: {api_base_url}")
    
    # PHASE 1: Badge Synchronization (existing logic)
    print("\nPhase 1: Badge Synchronization")
    print("-" * 30)
    
    # Get all public keys from database
    public_keys = get_public_keys()
    
    if not public_keys:
        print("No public keys found for badge sync. Continuing to leaderboard refresh...")
        badge_sync_successful = True
        successful_syncs = 0
        failed_syncs = 0
    else:
        # Process each public key for badge sync
        successful_syncs = 0
        failed_syncs = 0
        
        for i, public_key in enumerate(public_keys, 1):
            print(f"Processing {i}/{len(public_keys)}: {public_key}")
            
            success = sync_badges_for_public_key(public_key, api_base_url)
            
            if success:
                successful_syncs += 1
            else:
                failed_syncs += 1
            
            # Sleep for 2 seconds between calls (except for the last one)
            if i < len(public_keys):
                print("Waiting 2 seconds before next call...")
                time.sleep(2)
        
        badge_sync_successful = failed_syncs == 0
    
    # PHASE 2: Leaderboard Refresh (new logic)
    print("\n" + "=" * 50)
    print("Phase 2: Leaderboard Refresh")
    print("-" * 30)
    
    # Get pre-refresh stats
    pre_stats = get_leaderboard_stats()
    if pre_stats:
        print(f"Pre-refresh leaderboard: {pre_stats['total_entries']} entries, "
              f"max score: {pre_stats['max_score']}, min score: {pre_stats['min_score']}")
    
    # Perform leaderboard refresh
    leaderboard_refresh_successful = refresh_leaderboard_entries()
    
    # Get post-refresh stats
    if leaderboard_refresh_successful:
        post_stats = get_leaderboard_stats()
        if post_stats:
            print(f"Post-refresh leaderboard: {post_stats['total_entries']} entries, "
                  f"max score: {post_stats['max_score']}, min score: {post_stats['min_score']}")
    
    # FINAL SUMMARY
    print("\n" + "=" * 50)
    print("Job Summary")
    print("-" * 30)
    print(f"Badge Sync:")
    print(f"  - Total processed: {len(public_keys)}")
    print(f"  - Successful: {successful_syncs}")
    print(f"  - Failed: {failed_syncs}")
    print(f"  - Status: {'✓ Success' if badge_sync_successful else '✗ Some failures'}")
    
    print(f"Leaderboard Refresh:")
    print(f"  - Status: {'✓ Success' if leaderboard_refresh_successful else '✗ Failed'}")
    
    overall_success = badge_sync_successful and leaderboard_refresh_successful
    print(f"Overall Status: {'✓ Complete Success' if overall_success else '✗ Some Issues'}")


if __name__ == "__main__":
    main()