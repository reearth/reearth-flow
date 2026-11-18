import { useT } from "@flow/lib/i18n";

type Props = {
  missing: string[];
  eventSource?: string;
};

// Names the required variables that neither the trigger nor the deployment
// gives a value. A scheduled trigger cannot be saved like this; an API call
// or a CMS run can still supply them, so for those it is a reminder.
const RequiredVariablesNotice: React.FC<Props> = ({ missing, eventSource }) => {
  const t = useT();
  if (!missing.length) return null;
  const names = missing.join(", ");
  if (eventSource === "API_DRIVEN") {
    return (
      <p className="text-xs text-warning">
        {t("Each API call must provide: {{names}}", { names })}
      </p>
    );
  }
  if (eventSource === "CMS") {
    return (
      <p className="text-xs text-warning">
        {t("Each CMS run must provide: {{names}}", { names })}
      </p>
    );
  }
  return (
    <p className="text-xs text-destructive">
      {t("Required variables need a value: {{names}}", { names })}
    </p>
  );
};

export { RequiredVariablesNotice };
