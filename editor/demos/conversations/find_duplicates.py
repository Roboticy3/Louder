import json

with open("merchant.json", "r") as f:
    data = json.load(f)

hubs = [item for item in data if item.get("type") == "hub"]

id_map = {}
for hub in hubs:
    cid = hub.get("conversation_id")
    if cid is None:
        continue
    id_map.setdefault(cid, []).append(hub)

print("=== Duplicate conversation_ids ===")
found = False
for cid, hubs_list in sorted(id_map.items(), key=lambda x: int(x[0])):
    if len(hubs_list) > 1:
        found = True
        print(f"\nconversation_id: {cid} ({len(hubs_list)} entries)")
        for i, h in enumerate(hubs_list, 1):
            speaker = h.get("speaker", "(no speaker)")
            text = h.get("text", "(no text)")
            print(f"  {i}. speaker={speaker!r}, text={text[:80]!r}")

if not found:
    print("No duplicates found.")
