#!/bin/zsh
# ==============================
# 🧹 Storage Cleanup Script
# Setiap cache dijelaskan sebelum dihapus.
# Jalankan tanpa argumen untuk preview (dry-run).
# Jalankan dengan --clean untuk benar-benar menghapus.
# ==============================

# Pastikan tools standar selalu tersedia (PATH bisa berbeda saat dijalankan non-interaktif)
export PATH="/usr/bin:/bin:/usr/sbin:/sbin:$PATH"

DRY_RUN=true
TOTAL_FREED=0

if [[ "$1" == "--clean" ]]; then
    DRY_RUN=false
    echo "⚠️  Mode: HAPUS SUNGGUHAN"
else
    echo "👁️  Mode: DRY-RUN (preview saja, tidak ada yang dihapus)"
    echo "    Jalankan dengan --clean untuk menghapus.\n"
fi

# Helper: tampilkan ukuran dan tanya konfirmasi (jika --clean)
check_and_clean() {
    local label="$1"
    local dirpath="$2"   # NOTE: tidak pakai "path" — di zsh "path" adalah tied array ke $PATH
    local description="$3"
    local cmd="$4"

    if [[ ! -e "$dirpath" && -z "$cmd" ]]; then
        echo "  ⬜  $label — tidak ditemukan, skip"
        return
    fi

    local size=""
    if [[ -n "$dirpath" && -e "$dirpath" ]]; then
        local -a du_out
        du_out=( $(du -sh "$dirpath" 2>/dev/null) )
        size=${du_out[1]:-}
    fi

    echo "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📁 $label${size:+ ($size)}"
    echo "   📖 $description"

    if [[ "$DRY_RUN" == true ]]; then
        echo "   ➡️  [dry-run] Tidak dihapus"
        return
    fi

    echo -n "   🗑️  Hapus sekarang? [y/N] "
    read -r answer
    if [[ "$answer" =~ ^[Yy]$ ]]; then
        if [[ -n "$cmd" ]]; then
            eval "$cmd"
        else
            rm -rf "$dirpath"
        fi
        echo "   ✅ Dihapus"
    else
        echo "   ⏭️  Dilewati"
    fi
}

echo "======================================================"
echo "🧹 Storage Cleanup — $(date '+%Y-%m-%d %H:%M:%S')"
echo "======================================================"

# ==============================
# 🍺 HOMEBREW
# ==============================
echo "\n\n🍺 ── HOMEBREW ──────────────────────────────────────"

check_and_clean \
    "Homebrew download cache" \
    "$(brew --cache)" \
    "File installer (.tar.gz, .dmg, .pkg) yang diunduh saat 'brew install'.
   AMAN dihapus: homebrew akan re-download jika perlu install ulang.
   Biasanya 500MB–3GB kalau lama tidak dibersihkan." \
    "brew cleanup --prune=all 2>/dev/null; rm -rf '$(brew --cache)'"

# ==============================
# 📦 NODE.JS & NPM
# ==============================
echo "\n\n📦 ── NODE.JS & NPM ─────────────────────────────────"

check_and_clean \
    "npm cache" \
    "$HOME/.npm/_cacache" \
    "Cache paket npm yang sudah diunduh.
   AMAN dihapus: npm akan re-download dari registry saat install berikutnya.
   Bisa sangat besar (1–5GB) kalau sering install package."

check_and_clean \
    "fnm node versions lama" \
    "$HOME/.local/share/fnm" \
    "Semua versi Node.js yang diinstall via fnm.
   HATI-HATI: hapus hanya versi yang tidak lagi digunakan.
   Cek dulu dengan: fnm list
   Hapus versi spesifik dengan: fnm uninstall <version>" \
    "echo '  ℹ️  Gunakan: fnm list  lalu: fnm uninstall <version>'"

# ==============================
# 🐍 PYTHON & PIP
# ==============================
echo "\n\n🐍 ── PYTHON & PIP ──────────────────────────────────"

check_and_clean \
    "pip download cache" \
    "$HOME/Library/Caches/pip" \
    "Wheel dan source dist yang di-cache oleh pip.
   AMAN dihapus: pip akan re-download saat install berikutnya.
   Biasanya 200MB–2GB."

