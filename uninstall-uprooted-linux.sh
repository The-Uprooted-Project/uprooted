#!/bin/bash
# Uprooted Linux Uninstaller v0.1.9
# Removes all Uprooted files and restores original Root behavior.

set -euo pipefail

INSTALL_DIR="$HOME/.local/share/uprooted"
PROFILE_DIR="$HOME/.local/share/Root Communications/Root/profile/default"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log()   { echo -e "${GREEN}[+]${NC} $1"; }
warn()  { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[-]${NC} $1"; }

echo ""
echo "  Uprooted Linux Uninstaller"
echo "  ──────────────────────────"
echo ""

# ── Check if Root is running ──

if pgrep -x Root &>/dev/null; then
    error "Root is currently running. Please close it first."
    exit 1
fi

# ── Restore HTML files ──

restored=0
if [[ -d "$PROFILE_DIR" ]]; then
    # Find and restore backups
    while IFS= read -r -d '' backup; do
        original="${backup%.uprooted-backup}"
        if [[ -f "$backup" ]]; then
            cp "$backup" "$original"
            rm "$backup"
            restored=$((restored + 1))
            log "Restored: $(basename "$(dirname "$original")")/index.html"
        fi
    done < <(find "$PROFILE_DIR" -name "*.uprooted-backup" -print0 2>/dev/null)

    # If no backups found, try to remove our injected tags
    if [[ $restored -eq 0 ]]; then
        while IFS= read -r -d '' html; do
            if grep -q "uprooted-preload" "$html" 2>/dev/null; then
                sed -i '/uprooted-preload/d' "$html"
                sed -i '/uprooted\.css/d' "$html"
                restored=$((restored + 1))
                log "Cleaned: $(basename "$(dirname "$html")")/index.html"
            fi
        done < <(find "$PROFILE_DIR" -name "index.html" -print0 2>/dev/null)
    fi
fi

if [[ $restored -eq 0 ]]; then
    warn "No HTML files needed restoration"
else
    log "$restored HTML file(s) restored"
fi

# ── Remove environment.d config ──

env_conf="$HOME/.config/environment.d/uprooted.conf"
if [[ -f "$env_conf" ]]; then
    rm "$env_conf"
    log "Removed environment.d/uprooted.conf"
fi

# ── Remove .desktop file ──

desktop_file="$HOME/.local/share/applications/root-uprooted.desktop"
if [[ -f "$desktop_file" ]]; then
    rm "$desktop_file"
    log "Removed .desktop file"
fi

# ── Remove install directory ──

if [[ -d "$INSTALL_DIR" ]]; then
    rm -rf "$INSTALL_DIR"
    log "Removed $INSTALL_DIR"
else
    warn "Install directory not found (already removed?)"
fi

echo ""
log "Uprooted has been uninstalled."
log "Root will run normally on next launch."
echo ""
