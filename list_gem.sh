#!/bin/zsh

# Daftar gem Ruby yang terpasang pada environment Ruby aktif.
print "💎 RubyGems — daftar gem lokal"
print "=============================="

if ! command -v gem >/dev/null 2>&1; then
  print "⚠️  gem tidak ditemukan."
  exit 0
fi

print "Ruby: $(ruby --version 2>/dev/null || print 'versi tidak diketahui')"
print "RubyGems: $(gem --version 2>/dev/null || print 'versi tidak diketahui')"
print ""
gem list --local
