import sys
import json


d = json.dumps(json.loads(sys.stdin.read()), separators=(",", ":"))
d = d.replace('"', '\\"')
print(f'"{d}"')