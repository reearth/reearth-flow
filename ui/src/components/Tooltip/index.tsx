"use client";

import { Tooltip as TooltipPrimitive } from "@base-ui/react/tooltip";
import * as React from "react";

import { cn } from "@flow/lib/utils";

const TooltipProviderPrimitive = TooltipPrimitive.Provider;

const Tooltip = TooltipPrimitive.Root;

const TooltipTrigger = TooltipPrimitive.Trigger;

const TooltipContent = React.forwardRef<
  React.ElementRef<typeof TooltipPrimitive.Popup>,
  React.ComponentPropsWithoutRef<typeof TooltipPrimitive.Popup> & {
    showArrow?: boolean;
  } & Pick<
      React.ComponentPropsWithoutRef<typeof TooltipPrimitive.Positioner>,
      "side" | "sideOffset" | "align" | "alignOffset"
    >
>(
  (
    {
      className,
      sideOffset = 4,
      side,
      align,
      alignOffset,
      children,
      showArrow,
      ...props
    },
    ref,
  ) => (
    <TooltipPrimitive.Portal>
      <TooltipPrimitive.Positioner
        className="isolate z-50"
        side={side}
        sideOffset={sideOffset}
        align={align}
        alignOffset={alignOffset}>
        <TooltipPrimitive.Popup
          ref={ref}
          className={cn(
            "z-50 rounded-md bg-secondary/70 px-3 py-1.5 text-xs font-light text-secondary-foreground origin-(--transform-origin) transition-[opacity,transform,scale] duration-150 data-ending-style:scale-95 data-ending-style:opacity-0 data-starting-style:scale-95 data-starting-style:opacity-0",
            className,
          )}
          {...props}>
          {children}
          {showArrow && (
            <TooltipPrimitive.Arrow className="size-2 rotate-45 bg-secondary/70" />
          )}
        </TooltipPrimitive.Popup>
      </TooltipPrimitive.Positioner>
    </TooltipPrimitive.Portal>
  ),
);
TooltipContent.displayName = "TooltipContent";

const TooltipProvider = ({ children }: { children?: React.ReactNode }) => {
  return <TooltipProviderPrimitive>{children}</TooltipProviderPrimitive>;
};

export { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider };
