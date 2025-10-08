import { GearIcon } from "@phosphor-icons/react";
import { MoonIcon, SunIcon } from "@radix-ui/react-icons";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Button,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useTheme } from "@flow/lib/theme";

const ThemeToggle = () => {
  const { theme, setTheme } = useTheme();
  const t = useT();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" className="border-none">
          {theme === "light" && (
            <div className="relative flex items-center justify-between gap-2">
              <SunIcon />
              {t("Light")}
            </div>
          )}
          {theme === "dark" && (
            <div className="relative flex items-center justify-between gap-2">
              <MoonIcon />
              {t("Dark")}
            </div>
          )}
          {theme === "system" && (
            <div className="relative flex items-center justify-between gap-2">
              <GearIcon /> {t("System")}
            </div>
          )}
          <span className="sr-only">{t("Toggle theme")}</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="center">
        <DropdownMenuItem onClick={() => setTheme("light")}>
          <SunIcon />
          {t("Light")}
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme("dark")}>
          <MoonIcon />
          {t("Dark")}
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme("system")}>
          <GearIcon />
          {t("System")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export { ThemeToggle };
