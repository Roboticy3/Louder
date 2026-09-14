import argparse
import json
import os
import shutil
import sys

ENTITY_TYPES = ("hub", "speaker")

DEFAULT_TARGETS = (
    os.path.join("demos", "conversations"),
    os.path.join("demos", "tutorials"),
)


def _id_key(item):
    cid = item.get("conversation_id")
    if cid is None:
        return None
    return str(cid)


def _read_json(path):
    with open(path, "r", encoding="utf-8", newline="") as f:
        return json.load(f)


def _write_json(path, data):
    with open(path, "w", encoding="utf-8", newline="") as f:
        json.dump(data, f, indent="\t", ensure_ascii=False)


def repair_file(path, dry_run=False):
    data = _read_json(path)
    if not isinstance(data, list):
        return "skipped (not a JSON array)"

    seen = {}
    removed = []
    changed_content = []
    result = []

    for item in data:
        if not isinstance(item, dict):
            result.append(item)
            continue
        if item.get("type") not in ENTITY_TYPES:
            result.append(item)
            continue
        key = _id_key(item)
        if key is None:
            result.append(item)
            continue
        if key in seen:
            removed.append(key)
            if seen[key] != item:
                changed_content.append(key)
            continue
        seen[key] = item
        result.append(item)

    if not removed:
        return "clean"

    if dry_run:
        return "would fix: {} duplicate id(s) removed ({})".format(
            len(removed), ", ".join(sorted(removed))
        )

    backup = path + ".bak"
    shutil.copy2(path, backup)

    _write_json(path, result)

    fixed = len(removed)
    note = ""
    if changed_content:
        note = "; DIFFERING content dropped for ids: " + ", ".join(sorted(changed_content))
    return "fixed: {} duplicate id(s) removed ({}){}".format(
        fixed, ", ".join(sorted(removed)), note
    )


def main():
    parser = argparse.ArgumentParser(
        description="Remove duplicate entities (same conversation_id) from graph JSON files."
    )
    parser.add_argument("paths", nargs="*", help="JSON files (default: demos/conversations and demos/tutorials)")
    parser.add_argument("--dry-run", action="store_true", help="report only, don't modify files")
    parser.add_argument("--root", default=".", help="project root for default paths")
    args = parser.parse_args()

    if args.paths:
        files = args.paths
    else:
        files = []
        for target in DEFAULT_TARGETS:
            target_dir = os.path.join(args.root, *target.split(os.sep))
            if not os.path.isdir(target_dir):
                print("warning: directory not found: {}".format(target_dir))
                continue
            for name in sorted(os.listdir(target_dir)):
                if name.endswith(".json"):
                    files.append(os.path.join(target_dir, name))

    failed = False
    for path in files:
        if not os.path.isfile(path):
            print("MISSING  {}".format(path))
            failed = True
            continue
        try:
            status = repair_file(path, dry_run=args.dry_run)
        except (ValueError, KeyError, TypeError) as exc:
            print("ERROR    {}: {}".format(path, exc))
            failed = True
            continue
        print("{:<7} {}".format(status.split(":")[0].upper(), path))
        if ":" in status:
            print("            " + status.split(":", 1)[1].strip())

    if failed:
        sys.exit(2)


if __name__ == "__main__":
    main()