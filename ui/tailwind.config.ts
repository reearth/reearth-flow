import type { Config } from "tailwindcss";
import plugin from "tailwindcss/plugin";

const config = {
  darkMode: "class",
  important: true,
  content: ["./src/**/*.{ts,tsx}"],
  prefix: "",
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "var(--border)",
        input: "var(--input)",
        ring: "var(--ring)",
        background: "var(--background)",
        foreground: "var(--foreground)",
        primary: {
          DEFAULT: "var(--primary)",
          foreground: "var(--primary-foreground)",
        },
        secondary: {
          DEFAULT: "var(--secondary)",
          foreground: "var(--secondary-foreground)",
        },
        destructive: {
          DEFAULT: "var(--destructive)",
          foreground: "var(--destructive-foreground)",
        },
        warning: {
          DEFAULT: "var(--warning)",
          foreground: "var(--warning-foreground)",
        },
        muted: {
          DEFAULT: "var(--muted)",
          foreground: "var(--muted-foreground)",
        },
        accent: {
          DEFAULT: "var(--accent)",
          foreground: "var(--accent-foreground)",
        },
        popover: {
          DEFAULT: "var(--popover)",
          foreground: "var(--popover-foreground)",
        },
        card: {
          DEFAULT: "var(--card)",
          foreground: "var(--card-foreground)",
        },
        logo: "var(--logo)",
        node: {
          entrance: "var(--node-entrance)",
          exit: "var(--node-exit)",
          transformer: "var(--node-transformer)",
          reader: "var(--node-reader)",
          writer: "var(--node-writer)",
          subworkflow: "var(--node-subworkflow)",
          "reader-selected": "var(--node-reader-selected)",
          "writer-selected": "var(--node-writer-selected)",
          "transformer-selected": "var(--node-transformer-selected)",
          "subworkflow-selected": "var(--node-subworkflow-selected)",
        },
        success: "var(--success)",
        canvas: {
          background: "var(--canvas-background)",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      keyframes: {
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
        wave: {
          "0%, 60%, 100%": { transform: "translateY(0)" },
          "30%": { transform: "translateY(-4px)" },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        wave: "wave 1.2s ease-in-out infinite",
      },
    },
  },
  plugins: [
    require("tailwindcss-animate"), // eslint-disable-line
    // Custom theme variants plugin
    plugin(({ addVariant }) => {
      // Add variant for terminal theme
      addVariant("terminal", '[data-theme="terminal"] &');
      // Add variants for future themes
      addVariant("high-contrast", '[data-theme="high-contrast"] &');
      addVariant("midnight", '[data-theme="midnight"] &');
      addVariant("synthwave", '[data-theme="synthwave"] &');
    }),
  ],
} satisfies Config;

export default config;
