import {
  ArrowUpRightIcon,
  CopyrightIcon,
  GavelIcon,
} from "@phosphor-icons/react";

import { DropdownMenuItem } from "@flow/components";
import { config } from "@flow/config";
import { menuRowClass } from "@flow/features/common/ThemeLanguageMenu";
import { useT } from "@flow/lib/i18n";
import { openLinkInNewTab } from "@flow/utils";

type Props = {
  onAttributionsOpen: () => void;
};

const LegalMenuItems: React.FC<Props> = ({ onAttributionsOpen }) => {
  const t = useT();
  const { tosUrl } = config();

  return (
    <>
      {tosUrl && (
        <DropdownMenuItem
          className={menuRowClass}
          onClick={openLinkInNewTab(tosUrl)}>
          <GavelIcon weight="light" />
          <p className="flex-1 truncate">{t("Terms of Service")}</p>
          <ArrowUpRightIcon weight="light" />
        </DropdownMenuItem>
      )}
      <DropdownMenuItem className={menuRowClass} onClick={onAttributionsOpen}>
        <CopyrightIcon weight="light" />
        <p className="flex-1 truncate">{t("Attributions")}</p>
      </DropdownMenuItem>
    </>
  );
};

export { LegalMenuItems };
