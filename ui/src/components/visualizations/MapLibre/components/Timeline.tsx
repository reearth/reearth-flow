import { useMemo } from "react";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components/Select";
import { Slider } from "@flow/components/Slider";

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
  const property = useMemo(
    () => properties.find((p) => p.name === selectedProperty),
    [properties, selectedProperty],
  );

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
    <div className="absolute top-4 right-0.5 z-10 max-w-[500px] -translate-x-3 rounded-md bg-white/90 p-4 shadow-lg">
      <div className="mb-3 flex items-center justify-between">
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
      <div className="mb-2 text-center">
        <div className="text-2xl font-bold text-gray-800">
          {currentValue !== null ? formatTimelineValue(currentValue) : "-"}
        </div>
        <div className="text-xs text-gray-500">
          {currentIndex + 1} of {property.values.length}
        </div>
      </div>
      <div className="space-y-2">
        <Slider
          defaultValue={[currentIndex]}
          max={property.values.length - 1}
          step={1}
          onValueChange={(values: number[]) => handleSliderChange(values[0])}
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
