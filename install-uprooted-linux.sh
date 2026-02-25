#!/bin/bash
# Uprooted Linux Installer
# Standalone bash installer for systems without the GUI installer.
#
# Usage: ./install-uprooted-linux.sh [--root-path /path/to/Root.AppImage]
#        ./install-uprooted-linux.sh --channel canary
#        ./install-uprooted-linux.sh --local        (deploy from repo build output, skip download)
#        ./install-uprooted-linux.sh --uninstall
#        ./install-uprooted-linux.sh --repair
#        ./install-uprooted-linux.sh --diagnose
#        ./install-uprooted-linux.sh --desktop      (also create a .desktop file)
#
# This script:
# 1. Finds Root.AppImage (or uses --root-path)
# 2. Downloads (or copies local) profiler + hook artifacts
# 3. Deploys to ~/.local/share/uprooted/
# 4. Creates a wrapper script with CLR profiler env vars
# 5. Patches HTML files in Root's profile directory
# 6. Adds env vars to ~/.profile as fallback for non-systemd sessions

set -euo pipefail

# When launched by double-clicking in a file manager, the terminal auto-closes
# on exit. Trap errors so the user can read what went wrong before it vanishes.
trap 'echo ""; error "Script failed (line $LINENO). See error above."; echo ""; read -rp "Press Enter to exit..." || true' ERR

INSTALL_DIR="$HOME/.local/share/uprooted"
PROFILE_DIR="$HOME/.local/share/Root Communications/Root/profile/default"
PROFILER_GUID="{D1A6F5A0-1234-4567-89AB-CDEF01234567}"
VERSION="0.5.1"

# Default channel: pre-release versions (dev/alpha/beta/rc) use canary channel
if [[ "$VERSION" == *-dev* || "$VERSION" == *-alpha* || "$VERSION" == *-beta* || "$VERSION" == *-rc* ]]; then
    CHANNEL="canary"
else
    CHANNEL="stable"
fi
ROOT_EXEC=""        # actual binary/AppRun to exec (may differ from ROOT_PATH)
SQUASHFS_ROOT=""    # set when using an extracted AppImage

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m' # No Color

log()   { echo -e "${GREEN}[+]${NC} $1"; }
warn()  { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[-]${NC} $1"; }
die()   { error "$1"; exit 1; }

# ── Channel → GitHub repo mapping ──

channel_repo() {
    case "$CHANNEL" in
        stable) echo "The-Uprooted-Project/uprooted" ;;
        canary) echo "The-Uprooted-Project/uprooted-canary" ;;
        dev)    echo "The-Uprooted-Project/uprooted-private" ;;
        *)      die "Unknown channel: $CHANNEL (use stable, canary, or dev)" ;;
    esac
}

# ── Resolve latest release version from GitHub API ──

resolve_latest_version() {
    if ! command -v curl &>/dev/null; then
        warn "curl not found, using bundled version v$VERSION"
        return
    fi

    local repo
    repo=$(channel_repo)

    # /releases/latest only returns non-prerelease; canary/dev are always prerelease
    local api_url
    if [[ "$CHANNEL" == "stable" ]]; then
        api_url="https://api.github.com/repos/${repo}/releases/latest"
    else
        api_url="https://api.github.com/repos/${repo}/releases?per_page=1"
    fi

    local curl_opts=(-sL --max-time 10)
    if [[ "$CHANNEL" == "dev" && -n "${GITHUB_TOKEN:-}" ]]; then
        curl_opts+=(-H "Authorization: Bearer $GITHUB_TOKEN")
    fi

    local response
    response=$(curl "${curl_opts[@]}" "$api_url" 2>/dev/null) || {
        warn "Could not reach GitHub API, using bundled version v$VERSION"
        return
    }

    local tag
    tag=$(echo "$response" | grep -o '"tag_name"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | grep -o '"v[^"]*"' | tr -d '"')

    if [[ -z "$tag" ]]; then
        if [[ "$CHANNEL" == "dev" ]]; then
            warn "Could not fetch latest dev release (is GITHUB_TOKEN set?), using bundled v$VERSION"
        else
            warn "Could not parse latest version from GitHub API, using bundled v$VERSION"
        fi
        return
    fi

    local latest="${tag#v}"
    if [[ "$latest" != "$VERSION" ]]; then
        log "Latest $CHANNEL release: v$latest (script bundled: v$VERSION)"
        VERSION="$latest"
    fi
}

# ── Diagnose function ──

