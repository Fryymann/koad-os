#!/usr/bin/env python3
import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path


def parse_iso_utc(value: str) -> datetime:
    if value.endswith("Z"):
        value = value[:-1] + "+00:00"
    dt = datetime.fromisoformat(value)
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return dt.astimezone(timezone.utc)


def extract_required_paths(path: Path):
    if not path.exists():
        return []
    text = path.read_text(encoding="utf-8", errors="replace")
    # Extract backticked relative paths such as `.standards/...` or `.agent-ops/...`.
    candidates = re.findall(r"`([^`]+)`", text)
    out = []
    for item in candidates:
        if item.startswith(".") and "/" in item:
            out.append(item)
    # Preserve order and de-duplicate.
    seen = set()
    unique = []
    for item in out:
        if item not in seen:
            seen.add(item)
            unique.append(item)
    return unique


def main():
    parser = argparse.ArgumentParser(description="Check standards sync freshness and required file presence.")
    parser.add_argument("--max-age-hours", type=float, default=24.0)
    parser.add_argument("--manifest", default=".koad/.standards/sync_manifest.json")
    parser.add_argument("--required-sources", default=".koad/.agent-ops/CANONICAL_REQUIRED_SOURCES.md")
    args = parser.parse_args()

    manifest_path = Path(args.manifest)
    required_sources_path = Path(args.required_sources)

    status = "FRESH"
    synced_at = None
    age_hours = None
    missing = []

    if not manifest_path.exists():
        status = "MISSING_MANIFEST"
    else:
        try:
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            synced_at_raw = manifest.get("synced_at")
            if not synced_at_raw:
                status = "INVALID_MANIFEST"
            else:
                synced_at = parse_iso_utc(synced_at_raw)
                age_hours = (datetime.now(timezone.utc) - synced_at).total_seconds() / 3600.0
                if age_hours > args.max_age_hours:
                    status = "STALE"
        except Exception:
            status = "INVALID_MANIFEST"

    required_paths = extract_required_paths(required_sources_path)
    for rel in required_paths:
        if not Path(rel).exists():
            missing.append(rel)

    if missing and status == "FRESH":
        status = "MISSING_REQUIRED_SOURCES"

    print(f"manifest: {manifest_path.resolve()}")
    print(f"required_sources: {required_sources_path.resolve()}")
    print(f"synced_at: {synced_at.isoformat().replace('+00:00', 'Z') if synced_at else 'unknown'}")
    print(f"age_hours: {age_hours:.2f}" if age_hours is not None else "age_hours: unknown")
    print(f"max_age_hours: {args.max_age_hours:.2f}")
    print(f"required_paths_checked: {len(required_paths)}")
    print(f"missing_required_paths: {len(missing)}")
    if missing:
        for path in missing:
            print(f"missing: {path}")
    print(f"status: {status}")


if __name__ == "__main__":
    main()
