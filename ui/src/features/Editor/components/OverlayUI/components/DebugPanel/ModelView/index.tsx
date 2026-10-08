import { ArrowLeftIcon, CubeIcon } from "@phosphor-icons/react";
import { memo } from "react";

import { IconButton, RenderFallback } from "@flow/components";
import GlbViewer, {
  BACK_FACE_COLOR,
} from "@flow/components/visualizations/GlbViewer";
import type { IntermediateDataViewError } from "@flow/lib/gql/intermediateDataView";
import { useIntermediateDataView } from "@flow/lib/gql/intermediateDataView";
import { useT } from "@flow/lib/i18n";
import type { IntermediateDataViewRequest } from "@flow/types";

import Legend from "../Legend";
import { Notice, Rendering } from "../ViewStatus";

type Props = {
  request: IntermediateDataViewRequest;
  /** Why the render could not even be asked for, such as too many running. */
  openError?: IntermediateDataViewError;
  onRetry: () => void;
  onBack: () => void;
};

/**
 * One row rendered as a 3D model, from asking for it to showing it. Every
 * state a render can end in says what happened, and asking again is offered
 * only where it can turn out differently.
 */
const ModelView: React.FC<Props> = ({
  request,
  openError,
  onRetry,
  onBack,
}) => {
  const t = useT();
  const { useView, errorMessage } = useIntermediateDataView();
  const { view, error, isRendering, renderStartedAt } = useView(request);

  const body = (() => {
    if (isRendering) {
      return (
        <Rendering
          message={t("Rendering the 3D model…")}
          startedAt={renderStartedAt}
        />
      );
    }
    if (openError) {
      return <Notice message={errorMessage(openError)} onRetry={onRetry} />;
    }
    if (error) {
      // Only a render that ran out of time or failed for a reason the server
      // did not name can go differently a second time.
      const retryable = error.kind === "timedOut" || error.kind === "unknown";
      return (
        <Notice
          message={errorMessage(error)}
          onRetry={retryable ? onRetry : undefined}
        />
      );
    }
    if (!view) return null;

    switch (view.status) {
      case "ready":
        return view.entryPointUrl ? (
          <div className="relative h-full">
            <RenderFallback
              message={t("The 3D model could not be displayed.")}
              textSize="sm">
              <GlbViewer url={view.entryPointUrl} />
            </RenderFallback>
            <Legend
              className="absolute bottom-2 left-2"
              entries={[
                {
                  color: BACK_FACE_COLOR,
                  label: t(
                    "Back of a face: it points inward, or the solid is not closed",
                  ),
                },
              ]}
            />
          </div>
        ) : null;
      case "empty":
        return (
          <Notice
            message={t("This row has no geometry a 3D model can show.")}
            detail={view.error}
          />
        );
      case "unsupportedGeometry":
        return (
          <Notice
            message={t("This row's geometry is 2D, so it has no 3D model.")}
          />
        );
      case "failed":
        return (
          <Notice
            message={t("The 3D model could not be rendered.")}
            detail={view.error}
            onRetry={onRetry}
          />
        );
    }
  })();

  return (
    <div className="flex h-full flex-col rounded-md bg-card/60">
      <div className="flex items-center gap-2 border-b border-border p-2 pl-0">
        <IconButton
          className="h-7 w-7"
          icon={<ArrowLeftIcon size={16} />}
          onClick={onBack}
          tooltipText={t("Back to details")}
        />
        <CubeIcon size={16} />
        <h3 className="text-sm">{t("3D model")}</h3>
      </div>
      <div className="min-h-0 flex-1">{body}</div>
    </div>
  );
};

/** What the viewer's red means, since it marks a defect rather than a style. */
export default memo(ModelView);