run_diagnose() {
    echo ""
    echo "  Uprooted Diagnostics v$VERSION"
    echo "  ─────────────────────────────"
    echo ""

    # 1. Check env vars in current shell
    log "Checking environment variables in current session..."
    local env_ok=true
    if [[ "${DOTNET_ENABLE_PROFILING:-}" == "1" ]]; then
        log "  DOTNET_ENABLE_PROFILING=1"
    else
        error "  DOTNET_ENABLE_PROFILING is not set (or not '1')"
        env_ok=false
    fi
    if [[ -n "${DOTNET_PROFILER:-}" ]]; then
        log "  DOTNET_PROFILER=$DOTNET_PROFILER"
    else
        error "  DOTNET_PROFILER is not set"
        env_ok=false
    fi
    if [[ -n "${DOTNET_PROFILER_PATH:-}" ]]; then
        if [[ -f "$DOTNET_PROFILER_PATH" ]]; then
            log "  DOTNET_PROFILER_PATH=$DOTNET_PROFILER_PATH (exists)"
        else
            warn "  DOTNET_PROFILER_PATH=$DOTNET_PROFILER_PATH (FILE NOT FOUND)"
            env_ok=false
        fi
    else
        error "  DOTNET_PROFILER_PATH is not set"
        env_ok=false
    fi
    if [[ "${DOTNET_ReadyToRun:-}" == "0" ]]; then
        log "  DOTNET_ReadyToRun=0"
    else
        warn "  DOTNET_ReadyToRun is not set to '0' (optional but recommended)"
    fi
    # Legacy check (CORECLR_ prefix for .NET 8/9)
    if [[ "${CORECLR_ENABLE_PROFILING:-}" == "1" ]]; then
        log "  CORECLR_ENABLE_PROFILING=1 (legacy)"
    else
        warn "  CORECLR_ENABLE_PROFILING not set (legacy, optional for .NET 10+)"
    fi

    if [[ "$env_ok" == "false" ]]; then
        echo ""
        warn "Env vars are NOT active in this shell session."
        warn "Root launched from this session will NOT load Uprooted."
        warn "Fix: log out and back in, or use the wrapper script:"
        warn "  $INSTALL_DIR/launch-root.sh"
    else
        echo ""
        log "Env vars are active in this session."
    fi

    # 2. Check config files
    echo ""
    log "Checking configuration files..."
    local env_conf="$HOME/.config/environment.d/uprooted.conf"
    if [[ -f "$env_conf" ]]; then
        log "  environment.d/uprooted.conf: exists"
    else
        warn "  environment.d/uprooted.conf: missing"
    fi

    local wrapper="$INSTALL_DIR/launch-root.sh"
    if [[ -f "$wrapper" ]]; then
        log "  launch-root.sh: exists"
    else
        warn "  launch-root.sh: missing"
    fi

    local desktop="$HOME/.local/share/applications/root-uprooted.desktop"
    if [[ -f "$desktop" ]]; then
        log "  root-uprooted.desktop: exists"
        local exec_line
        exec_line=$(grep "^Exec=" "$desktop" 2>/dev/null || true)
        if [[ -n "$exec_line" ]]; then
            log "    $exec_line"
        fi
    else
        warn "  root-uprooted.desktop: missing (create with --desktop flag)"
    fi

    local plasma_env="$HOME/.config/plasma-workspace/env/uprooted.sh"
    if [[ -f "$plasma_env" ]]; then
        log "  plasma-workspace/env/uprooted.sh: exists (KDE Plasma)"
    elif is_kde; then
        warn "  plasma-workspace/env/uprooted.sh: missing (KDE detected — run repair)"
    fi

    if grep -q "DOTNET_ENABLE_PROFILING" "$HOME/.profile" 2>/dev/null; then
        log "  ~/.profile: contains Uprooted env vars"
    else
        warn "  ~/.profile: does not contain Uprooted env vars"
    fi

    # 3. Check deployed files
    echo ""
    log "Checking deployed files..."
    local files=("libuprooted_profiler.so" "UprootedHook.dll" "UprootedHook.deps.json" "uprooted-preload.js" "uprooted.css")
    for f in "${files[@]}"; do
        if [[ -f "$INSTALL_DIR/$f" ]]; then
            log "  $f: exists"
        else
            error "  $f: MISSING"
        fi
    done

    # 4. Check for running Root process
    echo ""
    log "Checking for running Root process..."
    local root_pids
    root_pids=$(pgrep -a "Root" 2>/dev/null || true)
    if [[ -n "$root_pids" ]]; then
        log "  Root is running:"
        echo "$root_pids" | while IFS= read -r line; do
            log "    PID $line"
        done

        # Check /proc/PID/exe for each Root process
        for pid in $(pgrep "Root" 2>/dev/null || true); do
            local exe_path
            exe_path=$(readlink "/proc/$pid/exe" 2>/dev/null || echo "(unreadable)")
            log "    /proc/$pid/exe -> $exe_path"

            # Check if DOTNET_ENABLE_PROFILING is set in the process
            if [[ -r "/proc/$pid/environ" ]]; then
                local proc_env
                proc_env=$(tr '\0' '\n' < "/proc/$pid/environ")
                if echo "$proc_env" | grep -q "DOTNET_ENABLE_PROFILING=1"; then
                    log "    Process has DOTNET_ENABLE_PROFILING=1"
                else
                    warn "    Process does NOT have DOTNET_ENABLE_PROFILING set"
                fi
                if echo "$proc_env" | grep -q "CORECLR_ENABLE_PROFILING=1"; then
                    log "    Process has CORECLR_ENABLE_PROFILING=1 (legacy)"
                else
                    warn "    Process does NOT have CORECLR_ENABLE_PROFILING set (legacy, optional)"
                fi
            else
                warn "    Cannot read /proc/$pid/environ (permission denied)"
            fi
        done
    else
        warn "  Root is not currently running"
    fi

    # 5. Check logs
    echo ""
    log "Checking log files..."
    local profiler_log="$INSTALL_DIR/profiler.log"
    if [[ -f "$profiler_log" ]]; then
        log "  profiler.log exists ($(wc -l < "$profiler_log") lines)"
        log "  Last 10 lines:"
        tail -10 "$profiler_log" | while IFS= read -r line; do
            echo "    $line"
        done
    else
        warn "  profiler.log: not found (profiler has never loaded)"
    fi

    local hook_log="$PROFILE_DIR/uprooted-hook.log"
    if [[ -f "$hook_log" ]]; then
        log "  uprooted-hook.log exists ($(wc -l < "$hook_log") lines)"
        log "  Last 10 lines:"
        tail -10 "$hook_log" | while IFS= read -r line; do
            echo "    $line"
        done
    else
        warn "  uprooted-hook.log: not found (hook has never loaded)"
    fi

    # 6. Check HTML patches
    echo ""
    log "Checking HTML patches..."
    if [[ -d "$PROFILE_DIR" ]]; then
        local html_files=()
        if [[ -f "$PROFILE_DIR/WebRtcBundle/index.html" ]]; then
            html_files+=("$PROFILE_DIR/WebRtcBundle/index.html")
        fi
        for app_dir in "$PROFILE_DIR/RootApps"/*/; do
            if [[ -f "${app_dir}index.html" ]]; then
                html_files+=("${app_dir}index.html")
            fi
        done

        if [[ ${#html_files[@]} -eq 0 ]]; then
            warn "  No HTML files found in profile directory"
        else
            for html in "${html_files[@]}"; do
                local name
                name="$(basename "$(dirname "$html")")/index.html"
                if grep -qE "(uprooted:start|uprooted-preload|<!-- uprooted -->)" "$html" 2>/dev/null; then
                    log "  $name: patched"
                else
                    error "  $name: NOT patched"
                fi
            done
        fi
    else
        warn "  Profile directory not found: $PROFILE_DIR"
        warn "  Launch Root once to generate it."
    fi

    echo ""
    log "Diagnostics complete."
    echo ""
}

# ── Uninstall function ──

run_uninstall() {
    echo ""
    echo "  Uprooted Uninstaller v$VERSION"
    echo "  ──────────────────────────────"
    echo ""

    # 1. Strip HTML patches
    log "Removing HTML patches..."
    if [[ -d "$PROFILE_DIR" ]]; then
        local html_files=()
        if [[ -f "$PROFILE_DIR/WebRtcBundle/index.html" ]]; then
            html_files+=("$PROFILE_DIR/WebRtcBundle/index.html")
        fi
        for app_dir in "$PROFILE_DIR/RootApps"/*/; do
            if [[ -f "${app_dir}index.html" ]]; then
                html_files+=("${app_dir}index.html")
            fi
        done

        local stripped=0
        for html in "${html_files[@]}"; do
            if grep -qE "(uprooted:start|uprooted-preload|<!-- uprooted -->|__UPROOTED_SETTINGS__)" "$html" 2>/dev/null; then
                # Strip injection lines (markers, legacy markers, bare tags)
                local tmp="${html}.tmp"
                local inside_block=false
                while IFS= read -r line; do
                    if [[ "$line" == *"<!-- uprooted:start -->"* ]]; then
                        inside_block=true
                        continue
                    fi
                    if [[ "$line" == *"<!-- uprooted:end -->"* ]]; then
                        inside_block=false
                        continue
                    fi
                    if [[ "$inside_block" == true ]]; then
                        continue
                    fi
                    # Legacy marker
                    if [[ "$line" == *"<!-- uprooted -->"* ]]; then
                        continue
                    fi
                    # Bare uprooted tags (bash installer without markers)
                    if [[ "$line" == *"uprooted-preload"* ]] && [[ "$line" == *"<script"* || "$line" == *"</script"* ]]; then
                        continue
                    fi
                    if [[ "$line" == *"uprooted.css"* ]] && [[ "$line" == *"<link"* ]]; then
                        continue
                    fi
                    if [[ "$line" == *"__UPROOTED_SETTINGS__"* ]] && [[ "$line" == *"<script"* ]]; then
                        continue
                    fi
                    echo "$line"
                done < "$html" > "$tmp"
                mv "$tmp" "$html"

                # Remove backup if it exists
                rm -f "${html}.uprooted.bak"
                stripped=$((stripped + 1))
                log "  Stripped: $(basename "$(dirname "$html")")/index.html"
            fi
        done
        log "$stripped HTML file(s) cleaned"
    else
        warn "Profile directory not found, skipping HTML cleanup"
    fi

    # 2. Remove environment.d config
    local env_conf="$HOME/.config/environment.d/uprooted.conf"
    if [[ -f "$env_conf" ]]; then
        rm -f "$env_conf"
        log "Removed $env_conf"
    fi

    # 3. Remove KDE Plasma env script
    local plasma_env="$HOME/.config/plasma-workspace/env/uprooted.sh"
    if [[ -f "$plasma_env" ]]; then
        rm -f "$plasma_env"
        log "Removed $plasma_env"
    fi

    # 4. Clean env vars from ~/.profile
    if grep -qE "(DOTNET_ENABLE_PROFILING|CORECLR_ENABLE_PROFILING)" "$HOME/.profile" 2>/dev/null; then
        # Remove the Uprooted block from .profile
        local tmp="$HOME/.profile.tmp"
        local skip_block=false
        while IFS= read -r line; do
            if [[ "$line" == "# Uprooted"* ]] && [[ "$line" != *"preload"* ]]; then
                skip_block=true
                continue
            fi
            if [[ "$skip_block" == true ]]; then
                # Skip export lines that are part of the block
                if [[ "$line" == export\ DOTNET_* || "$line" == export\ CORECLR_* || -z "$line" ]]; then
                    continue
                fi
                skip_block=false
            fi
            echo "$line"
        done < "$HOME/.profile" > "$tmp"
        mv "$tmp" "$HOME/.profile"
        log "Cleaned Uprooted env vars from ~/.profile"
    fi

    # 5. Remove .desktop file (backwards compat -- clean up even if we no longer create by default)
    local desktop="$HOME/.local/share/applications/root-uprooted.desktop"
    if [[ -f "$desktop" ]]; then
        rm -f "$desktop"
        log "Removed .desktop file"
    fi

    # 6. Remove installed files
    if [[ -d "$INSTALL_DIR" ]]; then
        rm -rf "$INSTALL_DIR"
        log "Removed $INSTALL_DIR"
    fi

    echo ""
    log "Uninstall complete."
    log "Log out and back in to clear env vars from your session."
    echo ""
}