check_and_clean \
    "pyenv build cache (source tarball Python)" \
    "$HOME/.pyenv/cache" \
    "File .tar.xz source Python yang diunduh saat 'pyenv install'.
   AMAN dihapus: pyenv akan re-download jika install versi Python baru.
   Biasanya 50–500MB."

check_and_clean \
    "pyenv versi Python yang tidak digunakan" \
    "$HOME/.pyenv/versions" \
    "Semua versi Python yang terinstall.
   HATI-HATI: cek dulu versi aktif dengan 'pyenv versions'.
   Hapus versi spesifik dengan: pyenv uninstall <version>" \
    "echo '  ℹ️  Gunakan: pyenv versions  lalu: pyenv uninstall <version>'"

check_and_clean \
    "uv cache" \
    "$HOME/.cache/uv" \
    "Cache wheel dan metadata paket yang diunduh uv.
   AMAN dihapus: uv akan re-download saat dibutuhkan.
   Jalankan juga: uv cache clean" \
    "uv cache clean 2>/dev/null; rm -rf '$HOME/.cache/uv'"

# ==============================
# 🐦 FLUTTER & DART
# ==============================
echo "\n\n🐦 ── FLUTTER & DART ────────────────────────────────"

check_and_clean \
    "Dart pub-cache (package cache global)" \
    "$HOME/.pub-cache" \
    "Semua paket Dart/Flutter yang pernah di-download (pub.dev).
   AMAN dihapus: akan re-download saat 'flutter pub get' di project.
   TAPI: bisa lambat karena semua project perlu download ulang.
   Biasanya 500MB–3GB."

check_and_clean \
    "Flutter SDK lama via fvm" \
    "$HOME/fvm/versions" \
    "Semua Flutter SDK versi yang diinstall via fvm.
   HATI-HATI: cek versi yang masih dipakai di project (file .fvmrc).
   Hapus versi spesifik: fvm remove <version>
   List versi: fvm list" \
    "echo '  ℹ️  Gunakan: fvm list  lalu: fvm remove <version>'"

# ==============================
# 💎 RUBY & GEMS
# ==============================
echo "\n\n💎 ── RUBY & GEMS ───────────────────────────────────"

check_and_clean \
    "RubyGems cache" \
    "$HOME/.gem/cache" \
    "File .gem yang di-cache saat install gems.
   AMAN dihapus: gem akan re-download jika diperlukan.
   Jalankan juga: gem cleanup"

check_and_clean \
    "CocoaPods cache" \
    "$HOME/Library/Caches/CocoaPods" \
    "Source pod yang sudah diunduh.
   AMAN dihapus: 'pod install' akan re-download.
   Bisa sangat besar (1–5GB) kalau banyak iOS project.
   Jalankan juga: pod cache clean --all"

# ==============================
# 🍎 XCODE
# ==============================
echo "\n\n🍎 ── XCODE ─────────────────────────────────────────"

check_and_clean \
    "Xcode DerivedData" \
    "$HOME/Library/Developer/Xcode/DerivedData" \
    "Build artifacts dari semua project Xcode (object files, index, dll).
   SANGAT AMAN dihapus: Xcode akan rebuild saat project dibuka lagi.
   SANGAT BESAR: biasanya 5–30GB!
   Project tidak akan rusak, tapi build pertama setelah ini akan lebih lama."

check_and_clean \
    "iOS Simulator cache" \
    "$HOME/Library/Developer/CoreSimulator/Caches" \
    "Cache simulator iOS yang sudah tidak terpakai.
   AMAN dihapus, tapi bisa memperlambat launch simulator pertama kali.
   Biasanya 1–5GB."

check_and_clean \
    "Xcode Archives (IPA/dSYM lama)" \
    "$HOME/Library/Developer/Xcode/Archives" \
    "⚠️  HATI-HATI: ini adalah build archive (IPA) dan dSYM files.
   dSYM dibutuhkan untuk men-symbolicate crash report dari app yang sudah dirilis.
   JANGAN hapus jika masih ada versi app di App Store yang mungkin crash.
   Lebih aman: hapus manual versi yang sudah sangat lama." \
    "echo '  ⚠️  Buka Xcode → Window → Organizer → Archives untuk hapus manual.'"

