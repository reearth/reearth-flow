import * as React from "react";
import { useMemo } from "react";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components/Select";

import type { TimelineProperty } from "../utils/timelineUtils";
import { formatTimelineValue } from "../utils/timelineUtils";

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
  // Get the current property data
  const property = useMemo(
    () => properties.find((p) => p.name === selectedProperty),
    [properties, selectedProperty],
  );

  // Find current value index
  const currentIndex = useMemo(() => {
    if (!property || currentValue === null) return 0;
    return property.values.findIndex((v) => v === currentValue);
  }, [property, currentValue]);

  if (properties.length === 0 || !property) return null;

  const handleSliderChange = (index: number) => {
    if (property) {
      onValueChange(property.values[index]);
    }
  };

  return (
    <div className="absolute top-4 right-0.5 z-10 max-w-[500px] -translate-x-1/2 rounded-md bg-white p-4 shadow-lg">
      <div className="mb-3 flex items-center justify-between">
        {properties.length > 1 && (
          <Select
            value={selectedProperty || undefined}
            onValueChange={onPropertyChange}>
            <SelectTrigger className="w-40">
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

      <div className="mb-2 text-center">
        <div className="text-2xl font-bold text-gray-800">
          {currentValue !== null ? formatTimelineValue(currentValue) : "-"}
        </div>
        <div className="text-xs text-gray-500">
          {currentIndex + 1} of {property.values.length}
        </div>
      </div>

      <div className="space-y-2">
        <input
          type="range"
          min={0}
          max={property.values.length - 1}
          value={currentIndex}
          step={1}
          onChange={(e) => handleSliderChange(Number(e.target.value))}
          className="h-2 w-full cursor-pointer appearance-none rounded-lg bg-gray-200 accent-blue-600"
        />

        <div className="flex justify-between text-xs text-gray-500">
          <span>{formatTimelineValue(property.min)}</span>
          <span>{formatTimelineValue(property.max)}</span>
        </div>
      </div>
    </div>
  );
};

export default Timeline;
