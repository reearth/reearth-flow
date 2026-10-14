import { useMemo } from "react";

import { useT } from "@flow/lib/i18n";

import type { LicenseGroupKey, LicenseValue } from "./licenses";

// Written out as literal t() calls, rather than looked up by value, so that
// the i18n extractor finds them.
export const useLicenseText = () => {
  const t = useT();

  return useMemo(() => {
    const groupLabels: Record<LicenseGroupKey, string> = {
      open: t("Open — Free to use"),
      shareAlike: t("Share-alike — Derivatives stay open"),
      limited: t("Limited use"),
    };
    const summaries: Record<LicenseValue, string> = {
      "cc-by-4.0": t(
        "Anyone can use, share, and adapt your work for any purpose, including commercially, as long as they credit you.",
      ),
      mit: t(
        "Anyone can use, change, and share your work, including in commercial or closed products – they only keep your copyright notice.",
      ),
      "apache-2.0": t(
        "Like MIT, plus an explicit patent grant for extra legal protection. Anyone can use it commercially; they keep notices and mark any changes.",
      ),
      "o-uda-1.0": t(
        "Anyone can freely use, modify, and share your data for any purpose – they only keep the attribution notice.",
      ),
      "odc-by-1.0": t(
        "Anyone can use and adapt your database, including commercially, as long as they credit you.",
      ),
      "cc-by-sa-4.0": t(
        "Anyone can use and adapt your work, including commercially, but must credit you and release their versions under this same license.",
      ),
      "gpl-3.0": t(
        "Anyone can use and modify your software, but anything they build on it must also be shared as open source under this license.",
      ),
      "lgpl-3.0": t(
        "Others can include your code in their own programs, even closed ones. Only changes to your code itself must be shared as open source.",
      ),
      "eupl-1.2": t(
        "A European open-source license. Anyone can use and modify your work, but their versions must stay open under a compatible license.",
      ),
      "odbl-1.0": t(
        "Anyone can use and adapt your database, including commercially, but must credit you and share any adapted database under this same license.",
      ),
      "cc-by-nd-4.0": t(
        "Anyone can share your work, including commercially, but must credit you and cannot publish modified versions.",
      ),
    };
    return { groupLabels, summaries };
  }, [t]);
};
