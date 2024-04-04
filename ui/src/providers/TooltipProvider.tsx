import { TooltipProvider as TooltipProviderComponent } from "@flow/components/Tooltip";

const TooltipProvider = ({ children }: { children?: React.ReactNode }) => {
  return <TooltipProviderComponent>{children}</TooltipProviderComponent>;
};

export { TooltipProvider };
