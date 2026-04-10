const STORAGE_KEY = "noordpool_theme";

type Mode = "light" | "dark";

function createThemeState() {
  let mode: Mode = $state("dark");

  function init() {
    const stored = localStorage.getItem(STORAGE_KEY) as Mode | null;
    mode = stored === "light" ? "light" : "dark";
    applyClass();
  }

  function toggle() {
    mode = mode === "dark" ? "light" : "dark";
    localStorage.setItem(STORAGE_KEY, mode);
    applyClass();
  }

  function applyClass() {
    if (mode === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }

  return {
    get mode() {
      return mode;
    },
    get isDark() {
      return mode === "dark";
    },
    init,
    toggle,
  };
}

export const theme = createThemeState();
