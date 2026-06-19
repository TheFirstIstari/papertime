#!/usr/bin/env python3
"""
PaperTime Service Cleaner — M4.1
Merges consecutive same-station stops (arrival+departure pairs)
and recovers column-shifted data by borrowing from the FIRST clean
service (not just i+1).

The key insight: when service i has backward time at station X,
the correct data for X and all subsequent stations exists in some
later service j (where j > i) whose data at X is clean. We find
the FIRST such j and borrow everything from X onward.

Usage: python3 clean_services.py
"""

import json, shutil
from pathlib import Path

BASE = Path(__file__).parent.parent
SVC_DIR = BASE / "static" / "services"
BACKUP_DIR = BASE / "pdf2data" / "backups"


def count_valid(stops):
    return sum(1 for s in stops if (s.get("dep") or s.get("arr")) is not None)


def find_first_backward(stops):
    """Find index of first backward-time stop (excluding midnight crossing).
    Applies midnight offset (adding 1440) when comparing across midnight.
    Returns (index, is_midnight_zone) where index=-1 means clean."""
    prev_time = -1      # raw time from previous stop
    prev_adj = -1       # midnight-adjusted previous time
    crossed_midnight = False
    for i, s in enumerate(stops):
        t = s.get("dep") or s.get("arr")
        if t is None:
            continue
        # Apply midnight offset for comparison
        adj = t
        if not crossed_midnight and prev_adj >= 720 and t < 240:
            adj += 1440  # first crossing
            crossed_midnight = True
            prev_time = t
            prev_adj = adj
            continue
        elif crossed_midnight and t < 240:
            adj += 1440  # still in post-midnight zone

        if prev_time >= 0 and adj < prev_adj:
            # Only treat as midnight zone if the backward jump is small
            # and we've crossed midnight recently (within the last 3 stations)
            is_midnight = crossed_midnight and (prev_adj < 720 + 480 or (prev_adj - adj) <= 120)
            return i, is_midnight

        prev_time = t
        prev_adj = adj
    return -1, False


def get_station_time(stops, station):
    """Get the first time entry for a station. Returns (idx, time) or (None, None)."""
    for i, s in enumerate(stops):
        if s["station"] == station:
            t = s.get("dep") or s.get("arr")
            if t is not None:
                return i, t
    return None, None


def merge_same_station_stops(services):
    """Merge consecutive same-station stops (arrival + departure split)."""
    changes = 0
    for svc in services:
        merged = []
        i = 0
        while i < len(svc["stops"]):
            stop = svc["stops"][i]
            if i + 1 < len(svc["stops"]) and svc["stops"][i+1]["station"] == stop["station"]:
                nxt = svc["stops"][i+1]
                merged.append({
                    "station": stop["station"],
                    "arr": stop.get("arr") or nxt.get("arr"),
                    "dep": stop.get("dep") or nxt.get("dep"),
                })
                i += 2
                changes += 1
            else:
                merged.append(stop)
                i += 1
        if changes:
            svc["stops"] = merged
    return changes


def find_clean_source(services, svc_idx, target_station, min_acceptable_time):
    """
    Find the first service j > svc_idx whose time at target_station
    is >= min_acceptable_time. Scans ALL remaining services.
    Returns (j, borrow_idx) or (None, None).
    """
    for j in range(svc_idx + 1, len(services)):
        for k, ns in enumerate(services[j]["stops"]):
            if ns["station"] == target_station:
                nt = ns.get("dep") or ns.get("arr")
                if nt is not None and nt >= min_acceptable_time:
                    return j, k
                else:
                    break  # try next service
    return None, None