# ── Parse arguments ──

ROOT_PATH=""
MODE="install"
USE_LOCAL=false
CREATE_DESKTOP=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --root-path) ROOT_PATH="$2"; shift 2 ;;
        --channel)
            CHANNEL="$2"
            case "$CHANNEL" in
                stable|canary|dev) ;;
                *) die "Unknown channel: $CHANNEL (use stable, canary, or dev)" ;;
            esac
            shift 2
            ;;
        --diagnose)
            MODE="diagnose"
            shift
            ;;
        --uninstall)
            MODE="uninstall"
            shift
            ;;
        --repair)
            MODE="repair"
            shift
            ;;
        --local)
            USE_LOCAL=true
            shift
            ;;
        --desktop)
            CREATE_DESKTOP=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--root-path /path/to/Root.AppImage] [--desktop]"
            echo "       $0 --channel canary"
            echo "       $0 --local"
            echo "       $0 --uninstall"
            echo "       $0 --repair"
            echo "       $0 --diagnose"
            echo ""
            echo "Installs Uprooted client mod framework for Root Communications."
            echo ""
            echo "Options:"
            echo "  --root-path    Path to Root.AppImage (auto-detected if not given)"
            echo "  --local        Deploy from repo build output (skip download — for dev use)"
            echo "  --channel CH   Release channel: stable (default), canary, dev (requires GITHUB_TOKEN)"
            echo "  --desktop      Create a .desktop file for launching Root with Uprooted"
            echo "  --uninstall    Remove Uprooted completely (patches, env vars, files)"
            echo "  --repair       Re-deploy artifacts and re-patch HTML files"
            echo "  --diagnose     Check installation health and runtime state"
            echo "  --help         Show this help"
            exit 0
            ;;
        *) die "Unknown option: $1" ;;
    esac
