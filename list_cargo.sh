#!/bin/zsh

# Daftar binary/aplikasi yang dipasang melalui `cargo install`.
print "🦀 Rust Cargo — daftar package ter-install"
print "==========================================="

if ! command -v cargo >/dev/null 2>&1; then
  print "⚠️  cargo tidak ditemukan."
  exit 0
fi

print "cargo: $(cargo --version 2>/dev/null || print 'versi tidak diketahui')"
print ""
cargo install --list