def fix_column_shifts(services):
    """
    Fix column-shifted data by iterative borrowing from the FIRST clean service.
    Processes from last to first so fixed data cascades correctly.
    """
    borrowed = 0
    truncated = 0

    for i in range(len(services) - 1, -1, -1):
        svc = services[i]
        max_iter = 15

        for _ in range(max_iter):
            fix_idx, is_midnight = find_first_backward(svc["stops"])
            if fix_idx < 0:
                break

            target = svc["stops"][fix_idx]["station"]

            # For midnight-crossing services with small backward jumps,
            # nudge the time forward to maintain monotonic progression.
            if is_midnight:
                bad_stop = svc["stops"][fix_idx]
                bad_raw = bad_stop.get("dep") or bad_stop.get("arr")
                
                # Find previous valid stop and normalize both to adjusted times
                prev_raw = None
                for p in range(fix_idx - 1, -1, -1):
                    pt = svc["stops"][p].get("dep") or svc["stops"][p].get("arr")
                    if pt is not None:
                        prev_raw = pt
                        break
                
                if prev_raw is not None:
                    prev_adj = prev_raw + 1440  # post-midnight
                    bad_adj = bad_raw + 1440
                    diff = prev_adj - bad_adj
                    
                    # Small diff (≤5 min): nudge forward to match prev time
                    if diff > 0 and diff <= 5:
                        new_raw = bad_raw + diff
                        if bad_stop.get("dep") is not None:
                            bad_stop["dep"] = new_raw
                        if bad_stop.get("arr") is not None:
                            bad_stop["arr"] = new_raw
                        svc["stops"][fix_idx] = bad_stop
                        truncated += 1
                        continue
                
                # Fallback: remove the bad stop
                svc["stops"] = svc["stops"][:fix_idx] + svc["stops"][fix_idx+1:]
                truncated += 1
                continue

            # Find the acceptable minimum time
            prev_t = None
            for p in range(fix_idx - 1, -1, -1):
                t_val = svc["stops"][p].get("dep") or svc["stops"][p].get("arr")
                if t_val is not None:
                    prev_t = t_val
                    break

            if prev_t is None:
                svc["stops"] = svc["stops"][:fix_idx]
                truncated += 1
                break

            # Find the first clean service with this station
            src_idx, borrow_idx = find_clean_source(services, i, target, prev_t)

            if src_idx is not None:
                head = svc["stops"][:fix_idx]
                tail = services[src_idx]["stops"][borrow_idx:]
                svc["stops"] = head + tail
                borrowed += 1
            else:
                svc["stops"] = svc["stops"][:fix_idx]
                truncated += 1
                break

    return borrowed, truncated


def main():
    if not SVC_DIR.exists():
        print(f"Services directory not found: {SVC_DIR}")
        return

    # Backup
    BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    for f in SVC_DIR.glob("*.json"):
        shutil.copy2(f, BACKUP_DIR / f.name)
    n_files = len(list(SVC_DIR.glob("*.json")))
    print(f"Backed up {n_files} files to {BACKUP_DIR}")

    total_merged = 0
    total_borrowed = 0
    total_truncated = 0
    total_svcs = 0
    before_short = 0
    after_short = 0
    before_backward = 0

    for fpath in sorted(SVC_DIR.glob("*.json")):
        with open(fpath) as f:
            data = json.load(f)
        services = data.get("services", [])
        if not services:
            continue

        total_svcs += len(services)
        before_short += sum(1 for s in services if count_valid(s["stops"]) < 2)
        before_backward += sum(1 for s in services if find_first_backward(s["stops"])[0] >= 0)

        merged = merge_same_station_stops(services)
        borrowed, truncated = fix_column_shifts(services)

        if merged or borrowed or truncated:
            with open(fpath, "w") as f:
                json.dump(data, f, indent=2)

        after_short += sum(1 for s in services if count_valid(s["stops"]) < 2)

        total_merged += merged
        total_borrowed += borrowed
        total_truncated += truncated

    # Verification
    still_bad = 0
    for fpath in SVC_DIR.glob("*.json"):
        with open(fpath) as f:
            d = json.load(f)
        for svc in d.get("services", []):
            idx, _ = find_first_backward(svc["stops"])
            if idx >= 0:
                still_bad += 1

    print(f"\nResults:")
    print(f"  Tables: {n_files}")
    print(f"  Same-station merges: {total_merged}")
    print(f"  Column borrows: {total_borrowed}")
    print(f"  Truncations: {total_truncated}")
    print(f"  Backward-time: {before_backward} → {still_bad}")
    print(f"  Short services (<2 pts): {before_short} → {after_short} (of {total_svcs})")


if __name__ == "__main__":
    main()
