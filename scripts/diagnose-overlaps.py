#!/usr/bin/env python3
"""
Diagnostic script to find overlapping events in ActivityWatch buckets.
"""

import requests
from datetime import datetime, timedelta
from dateutil.parser import isoparse

API_BASE = "http://localhost:5600/api/0"

def check_bucket_overlaps(bucket_id, limit=50):
    """Check for overlapping events in a bucket."""
    print(f"\n{'='*70}")
    print(f"Checking bucket: {bucket_id}")
    print('='*70)
    
    # Get events
    resp = requests.get(f"{API_BASE}/buckets/{bucket_id}/events?limit={limit}")
    if resp.status_code != 200:
        print(f"Error fetching events: {resp.status_code}")
        return
    
    events = resp.json()
    if not events:
        print("No events found.")
        return
    
    print(f"Total events: {len(events)}")
    
    # Sort events by timestamp (ascending, like flood.rs does)
    events = sorted(events, key=lambda e: isoparse(e['timestamp']))
    print(f"✓ Events sorted by timestamp (ascending)")
    
    # Check for overlaps
    overlaps = []
    for i in range(len(events) - 1):
        e1 = events[i]
        e2 = events[i + 1]
        
        t1_start = isoparse(e1['timestamp'])
        t1_end = t1_start + timedelta(seconds=e1['duration'])
        t2_start = isoparse(e2['timestamp'])
        
        gap = (t2_start - t1_end).total_seconds()
        
        if gap < 0:
            overlaps.append({
                'index': i,
                'gap_seconds': gap,
                'e1_start': e1['timestamp'],
                'e1_duration': e1['duration'],
                'e1_end': t1_end.isoformat(),
                'e2_start': e2['timestamp'],
                'e1_data': str(e1['data'])[:60],
                'e2_data': str(e2['data'])[:60],
                'data_match': e1['data'] == e2['data']
            })
    
    if overlaps:
        print(f"\n⚠️  Found {len(overlaps)} overlapping events:")
        for overlap in overlaps[:10]:  # Show first 10
            print(f"\n  Event {overlap['index']} → {overlap['index']+1}:")
            print(f"    Gap: {overlap['gap_seconds']:.3f}s (negative = overlap)")
            print(f"    E1: {overlap['e1_start']} + {overlap['e1_duration']}s → {overlap['e1_end']}")
            print(f"    E2: {overlap['e2_start']}")
            print(f"    E1 data: {overlap['e1_data']}")
            print(f"    E2 data: {overlap['e2_data']}")
            print(f"    Same data: {overlap['data_match']}")
            
            # Check if this matches the -0.622s gap
            if abs(overlap['gap_seconds'] - (-0.622)) < 0.01:
                print(f"    ⭐ MATCHES the -0.622s gap from logs!")
    else:
        print("\n✅ No overlapping events found.")

def main():
    # Get all buckets
    resp = requests.get(f"{API_BASE}/buckets")
    if resp.status_code != 200:
        print(f"Error fetching buckets: {resp.status_code}")
        return
    
    buckets = resp.json()
    
    # Check SUPC03 and SUPC04 buckets
    target_buckets = [bid for bid in buckets.keys() if 'SUPC03' in bid or 'SUPC04' in bid]
    
    print(f"Found {len(target_buckets)} SUPC buckets to check:")
    for bid in target_buckets:
        print(f"  - {bid}")
    
    for bucket_id in target_buckets:
        check_bucket_overlaps(bucket_id)

if __name__ == "__main__":
    main()
