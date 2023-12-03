#!/usr/bin/env python3

import subprocess
from pathlib import Path

path = Path.home() / ".keyboard_layout"

layouts = [("us", None), ("rs", "latin"), ("rs", None)]

with open(path, "r") as f:
    current = int(f.read())

next = (current + 1) % len(layouts)

print(next)

with open(path, "w") as f:
    f.write(str(next))

layout, variant = layouts[next]

cmd = ["setxkbmap", layout]
if variant is not None:
    cmd.extend(["-variant", variant])

subprocess.run(cmd)
