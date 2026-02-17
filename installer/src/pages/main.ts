import {
  detectRoot, installUprooted, uninstallUprooted, repairUprooted,
  getUprootedVersion, checkRootRunning, killRoot, type DetectionResult,
} from "../lib/tauri.js";

let logEl: HTMLDivElement;
let detection: DetectionResult | null = null;
const isLinux = navigator.platform.startsWith("Linux");
const rootExeName = isLinux ? "Root" : "root.exe";

function log(text: string, type: "info" | "success" | "error" | "warn" | "" = ""): void {
  const line = document.createElement("div");
  line.className = `log-line ${type}`;
  line.innerHTML = `<span class="prefix">&gt;</span>${escapeHtml(text)}`;
  logEl.appendChild(line);
  logEl.scrollTop = logEl.scrollHeight;
}

function logBlank(): void {
  const line = document.createElement("div");
  line.className = "log-line";
  line.innerHTML = "&nbsp;";
  logEl.appendChild(line);
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function statusDot(color: "green" | "red" | "yellow"): string {
  return `<span class="status-dot ${color}"></span>`;
}

function truncatePath(p: string, maxLen = 45): string {
  if (p.length <= maxLen) return p;
  return "..." + p.slice(p.length - maxLen + 3);
}

function fileName(p: string): string {
  const parts = p.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || p;
}

async function runDetection(): Promise<void> {
  log("scanning system...");
  try { detection = await detectRoot(); }
  catch (err) { log(`detection failed: ${err}`, "error"); return; }
  logBlank();
  if (detection.root_found) {
    log(`${rootExeName} found`, "success");
    log(`  path: ${detection.root_path}`);
  } else {
    log(`${rootExeName} not found`, "error");
    log("  is the app installed?", "warn");
  }
  log(`profile: ${detection.profile_dir}`);
  if (detection.html_files.length > 0) {
    log(`${detection.html_files.length} html target${detection.html_files.length > 1 ? "s" : ""} found`, "success");
    for (const f of detection.html_files) log(`  ${fileName(f)}`);
  } else {
    log("no html targets found", "warn");
    log("  app may need to be launched once to generate profile files", "warn");
  }
  logBlank();
  const hs = detection.hook_status;
  if (hs.files_ok) {
    log("hook files: all deployed", "success");
  } else {
    const missing: string[] = [];
    if (!hs.profiler_dll) missing.push(isLinux ? "profiler so" : "profiler dll");
    if (!hs.hook_dll) missing.push("hook dll");
    if (!hs.hook_deps) missing.push("hook deps");
    if (!hs.preload_js) missing.push("preload.js");
    if (!hs.theme_css) missing.push("theme css");
    if (missing.length === 5) log("hook files: not deployed");
    else log(`hook files: partial (missing: ${missing.join(", ")})`, "warn");
  }
  if (hs.env_ok) {
    if (isLinux && !hs.env_vars_active) log("env vars: configured (not active - restart session or use wrapper)", "warn");
    else log("env vars: configured", "success");
  } else {
    const missing: string[] = [];
    if (!hs.env_enable_profiling) missing.push("CORECLR_ENABLE_PROFILING");
    if (!hs.env_profiler_guid) missing.push("CORECLR_PROFILER");
    if (!hs.env_profiler_path) missing.push("CORECLR_PROFILER_PATH");
    if (!hs.env_ready_to_run) missing.push("DOTNET_ReadyToRun");
    if (missing.length === 4) log("env vars: not configured");
    else log(`env vars: partial (missing: ${missing.join(", ")})`, "warn");
  }
  if (detection.is_installed) log("html patches: applied", "success");
  else log("html patches: not applied");
  logBlank();
  analyzeScenario();
  updateStatusDisplay();
  updateButtons();
}

function analyzeScenario(): void {
  if (!detection) return;
  const hs = detection.hook_status;
  const allGood = detection.root_found && hs.files_ok && hs.env_ok && detection.is_installed;
  if (allGood) {
    if (isLinux && !hs.env_vars_active) {
      log("status: installed (env vars not active)", "warn");
      log("  log out and back in, or launch via 'Root (Uprooted)' in app menu", "warn");
      return;
    }
    log("status: fully installed", "success");
    log("  restart root to activate uprooted", "success");
    return;
  }
  if (!detection.root_found) { log("recommendation: install the app first", "warn"); return; }
  if (detection.html_files.length === 0) { log("recommendation: launch root once to generate profile, then install", "warn"); return; }
  const hasAnyFiles = hs.profiler_dll || hs.hook_dll || hs.hook_deps || hs.preload_js || hs.theme_css;
  const hasAnyEnv = hs.env_enable_profiling || hs.env_profiler_guid || hs.env_profiler_path;
  if (hasAnyFiles && !hs.files_ok) log("warning: hook files are partially deployed - try repair", "warn");
  if (hasAnyEnv && !hs.env_ok) log("warning: environment variables are partially configured - try repair", "warn");
  if (hs.files_ok && hs.env_ok && !detection.is_installed) log("hook is deployed but html is not patched - try repair", "warn");
  if (detection.is_installed && (!hs.files_ok || !hs.env_ok)) log("html is patched but hook deployment is incomplete - try repair", "warn");
  if (!hasAnyFiles && !hasAnyEnv && !detection.is_installed) log("ready to install", "info");
}

function updateStatusDisplay(): void {
  const el = document.getElementById("status-rows");
  if (!el || !detection) return;
  const hs = detection.hook_status;
  const allGood = detection.root_found && hs.files_ok && hs.env_ok && detection.is_installed;
  el.innerHTML = `
    <div class="status-row">${statusDot(detection.root_found ? "green" : "red")}<span class="status-label">${rootExeName}</span><span class="status-value">${detection.root_found ? truncatePath(detection.root_path) : "not found"}</span></div>
    <div class="status-row">${statusDot(detection.html_files.length > 0 ? "green" : "yellow")}<span class="status-label">Profile</span><span class="status-value">${detection.html_files.length} HTML target${detection.html_files.length !== 1 ? "s" : ""}</span></div>
    <div class="status-row">${statusDot(hs.files_ok ? "green" : (hs.profiler_dll || hs.hook_dll ? "yellow" : "red"))}<span class="status-label">Hook Files</span><span class="status-value">${hs.files_ok ? "deployed" : "not deployed"}</span></div>
    <div class="status-row">${statusDot(hs.env_ok ? (hs.env_vars_active ? "green" : "yellow") : (hs.env_enable_profiling ? "yellow" : "red"))}<span class="status-label">Env Vars</span><span class="status-value">${hs.env_ok ? (hs.env_vars_active ? "configured" : "configured (not active)") : "not set"}</span></div>
    <div class="status-row">${statusDot(detection.is_installed ? "green" : "red")}<span class="status-label">HTML Patch</span><span class="status-value">${detection.is_installed ? "applied" : "not applied"}</span></div>
    ${allGood ? '<div class="status-note success">restart root to activate</div>' : ""}`;
}

function updateButtons(): void {
  const installBtn = document.getElementById("btn-install") as HTMLButtonElement | null;
  const uninstallBtn = document.getElementById("btn-uninstall") as HTMLButtonElement | null;
  const repairBtn = document.getElementById("btn-repair") as HTMLButtonElement | null;
  if (!detection) return;
  const isInstalled = detection.is_installed || detection.hook_status.files_ok || detection.hook_status.env_ok;
  if (installBtn) { installBtn.disabled = !detection.root_found || isInstalled; installBtn.classList.remove("loading"); }
  if (uninstallBtn) { uninstallBtn.disabled = !isInstalled; uninstallBtn.classList.remove("loading"); }
  if (repairBtn) { repairBtn.disabled = !isInstalled; repairBtn.classList.remove("loading"); }
}

function setButtonLoading(btnId: string): void {
  const btn = document.getElementById(btnId) as HTMLButtonElement | null;
  if (btn) { btn.disabled = true; btn.classList.add("loading"); }
}

function setButtonsDisabled(disabled: boolean): void {
  for (const id of ["btn-install", "btn-uninstall", "btn-repair"]) {
    const btn = document.getElementById(id) as HTMLButtonElement | null;
    if (btn) btn.disabled = disabled;
  }
}

async function ensureRootClosed(): Promise<boolean> {
  let running = false;
  try { running = await checkRootRunning(); } catch { return true; }
  if (!running) return true;
  return new Promise((resolve) => {
    const overlay = document.createElement("div");
    overlay.className = "popup-overlay";
    overlay.innerHTML = `
      <div class="popup">
        <div class="popup-text">${rootExeName} is running</div>
        <div class="popup-sub">close it to continue, or we can do it for you</div>
        <div class="popup-actions">
          <button class="btn danger popup-kill">close root</button>
          <button class="btn popup-cancel">cancel</button>
        </div>
      </div>`;
    document.body.appendChild(overlay);
    const cleanup = () => { overlay.remove(); };
    overlay.addEventListener("click", (e) => { if (e.target === overlay) { cleanup(); resolve(false); } });
    overlay.querySelector(".popup-cancel")!.addEventListener("click", () => { cleanup(); resolve(false); });
    overlay.querySelector(".popup-kill")!.addEventListener("click", async () => {
      const killBtn = overlay.querySelector(".popup-kill") as HTMLButtonElement;
      killBtn.disabled = true;
      killBtn.textContent = "closing...";
      try {
        const killed = await killRoot();
        log(`closed ${killed} root process${killed !== 1 ? "es" : ""}`, "info");
        await new Promise((r) => setTimeout(r, 1500));
      } catch (err) { log(`failed to close root: ${err}`, "error"); cleanup(); resolve(false); return; }
      try {
        const still = await checkRootRunning();
        if (still) { log("root.exe is still running - close it manually", "error"); cleanup(); resolve(false); return; }
      } catch {}
      cleanup();
      resolve(true);
    });
  });
}

async function handleInstall(): Promise<void> {
  if (!(await ensureRootClosed())) return;
  setButtonLoading("btn-install");
  setButtonsDisabled(true);
  log("installing uprooted...", "info");
  log("  deploying hook files...");
  log("  setting environment variables...");
  log("  patching html files...");
  try {
    const result = await installUprooted();
    logBlank();
    if (result.success) {
      log(result.message, "success");
      for (const f of result.files_patched) log(`  patched: ${fileName(f)}`, "success");
      logBlank();
      log("restart root to activate uprooted", "success");
    } else log(result.message, "error");
    await runDetection();
  } catch (err) { log(`install failed: ${err}`, "error"); updateButtons(); }
}

async function handleUninstall(): Promise<void> {
  if (!(await ensureRootClosed())) return;
  setButtonLoading("btn-uninstall");
  setButtonsDisabled(true);
  log("uninstalling uprooted...", "info");
  log("  removing environment variables...");
  log("  restoring html files...");
  log("  removing hook files...");
  try {
    const result = await uninstallUprooted();
    logBlank();
    if (result.success) {
      log(result.message, "success");
      for (const f of result.files_patched) log(`  restored: ${fileName(f)}`, "success");
    } else log(result.message, "error");
    await runDetection();
  } catch (err) { log(`uninstall failed: ${err}`, "error"); updateButtons(); }
}

async function handleRepair(): Promise<void> {
  if (!(await ensureRootClosed())) return;
  setButtonLoading("btn-repair");
  setButtonsDisabled(true);
  log("repairing uprooted...", "info");
  log("  re-deploying hook files...");
  log("  re-setting environment variables...");
  log("  re-patching html files...");
  try {
    const result = await repairUprooted();
    logBlank();
    if (result.success) log(result.message, "success");
    else log(result.message, "error");
    await runDetection();
  } catch (err) { log(`repair failed: ${err}`, "error"); updateButtons(); }
}

function copyLogs(): void {
  if (!logEl) return;
  const text = Array.from(logEl.querySelectorAll(".log-line"))
    .map((el) => (el as HTMLElement).textContent?.replace(/^>/, "").trim() ?? "")
    .filter((l) => l.length > 0).join("\n");
  navigator.clipboard.writeText(text).then(() => {
    const badge = document.getElementById("copy-badge");
    if (badge) { badge.textContent = "copied"; badge.classList.add("show"); setTimeout(() => badge.classList.remove("show"), 1500); }
  });
}

export async function init(container: HTMLElement): Promise<void> {
  let version = "0.2.5";
  try { version = await getUprootedVersion(); } catch {}
  container.innerHTML = `
    <div class="header">
      <h1>uprooted <span class="dim">v${version}</span></h1>
      <p class="sub">a client mod framework</p>
    </div>
    <div class="status-section">
      <div class="status-header">-- status --</div>
      <div id="status-rows"><div class="status-row"><span class="status-dot yellow"></span><span class="status-label">detecting...</span></div></div>
    </div>
    <div class="actions">
      <button id="btn-install" class="btn primary" disabled>install</button>
      <button id="btn-uninstall" class="btn danger" disabled>uninstall</button>
      <button id="btn-repair" class="btn warn" disabled>repair</button>
    </div>
    <div class="log-section">
      <div class="log-toolbar">
        <span class="log-header">-- log --</span>
        <button id="btn-copy-log" class="log-copy-btn" title="Copy log to clipboard">copy</button>
        <span id="copy-badge" class="copy-badge"></span>
      </div>
      <div id="log" class="log"></div>
    </div>`;
  logEl = document.getElementById("log") as HTMLDivElement;
  document.getElementById("btn-install")!.addEventListener("click", handleInstall);
  document.getElementById("btn-uninstall")!.addEventListener("click", handleUninstall);
  document.getElementById("btn-repair")!.addEventListener("click", handleRepair);
  document.getElementById("btn-copy-log")!.addEventListener("click", copyLogs);
  await runDetection();
}