done

# ── Find Root ──

find_root() {
    if [[ -n "$ROOT_PATH" ]]; then
        if [[ -f "$ROOT_PATH" ]]; then
            log "Using Root at: $ROOT_PATH"
            return 0
        else
            die "Root not found at: $ROOT_PATH"
        fi
    fi

    # 1. Exact well-known paths (fastest check)
    local candidates=(
        "$HOME/Applications/Root.AppImage"
        "$HOME/Applications/root.appimage"
        "$HOME/AppImages/Root.AppImage"
        "$HOME/AppImages/root.appimage"
        "$HOME/Downloads/Root.AppImage"
        "$HOME/Downloads/root.appimage"
        "$HOME/.local/bin/Root.AppImage"
        "$HOME/.local/bin/root.appimage"
        "/opt/Root.AppImage"
        "/usr/bin/Root.AppImage"
        "$HOME/.local/bin/Root"
    )

    for c in "${candidates[@]}"; do
        if [[ -f "$c" ]]; then
            ROOT_PATH="$c"
            log "Found Root at: $ROOT_PATH"
            return 0
        fi
    done

    # 2. Glob for variant filenames (versioned, renamed, etc.) in common directories
    local search_dirs=(
        "$HOME/Applications"
        "$HOME/AppImages"
        "$HOME/Downloads"
        "$HOME/.local/bin"
        "$HOME/Desktop"
        "$HOME"
        "/opt"
        "/usr/bin"
        "/usr/local/bin"
    )
    for dir in "${search_dirs[@]}"; do
        [[ -d "$dir" ]] || continue
        for f in "$dir"/Root*.AppImage "$dir"/root*.AppImage "$dir"/Root*.appimage "$dir"/root*.appimage; do
            if [[ -f "$f" ]]; then
                ROOT_PATH="$f"
                log "Found Root at: $ROOT_PATH"
                return 0
            fi
        done
    done

    # 3. Search .desktop files for Root's Exec= path
    local desktop_dirs=(
        "$HOME/.local/share/applications"
        "/usr/share/applications"
        "/usr/local/share/applications"
        "/var/lib/flatpak/exports/share/applications"
        "$HOME/.local/share/flatpak/exports/share/applications"
    )
    for dir in "${desktop_dirs[@]}"; do
        [[ -d "$dir" ]] || continue
        for desktop_file in "$dir"/*.desktop; do
            [[ -f "$desktop_file" ]] || continue
            # Only consider desktop files that mention Root in Name or filename
            if ! grep -qiE '^Name=.*Root' "$desktop_file" 2>/dev/null \
               && [[ "$(basename "$desktop_file")" != *[Rr]oot* ]]; then
                continue
            fi
            local exec_path
            exec_path=$(grep -m1 '^Exec=' "$desktop_file" 2>/dev/null | sed 's/^Exec=//;s/ %[fFuUdDnNickvm]//g;s/ *$//')
            if [[ -n "$exec_path" && -f "$exec_path" ]]; then
                ROOT_PATH="$exec_path"
                log "Found Root via .desktop file: $ROOT_PATH"
                return 0
            fi
        done
    done

    # 4. Check running Root processes via /proc (skip FUSE mounts)
    for pid_dir in /proc/[0-9]*/; do
        local exe
        exe=$(readlink "${pid_dir}exe" 2>/dev/null) || continue
        # Skip ephemeral FUSE mounts from running AppImages
        [[ "$exe" == /tmp/.mount_* ]] && continue
        case "$exe" in
            *Root*.AppImage|*root*.appimage|*/Root)
                if [[ -f "$exe" ]]; then
                    ROOT_PATH="$exe"
                    log "Found Root via running process: $ROOT_PATH"
                    return 0
                fi
                ;;
        esac
    done

    # 5. Try PATH lookup
    if command -v Root &>/dev/null; then
        ROOT_PATH="$(command -v Root)"
        log "Found Root in PATH: $ROOT_PATH"
        return 0
    fi

    # 6. Try locate (fast indexed search)
    if command -v locate &>/dev/null; then
        local located
        located=$(locate -i -l 1 "Root.AppImage" 2>/dev/null || true)
        if [[ -n "$located" && -f "$located" ]]; then
            ROOT_PATH="$located"
            log "Found Root via locate: $ROOT_PATH"
            return 0
        fi
        located=$(locate -i -l 1 --regexp '[Rr]oot.*\.[Aa]pp[Ii]mage$' 2>/dev/null || true)
        if [[ -n "$located" && -f "$located" ]]; then
            ROOT_PATH="$located"
            log "Found Root via locate: $ROOT_PATH"
            return 0
        fi
    fi

    # 7. Shallow find in $HOME (depth-limited to stay fast)
    if command -v find &>/dev/null; then
        local found
        found=$(find "$HOME" -maxdepth 4 -iname "Root*.AppImage" -type f -print -quit 2>/dev/null)
        if [[ -n "$found" && -f "$found" ]]; then
            ROOT_PATH="$found"
            log "Found Root at: $ROOT_PATH"
            return 0
        fi
    fi

    # Nothing found
    echo ""
    error "Could not find Root.AppImage."
    echo ""
    echo "  Searched:"
    echo "    - Common locations (~/Applications, ~/Downloads, ~/.local/bin, /opt)"
    echo "    - Glob patterns for Root*.AppImage in common directories"
    echo "    - .desktop files in application directories"
    echo "    - Running Root processes (/proc)"
    echo "    - PATH, locate database"
    echo "    - find in \$HOME (depth 4)"
    echo ""
    echo "  Tip: locate it manually with:"
    echo "    find / -iname 'Root*.AppImage' 2>/dev/null"
    echo ""
    echo "  Then re-run with: $0 --root-path /path/to/Root.AppImage"
    exit 1
}

