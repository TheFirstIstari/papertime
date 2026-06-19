#!/usr/bin/env python3
"""
PaperTime Service Cleaner — M3.1
Removes backward-time stops caused by column misalignment in the parser.
Simple truncation approach: when a stop's time goes backward relative to
the maximum valid time seen so far, truncate the service at that point.
This removes corrupted data while preserving valid prefixes.

Usage: python3 clean_services.py
"""

import json, shutil
from pathlib import Path

BASE = Path(__file__).parent.parent
SVC_DIR = BASE / "static" / "services"
BACKUP_DIR = BASE / "pdf2data" / "backups"


def count_valid(stops):
    """Count stops with non-null times."""
    return sum(1 for s in stops if (s.get("dep") or s.get("arr")) is not None)


def clean_table(services):
    """Remove backward-time stops from all services via truncation.
    Returns (changes, removed)."""
    changes = 0
    removed = 0

    for svc in services:
        new_stops = []
        max_valid = -1
        prev_time = -1

        for stop in svc["stops"]:
            time = stop.get("dep") or stop.get("arr")
            if time is None:
                new_stops.append(stop)
                continue

            # Midnight crossing
            adj = time
            if prev_time >= 720 and time < 240:
                adj += 1440
            prev_time = time

            if adj >= max_valid:
                max_valid = adj
                new_stops.append(stop)
            else:
                removed += 1
                changes += 1

        svc["stops"] = new_stops

    return changes, removed


def main():
    if not SVC_DIR.exists():
        print(f"Services directory not found: {SVC_DIR}")
        return

    # Backup current state
    BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    for f in SVC_DIR.glob("*.json"):
        shutil.copy2(f, BACKUP_DIR / f.name)
    n_files = len(list(SVC_DIR.glob("*.json")))
    print(f"Backed up {n_files} files to {BACKUP_DIR}")

    total_changes = 0
    total_removed = 0
    cleaned = 0

    before_short = 0
    after_short = 0
    total_svcs = 0

    for fpath in sorted(SVC_DIR.glob("*.json")):
        with open(fpath) as f:
            data = json.load(f)
        services = data.get("services", [])
        if not services:
            continue

        total_svcs += len(services)
        before_short += sum(1 for s in services if count_valid(s["stops"]) < 2)

        changes, removed = clean_table(services)
        if changes:
            cleaned += 1
            total_changes += changes
            total_removed += removed
            with open(fpath, "w") as f:
                json.dump(data, f, indent=2)

        after_short += sum(1 for s in services if count_valid(s["stops"]) < 2)

    # Stats
    still_bad = 0
    for fpath in SVC_DIR.glob("*.json"):
        with open(fpath) as f:
            d = json.load(f)
        for svc in d.get("services", []):
            prev = -1
            for s in svc["stops"]:
                t = s.get("dep") or s.get("arr")
                if t is None: continue
                if prev >= 720 and t < 240:
                    prev = t
                    continue
                if prev >= 0 and t < prev:
                    still_bad += 1
                    break
                prev = t

    print(f"\nCleaned {cleaned} tables")
    print(f"Stops removed: {total_removed}")
    print(f"Short services (<2 valid): {before_short} → {after_short} (of {total_svcs})")
    print(f"Still backward: {still_bad}")


if __name__ == "__main__":
    main()
