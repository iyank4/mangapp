#!/bin/zsh

# Daftar package Python dari command pip dan/atau pip3 yang tersedia.
print "🐍 Python pip/pip3 — daftar package"
print "===================================="

typeset found=false

for pip_command in pip pip3; do
  if command -v "$pip_command" >/dev/null 2>&1; then
    found=true
    print ""
    print "[$pip_command: $(command -v "$pip_command")]"
    "$pip_command" list
  fi
done

if [[ "$found" == false ]]; then
  print "⚠️  pip maupun pip3 tidak ditemukan."
fi
