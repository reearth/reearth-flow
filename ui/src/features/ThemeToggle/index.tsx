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
  );
};

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
