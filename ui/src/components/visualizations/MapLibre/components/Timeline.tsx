import { useMemo, useState } from "react";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components/Select";
import { Slider } from "@flow/components/Slider";
import { useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";

import type { TimelineProperty } from "../utils/timelineUtils";

type TimeGranularity = "hour" | "day" | "month" | "year";

type Props = {
  properties: TimelineProperty[];
  selectedProperty: string | null;
  currentValue: string | number | null;
  onPropertyChange: (propertyName: string) => void;
  onValueChange: (value: string | number) => void;
};

const Timeline: React.FC<Props> = ({
  properties,
  selectedProperty,
  currentValue,
  onPropertyChange,
  onValueChange,
}) => {
  const [granularity, setGranularity] = useState<TimeGranularity>("day");
  const t = useT();
  const language = i18n.language;

  const property = useMemo(
    () => properties.find((p) => p.name === selectedProperty),
    [properties, selectedProperty],
  );

  // Group values by selected granularity
  const groupedValues = useMemo(() => {
    if (!property) return [];

    const groups = new Map<string, string | number>();

    property.values.forEach((value) => {
      const date = new Date(value);

      if (!isNaN(date.getTime())) {
        let groupKey: string;

        if (granularity === "year") {
          groupKey = date.getFullYear().toString();
        } else if (granularity === "month") {
          // Group by year-month
          groupKey = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;
        } else if (granularity === "day") {
          // Group by year-month-day
          groupKey = date.toISOString().split("T")[0];
        } else {
          // hour - Group by year-month-day-hour
          groupKey = `${date.toISOString().split("T")[0]}T${String(date.getHours()).padStart(2, "0")}`;
        }

        // Store the most recent value for each group
        groups.set(groupKey, value);
      } else {
        // Not a date, use as-is (likely a year number)
        groups.set(String(value), value);
      }
    });

    return Array.from(groups.entries())
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([, value]) => value);
  }, [property, granularity]);

  // Find current value index in grouped values
  const currentIndex = useMemo(() => {
    if (!groupedValues.length || currentValue === null) return 0;
    const index = groupedValues.findIndex((v) => v === currentValue);
    return index >= 0 ? index : groupedValues.length - 1;
  }, [groupedValues, currentValue]);

  if (properties.length === 0 || !property) return null;

  const handleSliderChange = (index: number) => {
    if (groupedValues.length > 0) {
      onValueChange(groupedValues[index]);
    }
  };

  const formatGranularValue = (value: string | number): string => {
    const date = new Date(value);

    if (!isNaN(date.getTime())) {
      if (granularity === "year") {
        return date.toLocaleDateString(language, { year: "numeric" });
      } else if (granularity === "month") {
        return date.toLocaleDateString(language, {
          year: "numeric",
          month: "short",
        });
      } else if (granularity === "hour") {
        return date.toLocaleString(language, {
          year: "numeric",
          month: "short",
          day: "numeric",
          hour: "2-digit",
          minute: "2-digit",
        });
      } else {
        return date.toLocaleDateString(language);
      }
    }

    return String(value);
  };

  return (
    <div className="absolute top-4 right-0.5 z-10 max-w-[500px] -translate-x-3 rounded-md bg-white/90 p-4 shadow-lg">
      <div className="mb-3 flex flex-col gap-2">
        <div className="flex items-center justify-between gap-2">
          {/* Granularity selector */}
          <Select
            value={granularity}
            onValueChange={(v) => setGranularity(v as TimeGranularity)}>
            <SelectTrigger className="w-28 bg-primary">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="hour">{t("Hour")}</SelectItem>
              <SelectItem value="day">{t("Day")}</SelectItem>
              <SelectItem value="month">{t("Month")}</SelectItem>
              <SelectItem value="year">{t("Year")}</SelectItem>
            </SelectContent>
          </Select>

          {/* Property selector (if multiple properties) */}
          {properties.length > 1 && (
            <Select
              value={selectedProperty || undefined}
              onValueChange={onPropertyChange}>
              <SelectTrigger className="w-40 bg-primary">
                <SelectValue placeholder="Select property" />
              </SelectTrigger>
              <SelectContent>
                {properties.map((prop) => (
                  <SelectItem key={prop.name} value={prop.name}>
                    {prop.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
        </div>
      </div>

      <div className="mb-2 text-center">
        <div className="text-2xl font-bold text-gray-800">
          {currentValue !== null ? formatGranularValue(currentValue) : "-"}
        </div>
        <div className="text-xs text-gray-500">
          {currentIndex + 1} of {groupedValues.length} {granularity}s
        </div>
      </div>

      <div className="space-y-2">
        <Slider
          key={`${granularity}-${selectedProperty}`}
          value={[currentIndex]}
          max={Math.max(0, groupedValues.length - 1)}
          step={1}
          onValueChange={(values: number[]) => handleSliderChange(values[0])}
        />
      </div>
    </div>
  );
};

export default Timeline;
