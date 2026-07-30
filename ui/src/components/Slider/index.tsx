import { Slider as SliderPrimitive } from "@base-ui/react/slider";
import * as React from "react";

import { cn } from "@flow/lib/utils";

const Slider = React.forwardRef<
  React.ElementRef<typeof SliderPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof SliderPrimitive.Root>
>(({ className, ...props }, ref) => (
  <SliderPrimitive.Root
    ref={ref}
    thumbAlignment="edge"
    className={cn("relative w-full select-none", className)}
    {...props}>
    <SliderPrimitive.Control className="relative flex w-full touch-none items-center py-1.5 select-none">
      <SliderPrimitive.Track className="relative h-1.5 w-full grow rounded-full bg-muted-foreground/20">
        <SliderPrimitive.Indicator className="absolute h-full rounded-full bg-muted-foreground" />
        <SliderPrimitive.Thumb className="block h-4 w-4 rounded-full border border-primary/50 bg-background shadow transition-colors focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none data-disabled:pointer-events-none data-disabled:opacity-50" />
      </SliderPrimitive.Track>
    </SliderPrimitive.Control>
  </SliderPrimitive.Root>
));
Slider.displayName = "Slider";

export { Slider };