# ── Resolve what we actually exec (handles extracted AppImages) ──
#
# On systems without FUSE, AppImages can't run directly.
# Users extract them with: ./Root.AppImage --appimage-extract
# This produces squashfs-root/ next to the .AppImage file.
# We detect that and run the extracted binary with proper LD_LIBRARY_PATH.

resolve_root_exec() {
    # Not an AppImage — exec directly, no lib setup needed
    if [[ "$ROOT_PATH" != *.AppImage && "$ROOT_PATH" != *.appimage ]]; then
        ROOT_EXEC="$ROOT_PATH"
        return 0
    fi

    # Look for an extracted AppImage adjacent to the .AppImage file
    local appimage_dir
    appimage_dir="$(dirname "$(realpath "$ROOT_PATH")")"

    local squash_candidates=(
        "$appimage_dir/squashfs-root"
        "$HOME/Downloads/squashfs-root"
    )

    for squash in "${squash_candidates[@]}"; do
        if [[ -f "$squash/usr/bin/Root" ]]; then
            SQUASHFS_ROOT="$squash"
            ROOT_EXEC="$squash/usr/bin/Root"
            log "Extracted AppImage found — using: $squash"
            return 0
        fi
    done

    # No extracted version found — check FUSE availability
    if [[ -c /dev/fuse ]]; then
        # FUSE present: AppImage should run directly
        ROOT_EXEC="$ROOT_PATH"
        return 0
    fi

    # No FUSE, no extracted version — warn and suggest
    echo ""
    warn "AppImages cannot run on this system (no FUSE support)."
    warn "Extract the AppImage first, then re-run the installer:"
    warn "  cd $(dirname "$ROOT_PATH")"
    warn "  chmod +x $(basename "$ROOT_PATH")"
    warn "  ./$(basename "$ROOT_PATH") --appimage-extract"
    warn "This creates squashfs-root/ in the same directory."
    echo ""
    # Fall back to the AppImage path anyway — let the user's system sort it
    ROOT_EXEC="$ROOT_PATH"
}

# ── Download pre-built artifacts ──

