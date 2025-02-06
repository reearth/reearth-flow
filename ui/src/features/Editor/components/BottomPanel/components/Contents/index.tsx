import { useState } from "react";

import {
  supportedVisualizations,
  SupportedVisualizations,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import mockLogs from "@flow/mock_data/logsv2Data";
import { Log } from "@flow/types";

import { DataTable } from "../DataTable";
import { LogsConsole } from "../LogsConsole";
import { Map } from "../Map";

export type ContentID = "output-log" | "visual-preview";

type Props = {
  isOpen: boolean;
  selectedId?: ContentID;
};

const Contents: React.FC<Props> = ({ isOpen, selectedId }) => {
  const t = useT();

  const [visualizationMode, setVisualizationMode] =
    useState<SupportedVisualizations>("2d-map");

  return (
    <div
      id="content"
      className={`flex h-[calc(100%-64px)] flex-1 bg-background ${isOpen ? "flex" : "hidden"}`}>
      <div className="flex flex-1">
        {selectedId === "visual-preview" ? (
          <div className="flex w-full justify-between">
            <DataTable />
            <div className="flex w-1/2">
              {!supportedVisualizations.includes(visualizationMode) ? (
                <div>
                  <p className="text-center text-accent-foreground">
                    {t("3D visualization is not supported in this browser")}
                  </p>
                </div>
              ) : (
                <Map
                  mapMode={visualizationMode}
                  setMapMode={setVisualizationMode}
                />
              )}
            </div>
          </div>
        ) : (
          <LogsConsole data={mockLogs as Log[]} />
        )}
      </div>
    </div>
  );
};

export { Contents };
