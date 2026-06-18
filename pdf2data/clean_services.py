#!/usr/bin/env python3
"""
PaperTime Service Cleaner — M2.5
Fixes data quality issues in parsed service JSON files:
1. Removes backward-time stops caused by column misalignment
   (when arrival/departure pairs shift column assignments)
2. Preserves legitimate midnight crossings (time drops from >=720 to <240)
3. Preserves None/null time entries (station starting points, through stations)

Usage: python3 clean_services.py
"""

import json, os, re
from pathlib import Path

BASE = Path(__file__).parent.parent
SVC_DIR = BASE / "static" / "services"
BACKUP_DIR = BASE / "pdf2data" / "backups"


def clean_service_stops(services, table_num):
    """Remove backward-time stops from all services in a table."""
    changes = 0
    removed = 0

    for svc in services:
        new_stops = []
        max_valid_time = -1
        prev_time = -1

        for stop in svc["stops"]:
            time = stop.get("dep") or stop.get("arr")
            station = stop["station"]

            if time is None:
                # Keep null-time entries (starting points, through stations)
                new_stops.append(stop)
                continue

            adjusted_time = time
            # Midnight crossing: time drops from evening (>=720) to early morning (<240)
            if prev_time >= 720 and time < 240:
                adjusted_time += 1440
            prev_time = time

            if adjusted_time >= max_valid_time:
                max_valid_time = adjusted_time
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

    # Create backup
    BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    import shutil
    for f in SVC_DIR.glob("*.json"):
        shutil.copy2(f, BACKUP_DIR / f.name)
    print(f"Backed up {len(list(SVC_DIR.glob('*.json')))} files to {BACKUP_DIR}")

    total_changes = 0
    total_removed = 0
    tables_cleaned = 0

    for fpath in sorted(SVC_DIR.glob("*.json")):
        table_num = fpath.stem
        with open(fpath) as f:
            data = json.load(f)

        services = data.get("services", [])
        if not services:
            continue

        changes, removed = clean_service_stops(services, table_num)
        if changes > 0:
            tables_cleaned += 1
            total_changes += changes
            total_removed += removed
            with open(fpath, "w") as f:
                json.dump(data, f, indent=2)

    print(f"\nCleaned {tables_cleaned} tables")
    print(f"Total stop changes: {total_changes}")
    print(f"Total stops removed: {total_removed}")
    print(f"Backup preserved at: {BACKUP_DIR}")

    # Stats: how many backward-time services remain?
    still_backward = 0
    for fpath in sorted(SVC_DIR.glob("*.json")):
        with open(fpath) as f:
            data = json.load(f)
        for svc in data.get("services", []):
            times = []
            for s in svc["stops"]:
                t = s.get("dep") or s.get("arr")
                times.append(t)
            for i in range(1, len(times)):
                if times[i] is not None and times[i-1] is not None:
                    if times[i-1] > 720 and times[i] < 240:
                        continue
                    if times[i] < times[i-1]:
                        still_backward += 1
                        break

    print(f"Services still with backward time: {still_backward}")


if __name__ == "__main__":
    main()
