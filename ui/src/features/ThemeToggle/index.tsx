import { GearIcon } from "@phosphor-icons/react";
import { MoonIcon, SunIcon } from "@radix-ui/react-icons";

import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useTheme } from "@flow/lib/theme";

const ThemeToggle = () => {
  const { theme, setTheme } = useTheme();
  const t = useT();

  const themes = [
    { value: "light", label: t("Light"), icon: <SunIcon /> },
    { value: "dark", label: t("Dark"), icon: <MoonIcon /> },
    { value: "system", label: t("System"), icon: <GearIcon /> },
  ];

  const currentTheme = themes.filter((t) => t.value === theme)[0];

  const handleThemeChange = (theme: "light" | "dark" | "system") => {
    setTheme(theme);
  };
  return (
    <>
      {/* <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" className="border-none">
            {theme === "light" && (
              <div className="flex items-center justify-between gap-2">
                <SunIcon />
                {t("Light")}
              </div>
            )}
            {theme === "dark" && (
              <div className="flex items-center justify-between gap-2">
                <MoonIcon />
                {t("Dark")}
              </div>
            )}
            {theme === "system" && (
              <div className="flex items-center justify-between gap-2">
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
      </DropdownMenu> */}

      <Select onValueChange={handleThemeChange}>
        <SelectTrigger>
          <SelectValue placeholder={<CurrentTheme theme={currentTheme} />} />
        </SelectTrigger>
        <SelectContent>
          {themes.map((theme) => (
            <SelectItem key={theme.value} value={theme.value}>
              <div className="flex items-center justify-between gap-2">
                {theme.icon}
                {theme.label}
              </div>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </>
  );
};
// Unsure if we want to keep the drop down or have select menu to match the other styles, so have commented out for now.
const CurrentTheme = ({
  theme,
}: {
  theme: { icon: React.ReactNode; label: string };
}) => {
  return (
    <div className="flex items-center justify-between gap-2">
      {theme.icon}
      {theme.label}
    </div>
  );
};

export { ThemeToggle };
