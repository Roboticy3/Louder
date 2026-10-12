import json, sys

d = json.load(open("jail.json"))
hubs = {}
for r in d:
    if r.get("type") == "hub":
        hubs[int(r["conversation_id"])] = r

for p in [22, 23, 24, 25, 32, 49, 34, 29, 50]:
    if p in hubs:
        print("=== hub %d ===" % p)
        print(hubs[p].get("text", ""))
        print()