// import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@flow/components/resizable";
import { Loading } from "@flow/components/Loading";
import BottomPanel from "@flow/features/BottomPanel";
import Canvas from "@flow/features/Canvas";
// import LeftPanel from "@flow/features/LeftPanel";
import Menubar from "@flow/features/Menubar";
// import RightPanel from "@flow/features/RightPanel";
import { useTimeoutOnLoad } from "@flow/hooks";

import LeftPanel from "./features/LeftPanel";

function App() {
  const { running: isLoading } = useTimeoutOnLoad(1000);

  return (
    <>
      <div className="flex flex-col bg-zinc-900 text-zinc-300 h-screen">
        <div className="flex flex-1">
          <div className="flex flex-col flex-1 p-0">
            <Canvas leftArea={<LeftPanel />} topArea={<Menubar />} />
            <BottomPanel />
          </div>
          {/* <RightPanel /> */}
        </div>
      </div>
      <Loading show={isLoading} />
    </>
    // <div
    //   className="bg-zinc-800"
    //   style={{ display: "flex", flexDirection: "column", height: "100vh" }}>
    //   <MenubarComponent />
    //   <div className="p-4 text-zinc-50">Toolbox</div>
    //   <div style={{ flex: 1 }}>
    //     <ResizablePanelGroup direction="horizontal" className="min-h-[200px]">
    //       <ResizablePanel defaultSize={18} className="min-w-52 bg-zinc-700">
    //         <div className="flex h-full items-center justify-center p-6">
    //           <p className="text-zinc-50">Navigator</p>
    //         </div>
    //       </ResizablePanel>
    //       <ResizableHandle withHandle />
    //       <ResizablePanel defaultSize={70}>
    //         <ResizablePanelGroup direction="vertical" className="min-h-[200px]">
    //           <ResizablePanel defaultSize={85} className="bg-white">
    //             <Canvas />
    //             {/* <div className="flex justify-center p-6">
    //               <h1 className="text-3xl text-slate-200 font-bold underline">Re:Earth Flow</h1>
    //             </div> */}
    //           </ResizablePanel>
    //           <ResizableHandle withHandle />
    //           <ResizablePanel defaultSize={15} className="bg-zinc-700">
    //             <div className="flex h-full gap-4 items-center justify-center p-6">
    //               <p className="text-zinc-50">Visual Preview</p>
    //             </div>
    //           </ResizablePanel>
    //         </ResizablePanelGroup>
    //       </ResizablePanel>
    //       <ResizableHandle withHandle />
    //       <ResizablePanel defaultSize={18} className="min-w-52 bg-zinc-700">
    //         <div className="flex h-full items-center justify-center p-6">
    //           <p className="text-zinc-50">Feature Information</p>
    //         </div>
    //       </ResizablePanel>
    //     </ResizablePanelGroup>
    //   </div>
    // </div>
  );
}

export default App;
