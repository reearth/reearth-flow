import { TooltipProvider as TooltipProviderComponent } from "@flow/components";

const TooltipProvider = ({ children }: { children?: React.ReactNode }) => {
  return <TooltipProviderComponent>{children}</TooltipProviderComponent>;
};

export { TooltipProvider };
