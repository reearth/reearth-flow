import * as React from "react";

import { cn } from "@flow/lib/utils";

export type TextareaProps = React.TextareaHTMLAttributes<HTMLTextAreaElement>;

const TextArea = React.forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ className, ...props }, ref) => {
    return (
      <textarea
        className={cn(
          "flex max-h-[300px] min-h-[40px] w-full rounded-md border bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-primary-foreground focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 dark:placeholder:font-light",
          className,
        )}
        ref={ref}
        {...props}
      />
    );
  },
);

export { TextArea };
