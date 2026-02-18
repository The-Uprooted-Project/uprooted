#!/bin/bash
# Uprooted Linux Installer v0.3.41
# Standalone bash installer for systems without the GUI installer.
#
# Usage: ./install-uprooted-linux.sh [--root-path /path/to/Root.AppImage]
#        ./install-uprooted-linux.sh --prebuilt [--root-path /path/to/Root.AppImage]
#        ./install-uprooted-linux.sh --uninstall
#        ./install-uprooted-linux.sh --repair [--prebuilt]
#        ./install-uprooted-linux.sh --diagnose
#
# This script:
# 1. Finds Root.AppImage (or uses --root-path)
# 2. Builds (or downloads) profiler + hook artifacts
# 3. Deploys to ~/.local/share/uprooted/
# 4. Creates a wrapper script with CLR profiler env vars
# 5. Creates a .desktop file for launching Root with Uprooted
# 6. Patches HTML files in Root's profile directory
# 7. Adds env vars to ~/.profile as fallback for non-systemd sessions

set -euo pipefail

INSTALL_DIR="$HOME/.local/share/uprooted"
PROFILE_DIR="$HOME/.local/share/Root Communications/Root/profile/default"
PROFILER_GUID="{D1A6F5A0-1234-4567-89AB-CDEF01234567}"
VERSION="0.3.41"
ARTIFACTS_URL="https://github.com/watchthelight/uprooted/releases/download/v${VERSION}/uprooted-linux-artifacts.tar.gz"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log()   { echo -e "${GREEN}[+]${NC} $1"; }
warn()  { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[-]${NC} $1"; }
die()   { error "$1"; exit 1; }

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
        warn "  root-uprooted.desktop: missing"
    fi

    local plasma_env="$HOME/.config/plasma-workspace/env/uprooted.sh"
    if [[ -f "$plasma_env" ]]; then
        log "  plasma-workspace/env/uprooted.sh: exists (KDE Plasma)"
    else
        warn "  plasma-workspace/env/uprooted.sh: missing (KDE users need this)"
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

    local hook_log="$INSTALL_DIR/uprooted-hook.log"
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

    # 5. Remove .desktop file
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
USE_PREBUILT=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --root-path) ROOT_PATH="$2"; shift 2 ;;
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
        --prebuilt)
            USE_PREBUILT=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--root-path /path/to/Root.AppImage] [--prebuilt]"
            echo "       $0 --uninstall"
            echo "       $0 --repair [--prebuilt]"
            echo "       $0 --diagnose"
            echo ""
            echo "Installs Uprooted client mod framework for Root Communications."
            echo ""
            echo "Options:"
            echo "  --root-path    Path to Root.AppImage (auto-detected if not given)"
            echo "  --prebuilt     Download pre-built artifacts instead of building from source"
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

    local candidates=(
        "$HOME/Applications/Root.AppImage"
        "$HOME/Downloads/Root.AppImage"
        "$HOME/.local/bin/Root.AppImage"
        "/opt/Root.AppImage"
        "$HOME/.local/bin/Root"
    )

    for c in "${candidates[@]}"; do
        if [[ -f "$c" ]]; then
            ROOT_PATH="$c"
            log "Found Root at: $ROOT_PATH"
            return 0
        fi
    done

    # Try which
    if command -v Root &>/dev/null; then
        ROOT_PATH="$(which Root)"
        log "Found Root in PATH: $ROOT_PATH"
        return 0
    fi

    die "Could not find Root.AppImage. Use --root-path to specify its location."
}

# ── Check prerequisites (build from source) ──

