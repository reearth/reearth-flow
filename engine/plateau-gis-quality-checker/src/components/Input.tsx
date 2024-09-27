import * as React from "react";

import { cn } from "@flow/utils";

export type InputProps = React.InputHTMLAttributes<HTMLInputElement>;

const Input = React.forwardRef<HTMLInputElement, InputProps>(({ className, type, ...props }, ref) => {
  return (
    <input
      type={type}
      className={cn(
        "h-9 rounded-md border border-zinc-600 bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:text-zinc-300 file:bg-transparent file:text-sm file:font-medium placeholder:text-primary-foregroun placeholder:font-light focus-visible:border-primary/50 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
        className,
      )}
      ref={ref}
      {...props}
    />
  );
});
Input.displayName = "Input";

export { Input };
