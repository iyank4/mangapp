#!/bin/zsh

# Daftar package npm global, yang biasanya berisi aplikasi CLI.
print "📦 npm — daftar package global"
print "=============================="

if ! command -v npm >/dev/null 2>&1; then
  print "⚠️  npm tidak ditemukan."
  exit 0
fi

print "Node.js: $(node --version 2>/dev/null || print 'versi tidak diketahui')"
print "npm:     $(npm --version 2>/dev/null || print 'versi tidak diketahui')"
print "Prefix:  $(npm prefix --global 2>/dev/null || print 'tidak diketahui')"
print ""
npm list --global --depth=0
