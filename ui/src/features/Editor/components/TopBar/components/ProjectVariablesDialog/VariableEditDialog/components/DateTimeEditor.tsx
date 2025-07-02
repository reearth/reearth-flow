import {
  Input,
  Label,
  Switch,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable, DateTimeConfig } from "@flow/types";

type Props = {
  variable: ProjectVariable;
  onUpdate: (variable: ProjectVariable) => void;
};

export const DateTimeEditor: React.FC<Props> = ({ variable, onUpdate }) => {
  const t = useT();

  // Get datetime config with defaults
  const config = (variable.config as DateTimeConfig) || {};

  const handleDefaultValueChange = (value: string) => {
    onUpdate({
      ...variable,
      defaultValue: value,
    });
  };

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

  // Format the value for datetime-local or date input based on allowTime setting
  const formatDateTimeValue = (value: any): string => {
    if (!value) return "";

    try {
      const allowTime = config.allowTime !== false; // Default to true

      // If it's already in the correct format, return as-is
      if (typeof value === "string") {
        if (allowTime && value.match(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}/)) {
          return value.slice(0, 16); // YYYY-MM-DDTHH:MM
        }
        if (!allowTime && value.match(/^\d{4}-\d{2}-\d{2}/)) {
          return value.slice(0, 10); // YYYY-MM-DD
        }
      }

      // Try to parse as Date
      const date = new Date(value);
      if (!isNaN(date.getTime())) {
        const year = date.getFullYear();
        const month = String(date.getMonth() + 1).padStart(2, "0");
        const day = String(date.getDate()).padStart(2, "0");

        if (allowTime) {
          const hours = String(date.getHours()).padStart(2, "0");
          const minutes = String(date.getMinutes()).padStart(2, "0");
          return `${year}-${month}-${day}T${hours}:${minutes}`;
        } else {
          return `${year}-${month}-${day}`;
        }
      }
    } catch {
      // If parsing fails, return empty string
    }

    return "";
  };

  // Format date string for min/max date inputs
  const formatDateOnly = (value: string | undefined): string => {
    if (!value) return "";
    try {
      const date = new Date(value);
      if (!isNaN(date.getTime())) {
        const year = date.getFullYear();
        const month = String(date.getMonth() + 1).padStart(2, "0");
        const day = String(date.getDate()).padStart(2, "0");
        return `${year}-${month}-${day}`;
      }
    } catch {
      // If parsing fails, return empty string
    }
    return value?.slice(0, 10) || "";
  };

  // Format min/max values for datetime-local inputs (needs YYYY-MM-DDTHH:MM format)
  const formatDateTimeMinMax = (value: string | undefined): string => {
    if (!value) return "";
    try {
      const date = new Date(value);
      if (!isNaN(date.getTime())) {
        const year = date.getFullYear();
        const month = String(date.getMonth() + 1).padStart(2, "0");
        const day = String(date.getDate()).padStart(2, "0");
        // For min/max on datetime-local, we use 00:00 for min and 23:59 for max to be inclusive
        return `${year}-${month}-${day}T00:00`;
      }
    } catch {
      // If parsing fails, return empty string
    }
    return value?.slice(0, 10) ? `${value.slice(0, 10)}T00:00` : "";
  };

  const formattedValue = formatDateTimeValue(variable.defaultValue);
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

      {/* Default Value */}
      <div>
        <h3 className="mb-4 text-lg font-medium">{t("Default Value")}</h3>

        <div>
          <Label
            htmlFor="default-datetime"
            className="mb-2 block text-sm font-medium">
            {allowTime ? t("Default Date & Time") : t("Default Date")}
          </Label>
          <Input
            id="default-datetime"
            type={allowTime ? "datetime-local" : "date"}
            value={formattedValue}
            onFocus={(e) => e.stopPropagation()}
            onChange={(e) => handleDefaultValueChange(e.target.value)}
            min={allowTime 
              ? (config.minDate ? formatDateTimeMinMax(config.minDate) : undefined)
              : (config.minDate ? formatDateOnly(config.minDate) : undefined)
            }
            max={allowTime 
              ? (config.maxDate ? formatDateTimeMinMax(config.maxDate) : undefined)
              : (config.maxDate ? formatDateOnly(config.maxDate) : undefined)
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
