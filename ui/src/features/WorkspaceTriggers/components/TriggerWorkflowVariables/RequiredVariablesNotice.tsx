import { useT } from "@flow/lib/i18n";

type Props = {
  missing: string[];
  eventSource?: string;
};

// Names the required variables that neither the trigger nor the deployment
// gives a value. A scheduled trigger cannot be saved like this; an API call
// can still supply them, so for an API trigger it is a reminder.
const RequiredVariablesNotice: React.FC<Props> = ({ missing, eventSource }) => {
  const t = useT();
  if (!missing.length) return null;
  const names = missing.join(", ");
  return eventSource === "API_DRIVEN" ? (
    <p className="text-xs text-warning">
      {t("Each API call must provide: {{names}}", { names })}
    </p>
  ) : (
    <p className="text-xs text-destructive">
      {t("Required variables need a value: {{names}}", { names })}
    </p>
  );
};

export { RequiredVariablesNotice };
