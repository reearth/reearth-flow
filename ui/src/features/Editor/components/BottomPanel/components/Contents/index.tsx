import { useState } from "react";

import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
  supportedVisualizations,
  SupportedVisualizations,
} from "@flow/components";
import LogsConsole from "@flow/features/LogsConsole";
import { useT } from "@flow/lib/i18n";
import mockLogs from "@flow/mock_data/logsv2Data";
import type { Log } from "@flow/types";

import { DataTable } from "../DataTable";
import { Map } from "../Map";

export type ContentID = "output-log" | "visual-preview";

type Props = {
  isOpen: boolean;
  selectedId?: ContentID;
};

const Contents: React.FC<Props> = ({ isOpen, selectedId }) => {
  const t = useT();

  const [visualizationMode, _setVisualizationMode] =
    useState<SupportedVisualizations>("3d-map");

  return (
    <div
      id="content"
      className={`flex h-[calc(100%-64px)] flex-1 bg-background ${isOpen ? "flex" : "hidden"}`}>
      <div className="flex flex-1">
        {selectedId === "visual-preview" ? (
          <ResizablePanelGroup
            className="flex h-full justify-between"
            direction="horizontal">
            <ResizablePanel defaultSize={65} className="">
              <DataTable />
            </ResizablePanel>
            <ResizableHandle className="bg-accent p-px" withHandle />
            <ResizablePanel defaultSize={35}>
              {!supportedVisualizations.includes(visualizationMode) ? (
                <div>
                  <p className="text-center text-accent-foreground">
                    {t("3D visualization is not supported in this browser")}
                  </p>
                </div>
              ) : (
                <Map />
              )}
            </ResizablePanel>
          </ResizablePanelGroup>
        ) : (
          <LogsConsole data={mockLogs as Log[]} />
        )}
      </div>
    </div>
  );
};

export { Contents };
