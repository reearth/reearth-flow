import type { Config } from "tailwindcss";

const config = {
  darkMode: ["class"],
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
        border: "rgba(var(--border))",
        input: "rgba(var(--input))",
        ring: "rgba(var(--ring))",
        background: "rgba(var(--background))",
        foreground: "rgba(var(--foreground))",
        primary: {
          DEFAULT: "rgba(var(--primary))",
          foreground: "rgba(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "rgba(var(--secondary))",
          foreground: "rgba(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "rgba(var(--destructive))",
          foreground: "rgba(var(--destructive-foreground))",
        },
        warning: {
          DEFAULT: "rgba(var(--warning))",
          foreground: "rgba(var(--warning-foreground))",
        },
        muted: {
          DEFAULT: "rgba(var(--muted))",
          foreground: "rgba(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "rgba(var(--accent))",
          foreground: "rgba(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "rgba(var(--popover))",
          foreground: "rgba(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "rgba(var(--card))",
          foreground: "rgba(var(--card-foreground))",
        },
        logo: "rgba(var(--logo))",
        node: {
          entrance: "rgba(var(--node-entrance))",
          exit: "rgba(var(--node-exit))",
          transformer: "rgba(var(--node-transformer))",
          reader: "rgba(var(--node-reader))",
          writer: "rgba(var(--node-writer))",
          subworkflow: "rgba(var(--node-subworkflow))",
          "reader-selected": "rgba(var(--node-reader-selected))",
          "writer-selected": "rgba(var(--node-writer-selected))",
          "transformer-selected": "rgba(var(--node-transformer-selected))",
          "subworkflow-selected": "rgba(var(--node-subworkflow-selected))",
        },
        success: "rgba(var(--success))",
        canvas: {
          background: "rgba(var(--canvas-background))",
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
  plugins: [require("tailwindcss-animate")], // eslint-disable-line
  safelist: ["line-clamp-2", "loading-pulse"],
} satisfies Config;

export default config;