check_and_clean \
    "Xcode Simulator Runtimes lama" \
    "$HOME/Library/Developer/CoreSimulator/Volumes" \
    "iOS/watchOS/tvOS simulator runtime yang mungkin sudah outdated.
   Hapus yang tidak dipakai: xcrun simctl runtime list
   Hapus: xcrun simctl runtime delete <identifier>" \
    "echo '  ℹ️  Gunakan: xcrun simctl runtime list  untuk cek yang bisa dihapus'"

# ==============================
# 🤖 ANDROID
# ==============================
echo "\n\n🤖 ── ANDROID ───────────────────────────────────────"

check_and_clean \
    "Gradle build cache" \
    "$HOME/.gradle/caches" \
    "Cache build Gradle untuk semua project Android.
   AMAN dihapus: akan di-regenerate saat build berikutnya (tapi build pertama lebih lama).
   Biasanya 1–10GB."

check_and_clean \
    "Android SDK cache" \
    "$HOME/.android/cache" \
    "Cache internal Android SDK tools.
   AMAN dihapus."

# ==============================
# 🐳 DOCKER
# ==============================
echo "\n\n🐳 ── DOCKER ────────────────────────────────────────"

if command -v docker &>/dev/null && docker info &>/dev/null 2>&1; then
    echo "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📁 Docker usage:"
    docker system df 2>/dev/null || true
    echo "   📖 Docker menyimpan: images, stopped containers, dangling volumes, build cache."
    echo "   'docker system prune' hanya hapus yang TIDAK DIGUNAKAN (container stopped, image dangling)."
    echo "   'docker system prune -a' juga hapus image yang tidak sedang dipakai container manapun."

    if [[ "$DRY_RUN" == false ]]; then
        echo -n "   🗑️  Hapus resources Docker yang tidak terpakai? (prune, AMAN) [y/N] "
        read -r answer
        if [[ "$answer" =~ ^[Yy]$ ]]; then
            docker system prune -f
            echo "   ✅ Docker pruned"
        fi
    else
        echo "   ➡️  [dry-run] Tidak dihapus"
    fi
else
    echo "  ⬜  Docker tidak berjalan, skip"
fi

# ==============================
# 📱 LM STUDIO
# ==============================
echo "\n\n🤖 ── LM STUDIO ─────────────────────────────────────"
check_and_clean \
    "LM Studio model cache" \
    "$HOME/.cache/lm-studio" \
    "Cache metadata LM Studio (bukan model itu sendiri).
   Model LLM sendiri biasanya ada di: ~/LM Studio/models/
   AMAN dihapus, tapi model yang besar (3–70GB) sebaiknya dikelola manual di LM Studio app." \
    "echo '  ℹ️  Buka LM Studio → My Models untuk kelola model yang tidak digunakan'"

# ==============================
# 🗂️ SYSTEM CACHE UMUM
# ==============================
echo "\n\n🗂️  ── SYSTEM CACHE UMUM ────────────────────────────"

check_and_clean \
    "macOS user caches (~/Library/Caches)" \
    "$HOME/Library/Caches" \
    "Cache berbagai aplikasi (Xcode, Simulator, browser, dsb.).
   HATI-HATI: jangan hapus folder ini secara keseluruhan!
   Lebih aman: hapus subfolder spesifik per aplikasi.
   Subfolder terbesar biasanya: Xcode, CocoaPods, pip, Google, Homebrew." \
    "echo '  ℹ️  Gunakan: du -sh ~/Library/Caches/* | sort -h -r | head -20'"

# ==============================
# ✅ SELESAI
# ==============================
echo "\n\n======================================================"
if [[ "$DRY_RUN" == true ]]; then
    echo "👁️  Dry-run selesai. Jalankan dengan --clean untuk hapus sungguhan:"
    echo "    zsh ~/cleanup.sh --clean"
else
    echo "✅ Cleanup selesai — $(date '+%Y-%m-%d %H:%M:%S')"
fi

echo "\n💡 Tips storage lainnya:"
echo "   • Cek folder terbesar: du -sh ~/* | sort -h -r | head -20"
echo "   • Cek subfolder Library: du -sh ~/Library/*/ | sort -h -r | head -20"
echo "   • Cek Xcode DerivedData per project: du -sh ~/Library/Developer/Xcode/DerivedData/*"
echo "======================================================"
