import { useMemo, useState } from "react";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components/Select";
import { Slider } from "@flow/components/Slider";

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

  const property = useMemo(
    () => properties.find((p) => p.name === selectedProperty),
    [properties, selectedProperty],
  );

  // Keep all values for smooth sliding, granularity only affects display
  const currentIndex = useMemo(() => {
    if (!property || currentValue === null) return 0;
    const index = property.values.findIndex((v) => v === currentValue);
    return index >= 0 ? index : property.values.length - 1;
  }, [property, currentValue]);

  if (properties.length === 0 || !property) return null;

  const handleSliderChange = (index: number) => {
    if (property) {
      onValueChange(property.values[index]);
    }
  };

  const formatGranularValue = (value: string | number): string => {
    const date = new Date(value);

    if (!isNaN(date.getTime())) {
      if (granularity === "year") {
        return date.getFullYear().toString();
      } else if (granularity === "month") {
        return date.toLocaleDateString(undefined, {
          year: "numeric",
          month: "short",
        });
      } else if (granularity === "hour") {
        return date.toLocaleString(undefined, {
          year: "numeric",
          month: "short",
          day: "numeric",
          hour: "2-digit",
          minute: "2-digit",
        });
      } else {
        // day
        return date.toLocaleDateString();
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
              <SelectItem value="hour">Hour</SelectItem>
              <SelectItem value="day">Day</SelectItem>
              <SelectItem value="month">Month</SelectItem>
              <SelectItem value="year">Year</SelectItem>
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
          {currentIndex + 1} of {property.values.length} ({granularity} view)
        </div>
      </div>

      <div className="space-y-2">
        <Slider
          value={[currentIndex]}
          max={Math.max(0, property.values.length - 1)}
          step={1}
          onValueChange={(values: number[]) => handleSliderChange(values[0])}
        />
        <div className="flex justify-between text-xs text-gray-500">
          <span>
            {property.values.length > 0 ? formatGranularValue(property.values[0]) : "-"}
          </span>
          <span>
            {property.values.length > 0
              ? formatGranularValue(property.values[property.values.length - 1])
              : "-"}
          </span>
        </div>
      </div>
    </div>
  );
};

export default Timeline;
