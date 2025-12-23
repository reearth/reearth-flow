import { createContext, useContext, useEffect, useState } from "react";

export const themes = [
  "light",
  "dark",
  "midnight",
  "synthwave",
  "terminal",
  "system",
] as const;

export type Theme = (typeof themes)[number];

// TODO: Should be 'system' once the light mode is fixed
export const DEFAULT_THEME: Theme = "dark";

export const THEME_LABELS: Record<Exclude<Theme, "system">, string> = {
  light: "Light",
  dark: "Dark",
  midnight: "Midnight",
  synthwave: "Synthwave",
  terminal: "Terminal",
};

export const THEME_DESCRIPTIONS: Record<Exclude<Theme, "system">, string> = {
  light: "Soft gray with pastel accents",
  dark: "Dark with subtle cool tones",
  midnight: "Extra dark for OLED",
  synthwave: "Neon 80s retro vibes",
  terminal: "Classic green phosphor CRT",
};

type ThemeProviderProps = {
  children: React.ReactNode;
  defaultTheme?: Theme;
  storageKey?: string;
};

type ThemeProviderState = {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  previewTheme: (theme: Theme) => void;
};

const initialState: ThemeProviderState = {
  theme: DEFAULT_THEME,
  setTheme: () => null,
  previewTheme: () => null,
};

const ThemeProviderContext = createContext<ThemeProviderState>(initialState);

function ThemeProvider({
  children,
  defaultTheme = DEFAULT_THEME,
  storageKey = "vite-ui-theme",
  ...props
}: ThemeProviderProps) {
  const [theme, setTheme] = useState<Theme>(
    () => (localStorage.getItem(storageKey) as Theme) || defaultTheme,
  );

  useEffect(() => {
    const root = window.document.documentElement;

    // Handle system theme
    if (theme === "system") {
      const systemTheme = window.matchMedia("(prefers-color-scheme: dark)")
        .matches
        ? "dark"
        : "light";

      // Set data-theme
      root.setAttribute("data-theme", systemTheme);

      // Maintain .dark class for backward compatibility
      root.classList.remove("light", "dark");
      root.classList.add(systemTheme);
      return;
    }

    // Set data-theme attribute for CSS [data-theme="..."] selectors
    root.setAttribute("data-theme", theme);

    // Also maintain .dark class for backward compatibility with existing dark: modifiers
    root.classList.remove("light", "dark");
    if (theme === "dark") {
      root.classList.add("dark");
    } else {
      root.classList.add(theme);
    }
  }, [theme]);

  const value = {
    theme,
    setTheme: (theme: Theme) => {
      localStorage.setItem(storageKey, theme);
      setTheme(theme);
    },
    previewTheme: (theme: Theme) => {
      // Set theme without saving to localStorage
      setTheme(theme);
    },
  };

  return (
    <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>
  );
}

const useTheme = () => {
  const context = useContext(ThemeProviderContext);

  if (context === undefined)
    throw new Error("useTheme must be used within a ThemeProvider");

  return context;
};

export { ThemeProvider, useTheme };
