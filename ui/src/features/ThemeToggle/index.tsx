import { MoonIcon, SunIcon } from "@radix-ui/react-icons";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Button,
} from "@flow/components";
import { useTheme } from "@flow/components/ThemeProvider";
import { useT } from "@flow/lib/i18n";

const ThemeToggle = () => {
  const { setTheme } = useTheme();
  const t = useT();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        {/* TODO: Disabling till the light mode colors are fixed  */}
        <Button variant="outline" size="icon" className="border-none" disabled>
          <SunIcon className="size-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
          <MoonIcon className="absolute size-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
          <span className="sr-only">{t("Toggle theme")}</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onClick={() => setTheme("light")}>{t("Light")}</DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme("dark")}>{t("Dark")}</DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme("system")}>{t("System")}</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export { ThemeToggle };
