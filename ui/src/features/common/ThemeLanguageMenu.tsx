import {
  CheckIcon,
  GearIcon,
  MoonIcon,
  MoonStarsIcon,
  SunIcon,
  TerminalWindowIcon,
  WaveformIcon,
} from "@phosphor-icons/react";

import {
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
} from "@flow/components";
import { useUser } from "@flow/lib/gql";
import type { AvailableLanguage } from "@flow/lib/i18n";
import { localesWithLabel, useLang, useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";
import type { Theme } from "@flow/lib/theme";
import { useTheme } from "@flow/lib/theme";

const themeIcons: Record<Theme, React.ReactNode> = {
  light: <SunIcon weight="light" />,
  dark: <MoonIcon weight="light" />,
  midnight: <MoonStarsIcon weight="light" />,
  synthwave: <WaveformIcon weight="light" />,
  terminal: <TerminalWindowIcon weight="light" />,
  system: <GearIcon weight="light" />,
};

const languageOrder: AvailableLanguage[] = [
  "en",
  "ja",
  // "zh",
  // "fr",
  // "es",
];

export const menuRowClass = "gap-3 rounded-md px-4 py-[5px] font-light";
export const menuGroupLabelClass =
  "px-4 pt-1.5 pb-0.5 text-xs font-light text-muted-foreground";

const ThemeLanguageMenu: React.FC = () => {
  const t = useT();
  const lang = useLang();
  const { theme, setTheme } = useTheme();
  const { updateMe } = useUser();

  const themeLabels: Record<Theme, string> = {
    light: t("Light"),
    dark: t("Dark"),
    midnight: t("Midnight"),
    synthwave: t("Synthwave"),
    terminal: t("Terminal"),
    system: t("System"),
  };

  const handleLanguageChange = (language: AvailableLanguage) => {
    i18n.changeLanguage(language);
    updateMe({ lang: language });
  };

  return (
    <>
      <DropdownMenuLabel className={menuGroupLabelClass}>
        {t("Theme")}
      </DropdownMenuLabel>
      {(Object.keys(themeLabels) as Theme[]).map((value) => (
        <DropdownMenuItem
          key={value}
          className={menuRowClass}
          onClick={() => setTheme(value)}>
          {themeIcons[value]}
          <p className="flex-1 truncate">{themeLabels[value]}</p>
          {theme === value && <CheckIcon weight="light" />}
        </DropdownMenuItem>
      ))}
      <DropdownMenuSeparator className="mx-0 my-0" />
      <DropdownMenuLabel className={menuGroupLabelClass}>
        {t("Language")}
      </DropdownMenuLabel>
      {languageOrder.map((language) => (
        <DropdownMenuItem
          key={language}
          className={menuRowClass}
          onClick={() => handleLanguageChange(language)}>
          <p className="flex-1 truncate">{localesWithLabel[language]}</p>
          {lang === language && <CheckIcon weight="light" />}
        </DropdownMenuItem>
      ))}
    </>
  );
};

export { ThemeLanguageMenu };
