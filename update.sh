#!/bin/zsh
# ==============================
# 🔧 System Update Script
# Stack: macOS, Homebrew, pyenv, fnm, fvm, rvm, gcloud, Docker
# ==============================
START_TIME=$SECONDS
echo "🚀 Memulai update: $(date '+%Y-%m-%d %H:%M:%S')"
echo "======================================================"

# ==============================
# 🍺 Homebrew
# ==============================
echo "\n🍺 Homebrew..."
brew update && brew upgrade && brew cleanup
echo "✅ Homebrew selesai"

# ==============================
# ☁️ Google Cloud SDK
# ==============================
if command -v gcloud &>/dev/null; then
    echo "\n☁️  gcloud components..."
    gcloud components update --quiet
    echo "✅ gcloud selesai"
fi

# ==============================
# 🍵 Node.js (fnm)
# ==============================
if command -v fnm &>/dev/null; then
    echo "\n🟢 Node.js via fnm..."

    # FIX: 'fnm current' (bukan 'fnm current-version' yang tidak valid)
    CURRENT_VERSION=$(fnm current 2>/dev/null || echo "")

    # FIX: sort -V -r menggantikan flag 'desc' yang tidak dikenal fnm
    LTS_VERSION=$(fnm ls-remote --lts 2>/dev/null \
        | grep -E "^v[0-9]" | sort -V -r | head -1 | awk '{print $1}' || echo "")

    if [[ -n "$LTS_VERSION" ]]; then
        if [[ "$CURRENT_VERSION" != "$LTS_VERSION" ]]; then
            echo "📦 Upgrade Node.js LTS: $CURRENT_VERSION → $LTS_VERSION"
            fnm install "$LTS_VERSION" 2>/dev/null || true
            fnm default "$LTS_VERSION" 2>/dev/null || true
        else
            echo "✅ Node.js LTS sudah terbaru: $CURRENT_VERSION"
        fi
    fi

    if command -v npm &>/dev/null; then
        npm install -g npm@latest 2>/dev/null || true
        npm -g update 2>/dev/null || true
        command -v ng &>/dev/null && npm -g install @angular/cli@latest 2>/dev/null || true
        npm -g install @continuedev/cli neovim --force 2>/dev/null || true
    fi
    echo "✅ fnm selesai"
fi

# ==============================
# 🐦 Flutter (fvm)
# ==============================
if command -v fvm &>/dev/null; then
    echo "\n🐦 Flutter via fvm..."
    brew upgrade fvm 2>/dev/null || true

    # Update semua Flutter versi yang sudah terinstall
    # fvm list pakai tabel dengan separator Unicode │ (U+2502) dan ANSI codes
    # Strip ANSI dulu, lalu pakai │ sebagai delimiter awk, ambil kolom Version saja
    local FVM_SEP=$'\xe2\x94\x82'
    fvm list 2>/dev/null | \
        sed $'s/\033\[[0-9;]*[mGKH]//g' | \
        awk -F"$FVM_SEP" '$2 ~ /^[[:space:]]*[0-9]+\.[0-9]+\.[0-9]+[[:space:]]*$/ {gsub(/[[:space:]]/, "", $2); print $2}' | \
    while read -r ver; do
        [[ -z "$ver" ]] && continue
        echo "  📦 Upgrading Flutter $ver..."
        fvm install "$ver" 2>/dev/null || true
    done

    # Update Dart pub global packages
    if command -v dart &>/dev/null; then
        echo "📦 Updating Dart pub global packages..."
        dart pub global activate dart_style 2>/dev/null || true
    fi
    echo "✅ fvm selesai"
fi

# ==============================
# 🐍 Python (pyenv + uv)
# ==============================
if command -v pyenv &>/dev/null; then
    echo "\n🐍 Python via pyenv..."
    # Update pyenv itself via homebrew
    brew upgrade pyenv 2>/dev/null || true

    # Update pip pada semua versi python yang terinstall
    pyenv versions --bare 2>/dev/null | while read -r pyver; do
        echo "  📦 Updating pip for Python $pyver..."
        "$HOME/.pyenv/versions/$pyver/bin/pip" install --upgrade pip 2>/dev/null || true
    done
    echo "✅ pyenv selesai"
fi

if command -v uv &>/dev/null; then
    echo "\n🐍 uv (Python package manager)..."
    uv self update 2>/dev/null || brew upgrade uv 2>/dev/null || true
    uv tool upgrade --all 2>/dev/null || true
    echo "✅ uv selesai"
fi

# ==============================
# 💎 Ruby (rvm + gems)
# ==============================
if command -v rvm &>/dev/null; then
    echo "\n💎 Ruby via rvm..."
    # Load rvm
    [[ -s "$HOME/.rvm/scripts/rvm" ]] && source "$HOME/.rvm/scripts/rvm"

    rvm get stable 2>/dev/null || true
    gem update --system 2>/dev/null || true

    # FIX: 'bundle update' tanpa --bundler agar semua gems ter-update (bukan hanya bundler)
    if command -v bundle &>/dev/null; then
        bundle update 2>/dev/null || true
    fi

    # CocoaPods
    if command -v pod &>/dev/null; then
        echo "📦 Mengecek CocoaPods..."
        gem install cocoapods --no-document 2>/dev/null || true
        pod repo update 2>/dev/null || true
    fi
    echo "✅ Ruby/gems selesai"
fi

# ==============================
# 📦 pipx tools
# ==============================
if command -v pipx &>/dev/null; then
    echo "\n📦 pipx global tools..."
    pipx upgrade-all 2>/dev/null || true
    echo "✅ pipx selesai"
fi

# ==============================
# 🍎 Mac App Store (mas)
# ==============================
if command -v mas &>/dev/null; then
    echo "\n🍎 Mac App Store apps..."
    mas upgrade 2>/dev/null || true
    echo "✅ mas selesai"
fi

# ==============================
# ✅ Summary
# ==============================
ELAPSED=$(( SECONDS - START_TIME ))
echo "\n======================================================"
echo "✅ Semua update selesai dalam ${ELAPSED}s — $(date '+%Y-%m-%d %H:%M:%S')"
echo "======================================================"