download_prebuilt() {
    resolve_latest_version

    local repo
    repo=$(channel_repo)
    local artifacts_url="https://github.com/${repo}/releases/download/v${VERSION}/uprooted-linux-artifacts.tar.gz"

    log "Downloading pre-built artifacts (v$VERSION, $CHANNEL channel)..."

    if [[ "$CHANNEL" == "dev" && -z "${GITHUB_TOKEN:-}" ]]; then
        error "The dev channel requires a GitHub token for the private repo."
        error "  Set GITHUB_TOKEN in your environment, e.g.:"
        error "    GITHUB_TOKEN=ghp_xxxx ./install-uprooted-linux.sh --channel dev"
        die "  Create a token at: https://github.com/settings/tokens"
    fi

    if ! command -v curl &>/dev/null && ! command -v wget &>/dev/null; then
        die "Neither curl nor wget found. Install one and try again."
    fi

    local tmpdir
    tmpdir=$(mktemp -d)
    local tarball="$tmpdir/uprooted-linux-artifacts.tar.gz"

    if command -v curl &>/dev/null; then
        local curl_opts=(-sL -w "%{http_code}" -o "$tarball" --max-time 120)
        if [[ "$CHANNEL" == "dev" && -n "${GITHUB_TOKEN:-}" ]]; then
            curl_opts+=(-H "Authorization: Bearer $GITHUB_TOKEN" -H "Accept: application/octet-stream")
        fi

        local http_code
        http_code=$(curl "${curl_opts[@]}" "$artifacts_url" 2>/dev/null) || http_code="000"

        if [[ "$http_code" == "404" ]]; then
            rm -rf "$tmpdir"
            error "Version v$VERSION not found on $CHANNEL channel (HTTP 404)."
            error "  Check available releases: https://github.com/${repo}/releases"
            die "  Run with --diagnose for more info."
        elif [[ "$http_code" != "200" && "$http_code" != "000" ]]; then
            rm -rf "$tmpdir"
            error "Failed to download artifacts (HTTP $http_code)."
            error "  URL: $artifacts_url"
            die "  Run with --diagnose for more info."
        elif [[ "$http_code" == "000" ]]; then
            rm -rf "$tmpdir"
            error "Network error — could not reach GitHub."
            die "  Check your internet connection and try again."
        fi
    else
        local wget_opts=(-q -O "$tarball")
        if [[ "$CHANNEL" == "dev" && -n "${GITHUB_TOKEN:-}" ]]; then
            wget_opts+=(--header="Authorization: Bearer $GITHUB_TOKEN" --header="Accept: application/octet-stream")
        fi

        if ! wget "${wget_opts[@]}" "$artifacts_url" 2>/dev/null; then
            rm -rf "$tmpdir"
            error "Failed to download pre-built artifacts."
            error "  URL: $artifacts_url"
            die "  Run with --diagnose for more info."
        fi
    fi

    # Validate tarball (catch corrupt downloads before tar fails cryptically)
    if command -v file &>/dev/null; then
        if ! file "$tarball" | grep -qi "gzip"; then
            rm -rf "$tmpdir"
            error "Downloaded file is not a valid gzip archive (corrupt download?)."
            die "  Try again or download manually from: $artifacts_url"
        fi
    else
        # Fallback: check gzip magic bytes (1f 8b)
        local magic
        magic=$(od -A n -t x1 -N 2 "$tarball" 2>/dev/null | tr -d ' ')
        if [[ "$magic" != "1f8b" ]]; then
            rm -rf "$tmpdir"
            error "Downloaded file is not a valid gzip archive (corrupt download?)."
            die "  Try again or download manually from: $artifacts_url"
        fi
    fi

    mkdir -p "$INSTALL_DIR"
    tar -xzf "$tarball" -C "$INSTALL_DIR"
    chmod +x "$INSTALL_DIR/libuprooted_profiler.so"
    rm -rf "$tmpdir"

    # Verify all expected files exist
    local files=("libuprooted_profiler.so" "UprootedHook.dll" "UprootedHook.deps.json" "uprooted-preload.js" "uprooted.css")
    for f in "${files[@]}"; do
        if [[ ! -f "$INSTALL_DIR/$f" ]]; then
            die "Pre-built artifact missing after extraction: $f"
        fi
    done

    log "Pre-built artifacts deployed to $INSTALL_DIR"
}

# ── Deploy local artifacts (skip building) ──

deploy_local() {
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

    log "Deploying from local build output..."

    # Look for artifacts in repo build output
    local hook_out="$script_dir/hook/bin/Release/net10.0"
    local hook_out9="$script_dir/hook/bin/Release/net9.0"
    local profiler="$script_dir/libuprooted_profiler.so"

    # Validate required artifacts exist
    local missing=false
    for f in "$hook_out/UprootedHook.dll" "$hook_out/UprootedHook.deps.json" \
             "$script_dir/dist/uprooted-preload.js" "$script_dir/dist/uprooted.css" \
             "$profiler"; do
        if [[ ! -f "$f" ]]; then
            error "Missing: $f"
            missing=true
        fi
    done
    if [[ "$missing" == true ]]; then
        die "Build artifacts not found. Run 'dotnet build hook/UprootedHook.csproj -c Release' and build the profiler first."
    fi

    mkdir -p "$INSTALL_DIR"

    cp "$profiler" "$INSTALL_DIR/"
    cp "$hook_out/UprootedHook.dll" "$INSTALL_DIR/"
    cp "$hook_out/UprootedHook.deps.json" "$INSTALL_DIR/"
    cp "$hook_out/nsfw-filter.js" "$INSTALL_DIR/" 2>/dev/null || true
    cp "$hook_out/link-embeds.js" "$INSTALL_DIR/" 2>/dev/null || true
    # net9.0 fallback
    if [[ -f "$hook_out9/UprootedHook.dll" ]]; then
        cp "$hook_out9/UprootedHook.dll" "$INSTALL_DIR/UprootedHook.net9.dll"
        cp "$hook_out9/UprootedHook.deps.json" "$INSTALL_DIR/UprootedHook.net9.deps.json"
    fi
    cp "$script_dir/dist/uprooted-preload.js" "$INSTALL_DIR/"
    cp "$script_dir/dist/uprooted.css" "$INSTALL_DIR/"

    chmod +x "$INSTALL_DIR/libuprooted_profiler.so"

    log "Local artifacts deployed to $INSTALL_DIR"
}

