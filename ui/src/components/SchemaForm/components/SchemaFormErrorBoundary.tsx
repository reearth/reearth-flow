import { Component, ErrorInfo, ReactNode } from "react";

import { Button } from "@flow/components";

type Props = {
  children: ReactNode;
  fallback?: ReactNode;
};

type State = {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
};

export class SchemaFormErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    this.setState({
      error,
      errorInfo,
    });

    // Log error to console in development
    if (process.env.NODE_ENV === "development") {
      console.error(
        "SchemaForm Error Boundary caught an error:",
        error,
        errorInfo,
      );
    }
  }

  handleReset = () => {
    this.setState({ hasError: false, error: undefined, errorInfo: undefined });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="flex flex-col items-center justify-center gap-4 rounded border border-destructive bg-destructive/5 p-6">
          <div className="text-center">
            <h3 className="text-lg font-semibold text-destructive">
              Form Rendering Error
            </h3>
            <p className="text-sm text-muted-foreground">
              There was an issue rendering the form. This might be due to an
              invalid schema or configuration.
            </p>
          </div>

          {process.env.NODE_ENV === "development" && this.state.error && (
            <details className="w-full rounded bg-muted p-3 text-xs">
              <summary className="cursor-pointer font-medium">
                Error Details (Development)
              </summary>
              <pre className="mt-2 break-words whitespace-pre-wrap">
                {this.state.error.toString()}
                {this.state.errorInfo?.componentStack}
              </pre>
            </details>
          )}

          <Button onClick={this.handleReset} variant="outline" size="sm">
            Try Again
          </Button>
        </div>
      );
    }

    return this.props.children;
  }
}
