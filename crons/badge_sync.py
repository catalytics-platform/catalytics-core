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


def main():
    """Main function to orchestrate the badge sync process."""
    print("Starting badge sync process...")
    print("=" * 50)
    
    # Get API base URL from environment
    api_base_url = os.getenv('CATALYTICS_API_BASE_URL')
    if not api_base_url:
        raise ValueError("CATALYTICS_API_BASE_URL environment variable is required")
    
    print(f"Using API base URL: {api_base_url}")
    
    # Get all public keys from database
    public_keys = get_public_keys()
    
    if not public_keys:
        print("No public keys found. Exiting.")
        return
    
    # Process each public key
    successful_syncs = 0
    failed_syncs = 0
    
    for i, public_key in enumerate(public_keys, 1):
        print(f"\nProcessing {i}/{len(public_keys)}: {public_key}")
        
        success = sync_badges_for_public_key(public_key, api_base_url)
        
        if success:
            successful_syncs += 1
        else:
            failed_syncs += 1
        
        # Sleep for 2 seconds between calls (except for the last one)
        if i < len(public_keys):
            print("Waiting 2 seconds before next call...")
            time.sleep(2)
    
    # Print summary
    print("\n" + "=" * 50)
    print("Badge sync process completed!")
    print(f"Total processed: {len(public_keys)}")
    print(f"Successful syncs: {successful_syncs}")
    print(f"Failed syncs: {failed_syncs}")


if __name__ == "__main__":
    main()