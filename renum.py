#!/usr/bin/env python
import sys

def rust_enum_values(input):
  parts = [part.strip() for part in input.split(',')]
  for part in parts:
    print('#[serde(rename = "'+part+'")]')
    print(part.replace("_", " ").title().replace(" ", "")+",")

if len(sys.argv) < 2:
  print("you need to specify the input string")

rust_enum_values(sys.argv[1])

