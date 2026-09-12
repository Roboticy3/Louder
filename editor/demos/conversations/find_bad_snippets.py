import json
import re

with open("merchant.json", "r") as f:
    data = json.load(f)

hubs = [item for item in data if item.get("type") == "hub"]

ASSIGNMENT_RE = re.compile(r"(?:\+\+|--|\+=|-=|(?<![!<>=])=(?!=))")
OPERATORS = "=, +=, -=, ++, or --"


def has_assignment(code):
    return bool(ASSIGNMENT_RE.search(code))


print("=== Hubs with invalid statements or conditions ===")
print(f"(statement lines must contain {OPERATORS}; condition lines must not)\n")

found = False
for hub in sorted(hubs, key=lambda h: int(h.get("conversation_id", 0))):
    cid = hub.get("conversation_id")
    problems = []

    for line in (hub.get("statements") or "").splitlines():
        if line.strip() and not has_assignment(line):
            problems.append(f"statement missing assignment: {line!r}")

    for line in (hub.get("conditions") or "").splitlines():
        if line.strip() and has_assignment(line):
            problems.append(f"condition contains assignment: {line!r}")

    if not problems:
        continue
    found = True
    speaker = hub.get("speaker", "(no speaker)")
    text = hub.get("text", "(no text)")
    print(f"  id={cid}  speaker={speaker!r}  text={text[:80]!r}")
    for p in problems:
        print(f"      {p}")

if not found:
    print("No hubs with invalid statements or conditions found.")