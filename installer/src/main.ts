import { initStarfield } from "./starfield.js";
import { init as initInstallerPage } from "./pages/main.js";
import { init as initThemesPage } from "./pages/themes.js";

const { getCurrentWindow } = (window as any).__TAURI__.window;

function setupTitlebar(): void {
  const appWindow = getCurrentWindow();
  document.getElementById("btn-minimize")?.addEventListener("click", () => { appWindow.minimize(); });
  document.getElementById("btn-close")?.addEventListener("click", () => { appWindow.close(); });
}

type PageName = "installer" | "themes";
const pageInits: Record<PageName, (el: HTMLElement) => Promise<void>> = {
  installer: initInstallerPage,
  themes: initThemesPage,
};
const initialized = new Set<PageName>();

function switchPage(name: PageName): void {
  for (const tab of document.querySelectorAll<HTMLElement>(".nav-tab"))
    tab.classList.toggle("active", tab.dataset.page === name);
  for (const page of document.querySelectorAll<HTMLElement>(".page"))
    page.classList.toggle("active", page.id === `page-${name}`);
  if (!initialized.has(name)) {
    initialized.add(name);
    const el = document.getElementById(`page-${name}`);
    if (el && pageInits[name]) pageInits[name](el);
  }
}

function setupNav(): void {
  for (const tab of document.querySelectorAll<HTMLElement>(".nav-tab")) {
    tab.addEventListener("click", () => {
      const page = tab.dataset.page as PageName | undefined;
      if (page) switchPage(page);
    });
  }
}

document.addEventListener("DOMContentLoaded", () => {
  initStarfield();
  setupTitlebar();
  setupNav();
  switchPage("installer");
});
