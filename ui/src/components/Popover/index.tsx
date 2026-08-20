import { Popover as PopoverPrimitive } from "@base-ui/react/popover";
import * as React from "react";

import { cn } from "@flow/lib/utils";

const Popover = PopoverPrimitive.Root;

const PopoverTrigger = PopoverPrimitive.Trigger;

const PopoverContent = React.forwardRef<
  React.ElementRef<typeof PopoverPrimitive.Popup>,
  React.ComponentPropsWithoutRef<typeof PopoverPrimitive.Popup> &
    Pick<
      React.ComponentPropsWithoutRef<typeof PopoverPrimitive.Positioner>,
      "side" | "sideOffset" | "align" | "alignOffset" | "collisionPadding"
    >
>(
  (
    {
      className,
      align = "center",
      side,
      sideOffset = 8,
      alignOffset,
      collisionPadding = 8,
      ...props
    },
    ref,
  ) => (
    <PopoverPrimitive.Portal>
      <PopoverPrimitive.Positioner
        className="isolate z-50"
        align={align}
        side={side}
        sideOffset={sideOffset}
        alignOffset={alignOffset}
        collisionPadding={collisionPadding}>
        <PopoverPrimitive.Popup
          ref={ref}
          className={cn(
            "z-50 w-80 origin-(--transform-origin) rounded-md border border-accent bg-primary/50 text-popover-foreground shadow-md backdrop-blur-lg transition-[opacity,transform,scale] duration-200 outline-none data-ending-style:scale-95 data-ending-style:opacity-0 data-starting-style:scale-95 data-starting-style:opacity-0",
            className,
          )}
          {...props}
        />
      </PopoverPrimitive.Positioner>
    </PopoverPrimitive.Portal>
  ),
);
PopoverContent.displayName = "PopoverContent";

export { Popover, PopoverTrigger, PopoverContent };
