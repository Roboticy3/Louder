"""Regenerate the Bare Minimum demo's data files from the editor's own output.

The editor writes conversation and speaker documents in a schema the engine does
not read, so the demo needs schema-correct copies. Doing that by hand invited a
typo and left the copies untraceable, so this does it mechanically and can be
re-run whenever the editor's output changes.

Conversations:
  - "auto": true        -> folded into the "flags" bitfield (bit 0 is AUTO)
  - "_private": true    -> dropped; "recipients" is the only privacy mechanism,
                           and the hub carrying "_private" already has them
  - "position_offset:x" -> dropped; editor layout, carried as additional data
    "position_offset:y"    otherwise and read back as user-defined properties
  - camera records      -> dropped; the engine ignores them

Speaker configs already use the engine's schema ("name", "flags", "listening"),
so they are copied through untouched.

Content is never rewritten. A speaker name that looks like a typo stays as it is,
because that is the editor's content to fix, not this script's.

    python make_demo_data.py
"""

import json
import os

AUTO_FLAG = 1  # HubFlags::AUTO

EDITOR_ROOT = os.path.join("..", "..", "editor", "demos")

CONVERSATION_SOURCE = os.path.join(EDITOR_ROOT, "conversations", "jail.json")
CONFIG_SOURCE = os.path.join(EDITOR_ROOT, "speakers", "test.json")

HUB_KEYS = {
    "type",
    "conversation_id",
    "speaker",
    "text",
    "tags",
    "choices",
    "expressions",
    "callbacks",
    "flags",
    "recipients",
    "channel",
    "priority",
}

DROPPED_HUB_KEYS = {
    "auto",  # folded into flags
    "_private",  # recipients is the real mechanism
    "position_offset:x",  # editor layout
    "position_offset:y",
}


def convert_conversation(records):
    out = []
    for record in records:
        kind = record.get("type")

        if kind == "camera":
            # Only remembers where the editor was scrolled to.
            continue

        if kind == "connection":
            out.append(
                {
                    "type": "connection",
                    "from_id": record["from_id"],
                    "from_port": record["from_port"],
                    "to_id": record["to_id"],
                }
            )
            continue

        if kind != "hub":
            raise SystemExit("unexpected record type: %r" % (kind,))

        flags = int(record.get("flags", 0) or 0)
        if record.get("auto"):
            flags |= AUTO_FLAG

        hub = {"type": "hub", "conversation_id": record["conversation_id"]}

        # Keep the schema fields the document actually sets, so the copy stays
        # recognisable next to the editor's file.
        for key in ("speaker", "text", "tags", "choices", "expressions",
                    "callbacks", "recipients", "channel", "priority"):
            if key in record:
                hub[key] = record[key]

        if flags:
            hub["flags"] = flags

        unknown = set(record) - HUB_KEYS - DROPPED_HUB_KEYS - {"type"}
        if unknown:
            raise SystemExit(
                "hub %s has fields the schema does not define: %s"
                % (record.get("conversation_id"), sorted(unknown))
            )

        out.append(hub)

    return out


def write(path, value):
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        json.dump(value, handle, indent="\t", ensure_ascii=False)
        handle.write("\n")
    print("wrote %s" % path)


def main():
    with open(CONVERSATION_SOURCE, encoding="utf-8") as handle:
        conversation = convert_conversation(json.load(handle))
    write("jail.json", conversation)

    with open(CONFIG_SOURCE, encoding="utf-8") as handle:
        config = json.load(handle)
    for record in config:
        if record.get("type") == "speaker":
            unknown = set(record) - {"type", "name", "flags", "listening"}
            if unknown:
                raise SystemExit(
                    "speaker %r has fields the schema does not define: %s"
                    % (record.get("name"), sorted(unknown))
                )
    write("speakers.json", config)


if __name__ == "__main__":
    main()