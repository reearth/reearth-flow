import { useT } from "@flow/lib/i18n";
import type {
  IntermediateDataView,
  IntermediateDataViewRequest,
} from "@flow/types";

import type { IntermediateDataViewError } from "./errors";
import { toIntermediateDataViewError } from "./errors";
import {
  getRenderStartedAt,
  MAX_RENDERS_IN_FLIGHT,
  useQueries,
} from "./useQueries";

export enum IntermediateDataViewQueryKeys {
  View = "intermediateDataView",
}

export type IntermediateDataViewState = {
  /** The view, once the server has answered. Its status may still say there is nothing to show. */
  view?: IntermediateDataView;
  /** Why the view could not be requested at all. */
  error?: IntermediateDataViewError;
  isRendering: boolean;
  /** When the running render started, for showing how long it has taken. */
  renderStartedAt?: number;
};

export const useIntermediateDataView = () => {
  const t = useT();
  const { useViewQuery, fetchView } = useQueries();

  /**
   * The state of the view for `request`, without starting a render. Pass no
   * request while there is nothing to view.
   */
  const useView = (
    request?: IntermediateDataViewRequest,
  ): IntermediateDataViewState => {
    const { data, error, fetchStatus } = useViewQuery(request);
    const isRendering = fetchStatus === "fetching";
    return {
      view: data,
      error: error ? toIntermediateDataViewError(error) : undefined,
      isRendering,
      renderStartedAt:
        isRendering && request ? getRenderStartedAt(request) : undefined,
    };
  };

  /**
   * Asks for the view for `request`, rendering it when the server has none.
   * Rejects with an IntermediateDataViewError when it cannot be asked for.
   */
  const requestView = (
    request: IntermediateDataViewRequest,
  ): Promise<IntermediateDataView> => fetchView(request);

  /** What to tell the user about a view that could not be requested. */
  const errorMessage = (error: IntermediateDataViewError): string => {
    switch (error.kind) {
      case "timedOut":
        return t("Rendering took too long and was stopped.");
      case "unavailable":
        return t("Rendering views is not available in this environment.");
      case "jobNotFinished":
        return t("Views can be rendered once the run has finished.");
      case "dataNotFound":
        return t("This port has no data to render.");
      case "permissionDenied":
        return t("Rendering views needs write access to this workspace.");
      case "tooManyRenders":
        return t(
          "You already have {{max}} renders running. Try again when one finishes.",
          { max: MAX_RENDERS_IN_FLIGHT },
        );
      case "unknown":
        return t("The view could not be rendered.");
    }
  };

  return { useView, requestView, errorMessage };
};
