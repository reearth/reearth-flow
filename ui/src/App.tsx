import { Loading } from "@flow/components";
import BottomPanel from "@flow/features/BottomPanel";
import Canvas from "@flow/features/Canvas";
import { Dialog } from "@flow/features/Dialog";
import LeftPanel from "@flow/features/LeftPanel";
// import RightPanel from "@flow/features/RightPanel";
import { useTimeoutOnLoad } from "@flow/hooks";

import { I18nProvider, TooltipProvider } from "./providers";

function App() {
  const { running: isLoading } = useTimeoutOnLoad(1000);

  return (
    <I18nProvider>
      <TooltipProvider>
        <div className="flex flex-col bg-zinc-900 text-zinc-300 h-screen">
          <div className="flex flex-1">
            <div className="flex flex-col flex-1 p-0">
              <Canvas leftArea={<LeftPanel />} />
              <BottomPanel />
            </div>
            {/* <RightPanel /> */}
          </div>
        </div>
        <Dialog />
        <Loading show={isLoading} />
      </TooltipProvider>
    </I18nProvider>
  );
}

export default App;
