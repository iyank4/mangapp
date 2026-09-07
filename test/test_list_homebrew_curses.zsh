#!/bin/zsh

set -e

SCRIPT_DIR=${0:A:h:h}
SCRIPT="$SCRIPT_DIR/list_homebrew.sh"
OUTPUT=$("$SCRIPT" --plain)

[[ "$OUTPUT" == *"Kategori"* ]] || { print -u2 "missing table category column"; exit 1; }
[[ "$OUTPUT" == *"Nama / Formula"* ]] || { print -u2 "missing table name column"; exit 1; }
[[ "$OUTPUT" == *"Formula tidak berhubungan dengan aplikasi"* ]] || {
  print -u2 "missing final formula section"
  exit 1
}
[[ "$OUTPUT" == *"item utama"* ]] || {
  print -u2 "missing selectable item count"
  exit 1
}
[[ "$OUTPUT" != *"0 item utama"* ]] || {
  print -u2 "curses UI reports zero selectable items"
  exit 1
}
[[ "$OUTPUT" == *"bruno (cask)"* ]] || {
  print -u2 "application rows are missing from the plain table"
  exit 1
}
[[ "$OUTPUT" == *"Cask"* ]] || {
  print -u2 "non-application casks are missing their category"
  exit 1
}
[[ "$OUTPUT" != *"Formula (CLI/library):"* ]] || {
  print -u2 "old formula-first layout is still present"
  exit 1
}

grep -q "curses.wrapper" "$SCRIPT" || {
  print -u2 "curses UI implementation is missing"
  exit 1
}

grep -q "stdscr.keypad(True)" "$SCRIPT" || {
  print -u2 "terminal keypad mode is not enabled"
  exit 1
}

python3 - "$SCRIPT" <<'PY'
import os
import pty
import select
import sys
import time

pid, master = pty.fork()
if pid == 0:
    os.execv(sys.argv[1], [sys.argv[1]])

deadline = time.time() + 20
next_navigation = time.time() + 5
next_quit = time.time() + 6
sent_navigation = False
sent_quit = False
finished = False
status = 0
screen_output = bytearray()
while time.time() < deadline:
    ready, _, _ = select.select([master], [], [], 0.25)
    if ready:
        try:
            screen_output.extend(os.read(master, 4096))
        except OSError:
            break
    if not sent_navigation and time.time() >= next_navigation:
        try:
            os.write(master, b"\x1b[B")
        except OSError:
            pass
        sent_navigation = True
    if not sent_quit and time.time() >= next_quit:
        try:
            os.write(master, b"q")
        except OSError:
            pass
        sent_quit = True
    waited_pid, status = os.waitpid(pid, os.WNOHANG)
    if waited_pid == pid:
        finished = True
        break

if not finished:
    os.kill(pid, 9)
    os.waitpid(pid, 0)
    raise SystemExit("curses mode did not exit within timeout")
if not os.WIFEXITED(status) or os.WEXITSTATUS(status) != 0:
    raise SystemExit("curses mode did not exit cleanly after q")
if b"0 item utama" in screen_output:
    raise SystemExit("curses UI reports zero selectable items")
PY

print "Homebrew curses table behavior is valid"
