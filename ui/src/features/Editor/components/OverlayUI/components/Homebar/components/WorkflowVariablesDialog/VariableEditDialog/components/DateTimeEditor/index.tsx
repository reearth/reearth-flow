import { Input, Label, Switch } from "@flow/components";
import { DateTimeDefaultValueInput } from "@flow/components/workflowVariables";
import { useT } from "@flow/lib/i18n";
import { WorkflowVariable, DateTimeConfig } from "@flow/types";
import { formatDateOnly } from "@flow/utils";

type Props = {
  variable: WorkflowVariable;
  onUpdate: (variable: WorkflowVariable) => void;
};

export const DateTimeEditor: React.FC<Props> = ({ variable, onUpdate }) => {
  const t = useT();

  // Get datetime config with defaults
  const config = (variable.config as DateTimeConfig) || {};

  const handleConfigChange = (
    configKey: keyof DateTimeConfig,
    value: string | boolean | undefined,
  ) => {
    const newConfig = {
      ...config,
      [configKey]: value,
    };

    onUpdate({
      ...variable,
      config: newConfig,
    });
  };

  const allowTime = config.allowTime !== false; // Default to true

  return (
    <div className="space-y-6">
      {/* Configuration Options */}
      <div>
        <h3 className="mb-4 text-lg font-medium">{t("Configuration")}</h3>

        {/* Allow Time Configuration */}
        <div>
          <Label className="mb-2 block text-sm font-medium">
            {t("Include Time")}
          </Label>
          <div className="flex items-center space-x-2">
            <Switch
              checked={allowTime}
              onCheckedChange={(checked) =>
                handleConfigChange("allowTime", checked)
              }
            />
            <span className="text-sm text-muted-foreground">
              {allowTime ? t("Date and time input") : t("Date only input")}
            </span>
          </div>
          <p className="mt-2 text-sm text-muted-foreground">
            {allowTime
              ? t("Date and time values will use the browser's local timezone")
              : t("Date values are timezone-independent")}
          </p>
        </div>
      </div>

      {/* Date Range Constraints */}
      <div>
        <h3 className="mb-4 text-lg font-medium">{t("Date Range")}</h3>

        <div className="mb-4 grid grid-cols-2 gap-4">
          <div>
            <Label
              htmlFor="min-date"
              className="mb-2 block text-sm font-medium">
              {t("Minimum Date")}
            </Label>
            <Input
              id="min-date"
              type="date"
              value={formatDateOnly(config.minDate)}
              onFocus={(e) => e.stopPropagation()}
              onChange={(e) => {
                const value = e.target.value || undefined;
                handleConfigChange("minDate", value);
              }}
              placeholder={t("No minimum")}
            />
          </div>

          <div>
            <Label
              htmlFor="max-date"
              className="mb-2 block text-sm font-medium">
              {t("Maximum Date")}
            </Label>
            <Input
              id="max-date"
              type="date"
              value={formatDateOnly(config.maxDate)}
              onFocus={(e) => e.stopPropagation()}
              onChange={(e) => {
                const value = e.target.value || undefined;
                handleConfigChange("maxDate", value);
              }}
              placeholder={t("No maximum")}
            />
          </div>
        </div>

        <p className="text-sm text-muted-foreground">
          {t(
            "Set date range constraints for this variable. Leave empty for no constraint.",
          )}
        </p>
      </div>

      <div>
        <h3 className="mb-4 text-lg font-medium">{t("Default Value")}</h3>

        <div>
          <Label
            htmlFor="default-datetime"
            className="mb-2 block text-sm font-medium">
            {allowTime ? t("Default Date & Time") : t("Default Date")}
          </Label>
          <DateTimeDefaultValueInput
            id="default-datetime"
            variable={variable}
            onDefaultValueChange={(newValue) =>
              onUpdate({ ...variable, defaultValue: newValue })
            }
          />
          <p className="mt-1 text-sm text-muted-foreground">
            {allowTime
              ? t(
                  "The default date and time value to use when this variable is not set",
                )
              : t(
                  "The default date value to use when this variable is not set",
                )}
            {(config.minDate || config.maxDate) && (
              <span>
                {" "}
                {config.minDate && config.maxDate
                  ? t("(between {{min}} and {{max}})", {
                      min: config.minDate,
                      max: config.maxDate,
                    })
                  : config.minDate
                    ? t("(after {{min}})", { min: config.minDate })
                    : t("(before {{max}})", { max: config.maxDate })}
              </span>
            )}
          </p>
        </div>
      </div>
    </div>
  );
};
