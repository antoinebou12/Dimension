/**
 * Theme (light/dark) toggle for mathlib WASM demo.
 * Applies saved theme on load and injects a toggle button into #theme-toggle-container.
 */
(function () {
  const STORAGE_KEY = "mathlib-demo-theme";

  function getTheme() {
    try {
      const t = localStorage.getItem(STORAGE_KEY);
      return t === "dark" ? "dark" : "light";
    } catch (_) {
      return "light";
    }
  }

  function setTheme(theme) {
    theme = theme === "dark" ? "dark" : "light";
    try {
      localStorage.setItem(STORAGE_KEY, theme);
    } catch (_) {}
    document.documentElement.setAttribute("data-theme", theme);
    return theme;
  }

  function applyTheme() {
    const theme = getTheme();
    document.documentElement.setAttribute("data-theme", theme);
  }

  function renderToggle(container) {
    if (!container) return;
    const theme = getTheme();
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "theme-toggle";
    btn.setAttribute("aria-label", "Toggle dark mode");
    btn.textContent = theme === "dark" ? "Light" : "Dark";
    btn.addEventListener("click", function () {
      const next = getTheme() === "dark" ? "light" : "dark";
      setTheme(next);
      btn.textContent = next === "dark" ? "Light" : "Dark";
    });
    container.appendChild(btn);
  }

  applyTheme();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      renderToggle(document.getElementById("theme-toggle-container"));
    });
  } else {
    renderToggle(document.getElementById("theme-toggle-container"));
  }

  window.mathlibDemoTheme = { getTheme: getTheme, setTheme: setTheme };
})();
