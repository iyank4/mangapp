#!/bin/zsh

# Daftar aplikasi CLI Python yang dipasang melalui `uv tool install`.
print "🐍 uv — daftar tool global"
print "=========================="

if ! command -v uv >/dev/null 2>&1; then
  print "⚠️  uv tidak ditemukan."
  exit 0
fi

print "uv: $(uv --version 2>/dev/null || print 'versi tidak diketahui')"
print ""
uv tool list
