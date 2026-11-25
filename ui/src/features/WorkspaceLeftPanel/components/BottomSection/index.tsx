import {
  ArrowUpRightIcon,
  BookIcon,
  CaretRightIcon,
  FileTextIcon,
  KeyboardIcon,
  PaletteIcon,
} from "@phosphor-icons/react";
import { useState } from "react";
import { useHotkeys } from "react-hotkeys-hook";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@flow/components";
import { config } from "@flow/config";
import { AttributionsDialog } from "@flow/features/common/AttributionsDialog";
import { LegalMenuItems } from "@flow/features/common/LegalMenuItems";
import { ThemeLanguageMenu } from "@flow/features/common/ThemeLanguageMenu";
import KeyboardShortcutDialog from "@flow/features/KeyboardShortcutDialog";
import { GENERAL_HOT_KEYS } from "@flow/global-constants";
import { useT } from "@flow/lib/i18n";
import { openLinkInNewTab } from "@flow/utils";

const rowClass =
  "flex w-full cursor-pointer items-center gap-2 rounded-md px-3 py-[5px] text-sm font-light hover:bg-accent data-popup-open:bg-accent [&_svg]:size-4 [&_svg]:shrink-0";

const flyoutClass = "flex w-[220px] flex-col gap-0.5";

const BottomSection: React.FC = () => {
  const t = useT();
  const { documentationUrl } = config();

  const [openShortcutDialog, setOpenShortcutDialog] = useState(false);
  const [openAttributionsDialog, setOpenAttributionsDialog] = useState(false);

  useHotkeys(GENERAL_HOT_KEYS, () => setOpenShortcutDialog(true));

  return (
    <>
      <div className="flex flex-col gap-1 pb-2">
        <div className="h-px bg-border" />
        {documentationUrl && (
          <button
            type="button"
            className={rowClass}
            onClick={openLinkInNewTab(documentationUrl)}>
            <BookIcon weight="light" />
            <p className="flex-1 truncate text-start">{t("Documentation")}</p>
            <ArrowUpRightIcon weight="light" />
          </button>
        )}
        <button
          type="button"
          className={rowClass}
          onClick={() => setOpenShortcutDialog(true)}>
          <KeyboardIcon weight="light" />
          <p className="flex-1 truncate text-start">
            {t("Keyboard Shortcuts")}
          </p>
        </button>
        <DropdownMenu>
          <DropdownMenuTrigger openOnHover className={rowClass}>
            <PaletteIcon weight="light" />
            <p className="flex-1 truncate text-start">
              {t("Theme & Language")}
            </p>
            <CaretRightIcon weight="light" />
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className={flyoutClass}
            side="right"
            align="end"
            sideOffset={8}>
            <ThemeLanguageMenu />
          </DropdownMenuContent>
        </DropdownMenu>
        <DropdownMenu>
          <DropdownMenuTrigger openOnHover className={rowClass}>
            <FileTextIcon weight="light" />
            <p className="flex-1 truncate text-start">{t("Legal & about")}</p>
            <CaretRightIcon weight="light" />
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className={flyoutClass}
            side="right"
            align="end"
            sideOffset={8}>
            <LegalMenuItems
              onAttributionsOpen={() => setOpenAttributionsDialog(true)}
            />
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
      {openShortcutDialog && (
        <KeyboardShortcutDialog
          isOpen={openShortcutDialog}
          onOpenChange={setOpenShortcutDialog}
        />
      )}
      {openAttributionsDialog && (
        <AttributionsDialog
          isOpen={openAttributionsDialog}
          onOpenChange={setOpenAttributionsDialog}
        />
      )}
    </>
  );
};

export { BottomSection };
