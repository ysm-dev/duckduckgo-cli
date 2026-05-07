#!/bin/sh
set -eu

limit=${LINE_CAP:-150}
violations=$(find crates -path '*/src/*.rs' -type f | while IFS= read -r file; do
  if [ "${file#*/region/codes.rs}" != "$file" ]; then
    continue
  fi
  lines=$(wc -l < "$file" | tr -d ' ')
  if [ "$lines" -gt "$limit" ]; then
    echo "$file:$lines exceeds ${limit} lines; split it into a sibling module"
  fi
done)

if [ -n "$violations" ]; then
  echo "$violations" >&2
  exit 1
fi
