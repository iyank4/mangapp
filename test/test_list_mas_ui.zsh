#!/bin/zsh

set -e

SCRIPT_DIR=${0:A:h:h}
SCRIPT="$SCRIPT_DIR/list_mas.sh"

grep -q "curses.wrapper" "$SCRIPT" || {
  print -u2 "missing curses.wrapper UI"
  exit 1
}
grep -q "TABLE_TOP = 3" "$SCRIPT" || {
  print -u2 "table coordinate constant is missing"
  exit 1
}
grep -q 'getattr(curses, "BUTTON5_PRESSED"' "$SCRIPT" || {
  print -u2 "mouse button handling is not portable"
  exit 1
}
grep -q 'python3 -c "\$(cat <<'"'"'PY'"'"'' "$SCRIPT" || {
  print -u2 "Python source must preserve terminal stdin"
  exit 1
}

OUTPUT=$("$SCRIPT" --plain)
print -r -- "$OUTPUT" | python3 -c '
import csv
import sys

rows = list(csv.DictReader(sys.stdin, delimiter="\t"))
assert rows, "no Mac App Store rows found"
assert list(rows[0]) == ["VENDOR", "NAMA APLIKASI", "VERSI", "ID APLIKASI"]
keys = [(row["VENDOR"].casefold(), row["NAMA APLIKASI"].casefold()) for row in rows]
assert keys == sorted(keys), "rows are not sorted by vendor then application name"
assert all(row["ID APLIKASI"].isdigit() for row in rows)
print("list_mas curses/plain behavior is valid")
'