check_prereqs() {
    local missing=()

    if ! command -v gcc &>/dev/null; then
        missing+=("gcc")
    fi
    if ! command -v dotnet &>/dev/null; then
        missing+=("dotnet-sdk-10.0")
    fi
    if ! command -v node &>/dev/null; then
        missing+=("nodejs")
    fi
    if ! command -v pnpm &>/dev/null; then
        missing+=("pnpm")
    fi

    if [[ ${#missing[@]} -gt 0 ]]; then
        error "Missing build dependencies: ${missing[*]}"
        echo "Install them and try again, or use --prebuilt to skip building."
        echo ""
        echo "Ubuntu/Debian:"
        echo "  sudo apt install gcc nodejs"
        echo "  # Install .NET 10: https://dotnet.microsoft.com/download"
        echo "  # Install pnpm: npm install -g pnpm"
        exit 1
    fi
}

# ── Download pre-built artifacts ──

download_prebuilt() {
    log "Downloading pre-built artifacts..."

    if ! command -v curl &>/dev/null && ! command -v wget &>/dev/null; then
        die "Neither curl nor wget found. Install one and try again."
    fi

    local tmpdir
    tmpdir=$(mktemp -d)
    local tarball="$tmpdir/uprooted-linux-artifacts.tar.gz"

    if command -v curl &>/dev/null; then
        if ! curl -fSL -o "$tarball" "$ARTIFACTS_URL" 2>/dev/null; then
            rm -rf "$tmpdir"
            die "Failed to download pre-built artifacts from $ARTIFACTS_URL"
        fi
    else
        if ! wget -q -O "$tarball" "$ARTIFACTS_URL" 2>/dev/null; then
            rm -rf "$tmpdir"
            die "Failed to download pre-built artifacts from $ARTIFACTS_URL"
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

# ── Build artifacts from source ──

build_artifacts() {
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

    log "Building artifacts from source..."

    # Build TypeScript layer
    log "Building TypeScript layer..."
    (cd "$script_dir" && pnpm install --frozen-lockfile && pnpm build)

    # Build Hook DLL
    log "Building UprootedHook.dll..."
    dotnet build "$script_dir/hook" -c Release -o "$script_dir/hook/_out"

    # Build profiler .so
    log "Compiling libuprooted_profiler.so..."
    gcc -shared -fPIC -O2 -o "$script_dir/libuprooted_profiler.so" "$script_dir/tools/uprooted_profiler_linux.c"

    # Deploy
    mkdir -p "$INSTALL_DIR"

    cp "$script_dir/libuprooted_profiler.so" "$INSTALL_DIR/"
    cp "$script_dir/hook/_out/UprootedHook.dll" "$INSTALL_DIR/"
    cp "$script_dir/hook/_out/UprootedHook.deps.json" "$INSTALL_DIR/"
    cp "$script_dir/dist/uprooted-preload.js" "$INSTALL_DIR/"
    cp "$script_dir/dist/uprooted.css" "$INSTALL_DIR/"

    chmod +x "$INSTALL_DIR/libuprooted_profiler.so"

    log "Artifacts deployed to $INSTALL_DIR"
}

# ── Deploy artifacts (prebuilt or from source) ──

deploy_artifacts() {
    if [[ "$USE_PREBUILT" == true ]]; then
        download_prebuilt
    else
        check_prereqs
        build_artifacts
    fi
}

# ── Set session-wide env vars (systemd environment.d) ──

set_env_vars() {
    local env_dir="$HOME/.config/environment.d"
    mkdir -p "$env_dir"

    cat > "$env_dir/uprooted.conf" << ENVCONF
# Uprooted -- remove this file or run the uninstaller to disable
# .NET 10+ (DOTNET_ prefix)
DOTNET_EnableDiagnostics=1
DOTNET_ENABLE_PROFILING=1
DOTNET_PROFILER=$PROFILER_GUID
DOTNET_PROFILER_PATH=$INSTALL_DIR/libuprooted_profiler.so
DOTNET_ReadyToRun=0
# Legacy (.NET 8/9)
CORECLR_ENABLE_PROFILING=1
CORECLR_PROFILER=$PROFILER_GUID
CORECLR_PROFILER_PATH=$INSTALL_DIR/libuprooted_profiler.so
ENVCONF
    log "Session env vars written to $env_dir/uprooted.conf"

    # KDE Plasma env script -- sourced on Plasma session startup
    local plasma_env_dir="$HOME/.config/plasma-workspace/env"
    mkdir -p "$plasma_env_dir"
    cat > "$plasma_env_dir/uprooted.sh" << PLASMAENV
#!/bin/sh
# Uprooted -- remove this file or run the uninstaller to disable
export DOTNET_EnableDiagnostics=1
export DOTNET_ENABLE_PROFILING=1
export DOTNET_PROFILER='$PROFILER_GUID'
export DOTNET_PROFILER_PATH='$INSTALL_DIR/libuprooted_profiler.so'
export DOTNET_ReadyToRun=0
export CORECLR_ENABLE_PROFILING=1
export CORECLR_PROFILER='$PROFILER_GUID'
export CORECLR_PROFILER_PATH='$INSTALL_DIR/libuprooted_profiler.so'
PLASMAENV
    chmod +x "$plasma_env_dir/uprooted.sh"
    log "KDE Plasma env script written to $plasma_env_dir/uprooted.sh"

    # Also add to ~/.profile as fallback for non-systemd sessions (X11, login shells)
    if ! grep -q "DOTNET_ENABLE_PROFILING" "$HOME/.profile" 2>/dev/null; then
        cat >> "$HOME/.profile" << PROFILE

# Uprooted (remove these lines to disable)
export DOTNET_EnableDiagnostics=1
export DOTNET_ENABLE_PROFILING=1
export DOTNET_PROFILER='$PROFILER_GUID'
export DOTNET_PROFILER_PATH='$INSTALL_DIR/libuprooted_profiler.so'
export DOTNET_ReadyToRun=0
export CORECLR_ENABLE_PROFILING=1
export CORECLR_PROFILER='$PROFILER_GUID'
export CORECLR_PROFILER_PATH='$INSTALL_DIR/libuprooted_profiler.so'
PROFILE
        log "Env vars appended to ~/.profile (login shell fallback)"
    else
        log "~/.profile already contains Uprooted env vars"
    fi

    warn "Log out and back in (or reboot) for env vars to take effect globally."
    warn "Or use the wrapper script below for immediate use."
}

# ── Create wrapper script ──

create_wrapper() {
    local wrapper="$INSTALL_DIR/launch-root.sh"
    cat > "$wrapper" << WRAPPER
#!/bin/bash
# Uprooted launcher - sets injection env vars for Root only
# .NET 10+ (DOTNET_ prefix)
export DOTNET_EnableDiagnostics=1
export DOTNET_ENABLE_PROFILING=1
export DOTNET_PROFILER='$PROFILER_GUID'
export DOTNET_PROFILER_PATH='$INSTALL_DIR/libuprooted_profiler.so'
export DOTNET_ReadyToRun=0
# Legacy (.NET 8/9)
export CORECLR_ENABLE_PROFILING=1
export CORECLR_PROFILER='$PROFILER_GUID'
export CORECLR_PROFILER_PATH='$INSTALL_DIR/libuprooted_profiler.so'
exec '$ROOT_PATH' "\$@"
WRAPPER
    chmod +x "$wrapper"
    log "Wrapper script created: $wrapper"
}

# ── Create .desktop file ──

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

    # Re-deploy artifacts
    log "Re-deploying artifacts..."
    deploy_artifacts

    # Re-set env vars
    set_env_vars
    create_wrapper
    create_desktop_file

    # Strip existing patches and re-apply
    log "Stripping existing HTML patches..."
    strip_html_patches

    log "Re-applying HTML patches..."
    patch_html

    echo ""
    log "Repair complete!"
    log "Restart Root to activate Uprooted."
    echo ""
}

# ── Main ──

if [[ "$MODE" == "diagnose" ]]; then
    run_diagnose
    exit 0
fi

if [[ "$MODE" == "uninstall" ]]; then
    run_uninstall
    exit 0
fi

if [[ "$MODE" == "repair" ]]; then
    run_repair
    exit 0
fi

echo ""
echo "  Uprooted Linux Installer v$VERSION"
echo "  ─────────────────────────────────"
echo ""

find_root
deploy_artifacts
set_env_vars
create_wrapper
create_desktop_file
patch_html

echo ""
log "Installation complete!"
log ""
log "To activate Uprooted, either:"
log "  1. Log out and back in, then launch Root normally"
log "  2. Run: $INSTALL_DIR/launch-root.sh"
log "  3. Find 'Root (Uprooted)' in your application menu"
log ""
log "If Uprooted isn't loading, run: $0 --diagnose"
echo ""
