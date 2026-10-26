import { ArrowSquareOutIcon } from "@phosphor-icons/react";
import type { SyntheticEvent } from "react";
import { useMemo } from "react";

import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import {
  NO_LICENSE_VALUE,
  isLicenseChoice,
  licenseGroups,
  licenseOptions,
  useLicenseText,
} from "@flow/lib/license";
import type { LicenseChoice } from "@flow/lib/license";

type Props = {
  id?: string;
  value?: LicenseChoice;
  placeholder: string;
  // Only creation offers "no license": when editing, clearing the license is
  // just emptying its text.
  withNoLicense?: boolean;
  className?: string;
  onChange: (choice: LicenseChoice) => void;
};

// Opening the license's text must not also pick it.
const stop = (e: SyntheticEvent) => e.stopPropagation();

const LicenseSelect: React.FC<Props> = ({
  id,
  value,
  placeholder,
  withNoLicense,
  className,
  onChange,
}) => {
  const t = useT();
  const { groupLabels, summaries } = useLicenseText();

  const items = useMemo(
    () => [
      { value: NO_LICENSE_VALUE, label: t("No license") },
      ...licenseOptions.map(({ value, label }) => ({ value, label })),
    ],
    [t],
  );

  return (
    <Select
      value={value ?? null}
      items={items}
      onValueChange={(v) => isLicenseChoice(v) && onChange(v)}>
      <SelectTrigger id={id} className={className}>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent className="max-h-80 w-[420px] max-w-[calc(100vw-2rem)] overflow-y-auto">
        {withNoLicense && (
          <>
            <SelectItem value={NO_LICENSE_VALUE}>
              <LicenseOptionBody
                label={t("No license")}
                summary={t(
                  "Keep all rights reserved — you can select a license or create your own later.",
                )}
              />
            </SelectItem>
            <SelectSeparator />
          </>
        )}
        {licenseGroups.map((group) => (
          <SelectGroup key={group.key}>
            <SelectLabel className="text-xs text-muted-foreground">
              {groupLabels[group.key]}
            </SelectLabel>
            {group.licenses.map((license) => (
              <SelectItem key={license.value} value={license.value}>
                <LicenseOptionBody
                  label={license.label}
                  summary={summaries[license.value]}
                  url={license.url}
                  linkLabel={t("View license")}
                />
              </SelectItem>
            ))}
          </SelectGroup>
        ))}
      </SelectContent>
    </Select>
  );
};

const LicenseOptionBody: React.FC<{
  label: string;
  summary: string;
  url?: string;
  linkLabel?: string;
}> = ({ label, summary, url, linkLabel }) => (
  <span className="flex flex-col gap-0.5 py-0.5 whitespace-normal">
    <span>{label}</span>
    <span className="text-xs font-light text-muted-foreground">
      {summary}{" "}
      {url && (
        <a
          href={url}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-0.5 underline underline-offset-2"
          onClick={stop}
          onPointerDown={stop}
          onPointerUp={stop}
          onMouseUp={stop}>
          {linkLabel}
          <ArrowSquareOutIcon className="size-3" />
        </a>
      )}
    </span>
  </span>
);

export { LicenseSelect };
