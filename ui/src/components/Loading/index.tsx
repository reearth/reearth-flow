import { DoubleArrowRightIcon } from "@radix-ui/react-icons";
import * as ProgressPrimitive from "@radix-ui/react-progress";
import { forwardRef, useEffect, useState } from "react";

import { cn } from "@flow/lib/utils";

import { FlowLogo } from "..";

import "./styles.css";

const Loading: React.FC<{ show?: boolean }> = () => {
  const [fakeLoadValue, setFakeLoadValue] = useState(0);
  const [show, setShow] = useState(true);

  useEffect(() => {
    if (!show) return;
    const intervalId = setInterval(() => {
      if (fakeLoadValue < 100) {
        setFakeLoadValue(currentValue => currentValue + 10);
      } else {
        clearInterval(intervalId);
      }
    }, 80);

    return () => clearInterval(intervalId);
  }, [fakeLoadValue, show]);

  useEffect(() => {
    if (fakeLoadValue === 100) {
      const intervalId = setInterval(() => {
        setShow(false);
      }, 500);

      return () => clearInterval(intervalId);
    }
  }, [fakeLoadValue]);

  return (
    show && (
      <div className="absolute top-0 z-40 flex justify-center h-[100vh] w-full bg-zinc-900">
        <div className="flex items-center h-full">
          <div className="flex flex-col gap-5">
            <div className="flex gap-3">
              <FlowLogo
                id="loading-svg"
                className="bg-red-900 bg-opacity-50 text-zinc-200 rounded-lg p-1 mb-8"
                style={{ height: "110px", width: "110px" }}
              />
              <DoubleArrowRightIcon className="w-[110px] h-[110px] text-zinc-600" />
            </div>
            <Progress value={fakeLoadValue} />
          </div>
        </div>
        {/* <p id="loading-text" className="text-2xl text-zinc-500">
        Loading...
      </p> */}
      </div>
    )
  );
};

export { Loading };

const Progress = forwardRef<
  React.ElementRef<typeof ProgressPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof ProgressPrimitive.Root>
>(({ className, value, ...props }, ref) => (
  <ProgressPrimitive.Root
    ref={ref}
    className={cn("relative h-1 w-full overflow-hidden rounded-full bg-zinc-800", className)}
    {...props}>
    <ProgressPrimitive.Indicator
      className="h-full w-full flex-1 bg-zinc-500 transition-all"
      style={{ transform: `translateX(-${100 - (value || 0)}%)` }}
    />
  </ProgressPrimitive.Root>
));
Progress.displayName = ProgressPrimitive.Root.displayName;

export { Progress };
