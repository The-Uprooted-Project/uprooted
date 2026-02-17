import { listThemes, applyTheme, loadSettings, type ThemeDefinition } from "../lib/tauri.js";

let themes: ThemeDefinition[] = [];
let activeTheme = "default";
let selectedTheme: ThemeDefinition | null = null;
let container: HTMLElement;

function renderThemeCard(theme: ThemeDefinition): string {
  const isActive = theme.name === activeTheme;
  const p = theme.preview_colors;
  return `
    <div class="theme-card ${isActive ? "active" : ""}" data-theme="${theme.name}">
      <div class="theme-swatch">
        <div class="swatch-color" style="background: ${p.background}"></div>
        <div class="swatch-color" style="background: ${p.accent}"></div>
        <div class="swatch-color" style="background: ${p.text}"></div>
      </div>
      <div class="theme-info">
        <div class="theme-name">${theme.display_name}</div>
        <div class="theme-desc">${theme.description}</div>
        <div class="theme-author">by ${theme.author}</div>
      </div>
      ${isActive ? '<span class="theme-active-badge">active</span>' : ""}
    </div>`;
}

function renderDetail(theme: ThemeDefinition): string {
  const entries = Object.entries(theme.variables);
  if (entries.length === 0) return `<div class="theme-detail"><div class="theme-detail-header">${theme.display_name}</div><p class="notice">default theme - no variable overrides</p></div>`;
  const rows = entries.map(([name, value]) => `<div class="var-row"><span class="var-name">${name}</span><span class="var-value"><span class="var-color-chip" style="background: ${value}"></span>${value}</span></div>`).join("");
  return `<div class="theme-detail"><div class="theme-detail-header">${theme.display_name} - css variables</div><div class="var-table">${rows}</div></div>`;
}

function render(): void {
  const grid = themes.map(renderThemeCard).join("");
  const detail = selectedTheme ? renderDetail(selectedTheme) : "";
  container.innerHTML = `
    <div class="header"><h1>themes</h1><p class="sub">click a theme to select it, then apply</p></div>
    <div class="themes-grid">${grid}</div>
    ${detail}
    <p class="notice">restart root to apply theme changes.</p>`;
  for (const card of container.querySelectorAll<HTMLElement>(".theme-card")) {
    card.addEventListener("click", async () => {
      const name = card.dataset.theme!;
      selectedTheme = themes.find((t) => t.name === name) ?? null;
      if (selectedTheme && selectedTheme.name !== activeTheme) {
        try { await applyTheme(name); activeTheme = name; }
        catch (err) { console.error("Failed to apply theme:", err); }
      }
      render();
    });
  }
}

export async function init(el: HTMLElement): Promise<void> {
  container = el;
  container.innerHTML = `<div class="header"><h1>themes</h1><p class="sub">loading...</p></div>`;
  try { themes = await listThemes(); } catch { themes = []; }
  try {
    const settings = await loadSettings();
    const themeConfig = settings.plugins?.themes?.config;
    if (themeConfig?.theme && typeof themeConfig.theme === "string") activeTheme = themeConfig.theme;
  } catch {}
  selectedTheme = themes.find((t) => t.name === activeTheme) ?? themes[0] ?? null;
  render();
}
