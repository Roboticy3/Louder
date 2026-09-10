import json

with open("merchant.json", "r") as f:
    data = json.load(f)

hubs = [item for item in data if item.get("type") == "hub"]
connections = [item for item in data if item.get("type") == "connection"]

conn_map = {}
for conn in connections:
    conn_map.setdefault(conn["from_id"], set()).add(conn["from_choice"])

print("=== Incomplete hubs ===")
print("(Hub choices with no matching connection, or choices-less hubs with no outbound")
print(" empty-choice connection; 'Note' hubs excluded)\n")

found = False
for hub in sorted(hubs, key=lambda h: int(h.get("conversation_id", 0))):
    if hub.get("speaker") == "Note":
        continue
    cid = hub.get("conversation_id")
    if cid is None:
        continue
    outbound = conn_map.get(cid, set())
    choices = [c for c in hub.get("choices") or [] if c != ""]

    if not choices and "branch" in hub:
        continue

    if choices:
        missing = [c for c in choices if c not in outbound]
        if not missing:
            continue
        found = True
        speaker = hub.get("speaker", "(no speaker)")
        text = hub.get("text", "(no text)")
        print(f"  id={cid}  speaker={speaker!r}  text={text[:80]!r}")
        for c in choices:
            status = "OK" if c in outbound else "MISSING"
            print(f"      [{status}] {c!r}")
    else:
        if "" in outbound:
            continue
        found = True
        marker = "auto" if hub.get("auto") else "is_choice" if hub.get("is_choice") else "plain"
        speaker = hub.get("speaker", "(no speaker)")
        text = hub.get("text", "(no text)")
        print(f"  id={cid} [{marker}]  speaker={speaker!r}  text={text[:80]!r}  (no choices, no outbound empty-choice connection)")

if not found:
    print("No incomplete hubs found.")