# ── Deploy artifacts (prebuilt or local) ──

deploy_artifacts() {
    if [[ "$USE_LOCAL" == true ]]; then
        deploy_local
    else
        download_prebuilt
    fi
}

# ── Desktop environment detection ──

is_kde() {
    [[ "${XDG_CURRENT_DESKTOP:-}" == *KDE* ]] \
    || [[ "${KDE_SESSION_VERSION:-}" != "" ]] \
    || [[ "${KDE_FULL_SESSION:-}" == "true" ]]
}


# ── Create wrapper script ──

create_wrapper() {
    local wrapper="$INSTALL_DIR/launch-root.sh"
    # Bake in the AppImage dir so the wrapper can detect squashfs-root/ at
    # runtime — this means extraction order doesn't matter (extract before or
    # after install, the wrapper just works).
    local appimage_dir
    appimage_dir="$(dirname "$(realpath "$ROOT_PATH")")"

    {
        echo '#!/bin/bash'
        echo '# Uprooted launcher — sets CLR profiler env vars and launches Root.'
        echo '# Detects squashfs-root/ at runtime so extraction order does not matter.'
        echo ''
        echo "APPIMAGE='$ROOT_PATH'"
        echo "APPIMAGE_DIR='$appimage_dir'"
        echo ''
        echo '# Prefer extracted AppImage (required on systems without FUSE).'
        echo '# AppRun adds usr/bin/ to PATH and execs Root via the .desktop Exec= field.'
        echo '# Fall through to the AppImage itself if no extraction is found.'
        echo 'if [[ -f "$APPIMAGE_DIR/squashfs-root/AppRun" ]]; then'
        echo '    ROOT_EXEC="$APPIMAGE_DIR/squashfs-root/AppRun"'
        echo '    export APPDIR="$APPIMAGE_DIR/squashfs-root"'
        echo 'elif [[ -f "$APPIMAGE_DIR/squashfs-root/usr/bin/Root" ]]; then'
        echo '    ROOT_EXEC="$APPIMAGE_DIR/squashfs-root/usr/bin/Root"'
        echo '    export PATH="$APPIMAGE_DIR/squashfs-root/usr/bin:$PATH"'
        echo 'else'
        echo '    ROOT_EXEC="$APPIMAGE"'
        echo 'fi'
        echo ''
        echo '# .NET 10+ (DOTNET_ prefix)'
        echo 'export DOTNET_EnableDiagnostics=1'
        echo 'export DOTNET_ENABLE_PROFILING=1'
        echo "export DOTNET_PROFILER='$PROFILER_GUID'"
        echo "export DOTNET_PROFILER_PATH='$INSTALL_DIR/libuprooted_profiler.so'"
        echo 'export DOTNET_ReadyToRun=0'
        echo '# Legacy (.NET 8/9)'
        echo 'export CORECLR_ENABLE_PROFILING=1'
        echo "export CORECLR_PROFILER='$PROFILER_GUID'"
        echo "export CORECLR_PROFILER_PATH='$INSTALL_DIR/libuprooted_profiler.so'"
        echo ''
        echo 'exec "$ROOT_EXEC" "$@"'
    } > "$wrapper"
    chmod +x "$wrapper"
    log "Wrapper script created: $wrapper"
}

# ── Create .desktop file (opt-in via --desktop) ──

create_desktop_file() {
    local apps_dir="$HOME/.local/share/applications"
    mkdir -p "$apps_dir"

    cat > "$apps_dir/root-uprooted.desktop" << DESKTOP
[Desktop Entry]
Name=Root (Uprooted)
Comment=Root Communications with Uprooted mods
Exec=$INSTALL_DIR/launch-root.sh
Type=Application
Categories=Network;Chat;
Terminal=false
DESKTOP
    chmod +x "$apps_dir/root-uprooted.desktop"
    log ".desktop file created"
}

# ── Patch HTML files ──

