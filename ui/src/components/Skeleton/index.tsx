import { cn } from "@flow/lib/utils";

function Skeleton({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="skeleton"
      className={cn("animate-pulse rounded bg-accent", className)}
      {...props}
    />
  );
}
export { Skeleton };
