import json

with open("merchant.json", "r") as f:
    data = json.load(f)

hubs = [item for item in data if item.get("type") == "hub"]

print("=== Hubs with both auto and once ===")

found = False
for hub in sorted(hubs, key=lambda h: int(h.get("conversation_id", 0))):
    once = hub.get("once", False)
    auto = hub.get("auto", False)
    if not once or not auto:
        continue
    found = True
    cid = hub.get("conversation_id")
    text = hub.get("text", "(no text)")
    speaker = hub.get("speaker", "")
    print(f"  id={cid}  speaker={speaker!r}  text={text[:80]!r}")

if not found:
    print("No hubs with missing or empty speaker found.")