patch_html() {
    if [[ ! -d "$PROFILE_DIR" ]]; then
        warn "Profile directory not found: $PROFILE_DIR"
        warn "Launch Root once to generate it, then re-run this script."
        return
    fi

    local patched=0
    local js_path="$INSTALL_DIR/uprooted-preload.js"
    local css_path="$INSTALL_DIR/uprooted.css"

    # Find HTML files
    local html_files=()
    if [[ -f "$PROFILE_DIR/WebRtcBundle/index.html" ]]; then
        html_files+=("$PROFILE_DIR/WebRtcBundle/index.html")
    fi
    for app_dir in "$PROFILE_DIR/RootApps"/*/; do
        if [[ -f "${app_dir}index.html" ]]; then
            html_files+=("${app_dir}index.html")
        fi
    done

    if [[ ${#html_files[@]} -eq 0 ]]; then
        warn "No HTML files found to patch."
        warn "Launch Root once, then re-run this script."
        return
    fi

    local script_tag="<script src=\"file://$js_path\"></script>"
    local css_tag="<link rel=\"stylesheet\" href=\"file://$css_path\">"
    local marker_start="<!-- uprooted:start -->"
    local marker_end="<!-- uprooted:end -->"

    for html in "${html_files[@]}"; do
        if grep -qE "(uprooted:start|uprooted-preload|<!-- uprooted -->)" "$html" 2>/dev/null; then
            log "Already patched: $(basename "$(dirname "$html")")/index.html"
            continue
        fi

        # Backup original
        cp "$html" "${html}.uprooted.bak"

        # Build injection block with markers
        local injection="${marker_start}\n    ${css_tag}\n    ${script_tag}\n    ${marker_end}"

        # Insert before </head>
        sed -i "s|</head>|    ${injection}\n  </head>|" "$html"
        patched=$((patched + 1))
        log "Patched: $(basename "$(dirname "$html")")/index.html"
    done

    log "$patched HTML file(s) patched"
}

# ── Strip HTML patches (used by repair) ──

strip_html_patches() {
    if [[ ! -d "$PROFILE_DIR" ]]; then
        return
    fi

    local html_files=()
    if [[ -f "$PROFILE_DIR/WebRtcBundle/index.html" ]]; then
        html_files+=("$PROFILE_DIR/WebRtcBundle/index.html")
    fi
    for app_dir in "$PROFILE_DIR/RootApps"/*/; do
        if [[ -f "${app_dir}index.html" ]]; then
            html_files+=("${app_dir}index.html")
        fi
    done

    for html in "${html_files[@]}"; do
        if grep -qE "(uprooted:start|uprooted-preload|<!-- uprooted -->|__UPROOTED_SETTINGS__)" "$html" 2>/dev/null; then
            local tmp="${html}.tmp"
            local inside_block=false
            while IFS= read -r line; do
                if [[ "$line" == *"<!-- uprooted:start -->"* ]]; then
                    inside_block=true
                    continue
                fi
                if [[ "$line" == *"<!-- uprooted:end -->"* ]]; then
                    inside_block=false
                    continue
                fi
                if [[ "$inside_block" == true ]]; then
                    continue
                fi
                if [[ "$line" == *"<!-- uprooted -->"* ]]; then
                    continue
                fi
                if [[ "$line" == *"uprooted-preload"* ]] && [[ "$line" == *"<script"* || "$line" == *"</script"* ]]; then
                    continue
                fi
                if [[ "$line" == *"uprooted.css"* ]] && [[ "$line" == *"<link"* ]]; then
                    continue
                fi
                if [[ "$line" == *"__UPROOTED_SETTINGS__"* ]] && [[ "$line" == *"<script"* ]]; then
                    continue
                fi
                echo "$line"
            done < "$html" > "$tmp"
            mv "$tmp" "$html"
            log "  Stripped: $(basename "$(dirname "$html")")/index.html"
        fi
    done
}

# ── Repair function ──

run_repair() {
    echo ""
    echo "  Uprooted Repair v$VERSION"
    echo "  ────────────────────────"
    echo ""

    find_root
    resolve_root_exec

    # Re-deploy artifacts
    log "Re-deploying artifacts..."
    deploy_artifacts

    # Re-set env vars
    create_wrapper

    if [[ "$CREATE_DESKTOP" == true ]]; then
        create_desktop_file
    fi

    # Strip existing patches and re-apply
    log "Stripping existing HTML patches..."
    strip_html_patches

    log "Re-applying HTML patches..."
    patch_html

    # Kill running Root and relaunch with Uprooted
    if pgrep -f "Root" &>/dev/null; then
        log "Stopping running Root process..."
        pkill -f "Root" 2>/dev/null || true
        sleep 1
        pkill -9 -f "Root" 2>/dev/null || true
        sleep 0.5
    fi

    log "Launching Root with Uprooted..."
    nohup "$INSTALL_DIR/launch-root.sh" &>/dev/null &
    disown

    echo ""
    log "Repair complete! Root is launching."
    log "Trouble? Run: $0 --diagnose"
    echo ""
}

# ── Main ──

if [[ "$MODE" == "diagnose" ]]; then
    run_diagnose
    read -rp "Press Enter to exit..." || true
    exit 0
fi

if [[ "$MODE" == "uninstall" ]]; then
    run_uninstall
    read -rp "Press Enter to exit..." || true
    exit 0
fi

if [[ "$MODE" == "repair" ]]; then
    run_repair
    read -rp "Press Enter to exit..." || true
    exit 0
fi

echo ""
echo "  Uprooted Linux Installer v$VERSION"
echo "  ─────────────────────────────────"
echo ""

find_root
resolve_root_exec
deploy_artifacts
create_wrapper

if [[ "$CREATE_DESKTOP" == true ]]; then
    create_desktop_file
fi

patch_html

# Kill running Root and relaunch with Uprooted
if pgrep -f "Root" &>/dev/null; then
    log "Stopping running Root process..."
    pkill -f "Root" 2>/dev/null || true
    sleep 1
    # Force kill if still alive
    pkill -9 -f "Root" 2>/dev/null || true
    sleep 0.5
fi

log "Launching Root with Uprooted..."
nohup "$INSTALL_DIR/launch-root.sh" &>/dev/null &
disown

echo ""
log "Installation complete! Root is launching."
log "Trouble? Run: $0 --diagnose"
echo ""
read -rp "Press Enter to exit..." || true
