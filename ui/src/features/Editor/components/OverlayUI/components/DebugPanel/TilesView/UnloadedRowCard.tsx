import { memo } from "react";

import { useLang, useT } from "@flow/lib/i18n";

/**
 * In place of the selected row's card when the feature picked on the map is
 * beyond the rows the table has loaded, so there is no row to show.
 */
const UnloadedRowCard: React.FC<{ loadedRows: number }> = ({ loadedRows }) => {
  const t = useT();
  const lang = useLang();
  return (
    <div className="pointer-events-auto w-64 max-w-full rounded-md border border-border bg-card/95 p-2 text-xs text-muted-foreground shadow-md backdrop-blur-sm">
      {t(
        "This feature is not among the {{rows}} rows loaded in the table, so its details cannot be shown.",
        { rows: loadedRows.toLocaleString(lang) },
      )}
    </div>
  );
};

export default memo(UnloadedRowCard);
