import { createFileRoute } from "@tanstack/react-router";
import { TooltipProvider } from "@radix-ui/react-tooltip";

import { DocumentApiTest } from "@flow/components/DocumentApiTest";
import { ThemeProvider } from "@flow/lib/theme";
import { I18nProvider } from "@flow/lib/i18n";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const queryClient = new QueryClient();

export const Route = createFileRoute("/test-document-api")({
  component: () => (
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <I18nProvider>
          <TooltipProvider>
            <DocumentApiTest />
          </TooltipProvider>
        </I18nProvider>
      </QueryClientProvider>
    </ThemeProvider>
  ),
}); 