import { ErrorBoundary } from "react-error-boundary";

import BasicBoiler from "@flow/components/BasicBoiler";
import { FlowLogo } from "@flow/components/icons";

const RenderFallback: React.FC<{
  children: React.ReactNode;
  onError?: () => void;
  message: string;
  textSize: "sm" | "md" | "lg";
}> = ({ children, message, onError, textSize }) => {
  return (
    <ErrorBoundary
      onError={onError}
      fallback={
        <BasicBoiler
          text={message}
          className={`size-4 h-full [&>div>p]:text-${textSize}`}
          icon={<FlowLogo className="size-20 text-accent" />}
        />
      }>
      {children}
    </ErrorBoundary>
  );
};

export { RenderFallback };
