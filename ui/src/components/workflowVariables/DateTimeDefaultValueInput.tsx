import { Input } from "@flow/components";
import type { DateTimeConfig, WorkflowVariable } from "@flow/types";
import { formatDateOnly } from "@flow/utils";

type Props = {
  id?: string;
  variable: Pick<WorkflowVariable, "defaultValue" | "config">;
  onDefaultValueChange: (newValue: any) => void;
};

export const DateTimeDefaultValueInput: React.FC<Props> = ({
  id = "default-datetime",
  variable,
  onDefaultValueChange,
}) => {
  const config = (variable.config as DateTimeConfig) || {};
  const allowTime = config.allowTime !== false; // Default to true

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

  const formattedValue = formatDateTimeValue(variable.defaultValue);

  const handleDefaultValueChange = (value: string) => {
    let storedValue = value;

    // datetime-local input returns YYYY-MM-DDTHH:MM — add seconds and local timezone
    if (allowTime && value && /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(value)) {
      const offset = -new Date().getTimezoneOffset(); // minutes ahead of UTC
      if (offset === 0) {
        storedValue = `${value}:00Z`;
      } else {
        const sign = offset >= 0 ? "+" : "-";
        const h = Math.floor(Math.abs(offset) / 60)
          .toString()
          .padStart(2, "0");
        const m = (Math.abs(offset) % 60).toString().padStart(2, "0");
        storedValue = `${value}:00${sign}${h}:${m}`;
      }
    }

    onDefaultValueChange(storedValue);
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

  return (
    <Input
      id={id}
      type={allowTime ? "datetime-local" : "date"}
      value={formattedValue}
      onFocus={(e) => e.stopPropagation()}
      onChange={(e) => handleDefaultValueChange(e.target.value)}
      min={
        allowTime
          ? config.minDate
            ? formatDateTimeMinMax(config.minDate)
            : undefined
          : config.minDate
            ? formatDateOnly(config.minDate)
            : undefined
      }
      max={
        allowTime
          ? config.maxDate
            ? formatDateTimeMinMax(config.maxDate)
            : undefined
          : config.maxDate
            ? formatDateOnly(config.maxDate)
            : undefined
      }
    />
  );
};

export default DateTimeDefaultValueInput;
