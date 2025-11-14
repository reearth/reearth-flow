import { ChevronDownIcon, ChevronUpIcon } from "@radix-ui/react-icons";
import { useMemo } from "react";

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

type TimeGranularity = "day" | "month" | "year";

type Props = {
  properties: TimelineProperty[];
  selectedProperty: string | null;
  currentValue: string | number | null;
  granularity: TimeGranularity;
  isExpanded: boolean;
  onExpand: (isExpanded: boolean) => void;
  onPropertyChange: (propertyName: string) => void;
  onValueChange: (value: string | number) => void;
  onGranularityChange: (granularity: TimeGranularity) => void;
};

const Timeline: React.FC<Props> = ({
  properties,
  selectedProperty,
  currentValue,
  granularity,
  isExpanded,
  onExpand,
  onPropertyChange,
  onValueChange,
  onGranularityChange,
}) => {
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
      // Detect plain year numbers (number 1900-2100 or string /^\d{4}$/)
      const isPlainYear =
        (typeof value === "number" && value >= 1900 && value <= 2100) ||
        (typeof value === "string" && /^\d{4}$/.test(value));

      let groupKey: string | undefined = undefined;

      if (isPlainYear) {
        // Use the year directly as the group key
        groupKey = String(value);
      } else {
        const date = new Date(value);
        if (!isNaN(date.getTime())) {
          if (granularity === "year") {
            groupKey = date.getFullYear().toString();
          } else if (granularity === "month") {
            // Group by year-month
            groupKey = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;
          } else {
            groupKey = date.toISOString().split("T")[0];
          }
        }
      }

      if (groupKey !== undefined) {
        // Store the most recent value for each group
        groups.set(groupKey, value);
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
      } else {
        return date.toLocaleDateString(language);
      }
    }

    return String(value);
  };

  return (
    <div
      className={`absolute right-0 bottom-0 left-0 z-10 transition-transform duration-300 ${
        isExpanded ? "translate-y-0" : "translate-y-full"
      }`}>
      <div
        onClick={() => onExpand(!isExpanded)}
        className="absolute -top-8 left-1/2 -translate-x-1/2 cursor-pointer rounded-t-md bg-white/90 px-2 py-1 shadow-lg transition-all hover:bg-white">
        {isExpanded ? (
          <ChevronDownIcon className="size-5 text-gray-700" />
        ) : (
          <ChevronUpIcon className="size-5 text-gray-700" />
        )}
      </div>
      <div className="border-t-2 border-gray-200 bg-white/95 px-6 py-4 shadow-2xl backdrop-blur-sm">
        <div className="mx-auto max-w-4xl">
          <div className="mb-4 flex items-center justify-between gap-4">
            <Select
              value={granularity}
              onValueChange={(v) => onGranularityChange(v as TimeGranularity)}>
              <SelectTrigger className="w-32 bg-primary">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="day">{t("Day")}</SelectItem>
                <SelectItem value="month">{t("Month")}</SelectItem>
                <SelectItem value="year">{t("Year")}</SelectItem>
              </SelectContent>
            </Select>
            <div className="flex-1 text-center">
              <div className="text-xl font-bold text-gray-800">
                {currentValue !== null
                  ? formatGranularValue(currentValue)
                  : "-"}
              </div>
            </div>
            {properties.length > 1 ? (
              <Select
                value={selectedProperty || undefined}
                onValueChange={onPropertyChange}>
                <SelectTrigger className="w-40 bg-primary">
                  <SelectValue placeholder={t("Select Property")} />
                </SelectTrigger>
                <SelectContent>
                  {properties.map((prop) => (
                    <SelectItem key={prop.name} value={prop.name}>
                      {prop.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : (
              <div className="w-32" />
            )}
          </div>
          <div className="space-y-2">
            <Slider
              key={`${granularity}-${selectedProperty}`}
              value={[currentIndex]}
              max={Math.max(0, groupedValues.length - 1)}
              step={1}
              onValueChange={(values: number[]) =>
                handleSliderChange(values[0])
              }
            />
            <div className="flex justify-between text-xs text-gray-500">
              <span>
                {groupedValues.length > 0
                  ? formatGranularValue(groupedValues[0])
                  : "-"}
              </span>
              <span>
                {groupedValues.length > 0
                  ? formatGranularValue(groupedValues[groupedValues.length - 1])
                  : "-"}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Timeline;
