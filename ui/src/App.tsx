import { useEffect } from "react";
import { ReactFlowProvider } from "reactflow";

import { Loading } from "@flow/components";
import BottomPanel from "@flow/features/BottomPanel";
import Canvas from "@flow/features/Canvas";
import { Dialog } from "@flow/features/Dialog";
import LeftPanel from "@flow/features/LeftPanel";
import { useTimeoutOnLoad } from "@flow/hooks";

import { workspaces } from "./mock_data/workspaceData";
import { I18nProvider, TooltipProvider } from "./providers";
import { useCurrentProject, useCurrentWorkspace } from "./stores";

function App() {
  const { running: isLoading } = useTimeoutOnLoad(1000);

  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();
  const [currentProject] = useCurrentProject();

  useEffect(() => {
    if (!currentWorkspace) {
      setCurrentWorkspace(workspaces[0]);
    }
  }, [currentWorkspace, setCurrentWorkspace]);

  return (
    <I18nProvider>
      <TooltipProvider>
        <div className="flex flex-col bg-zinc-900 text-zinc-300 h-screen">
          <div className="flex flex-1">
            <div className="flex flex-col flex-1 p-0">
              <ReactFlowProvider>
                <Canvas workflow={currentProject?.workflows?.[0]} leftArea={<LeftPanel />} />
              </ReactFlowProvider>
              <BottomPanel />
            </div>
          </div>
        </div>
        {!isLoading && <Dialog />}
        <Loading show={isLoading} />
      </TooltipProvider>
    </I18nProvider>
  );
}

export default App;
