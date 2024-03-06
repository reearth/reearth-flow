import { useState } from "react";

// import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@flow/components/resizable";
import MenubarComponent from "@flow/features/Menubar";

import { CollapsibleSidebar } from "./components";
import Canvas from "./features/Canvas";

function App() {
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const [isBottomBarOpen, setIsBottomBarOpen] = useState(false);

  const toggleSidebar = () => {
    setIsSidebarOpen(!isSidebarOpen);
  };

  const toggleBottombar = () => {
    setIsBottomBarOpen(!isBottomBarOpen);
  };

  return (
    <div className="flex flex-col bg-zinc-900 h-screen">
      <MenubarComponent />
      <div className="flex flex-1">
        <CollapsibleSidebar
          className="border-r border-zinc-800"
          isOpen={isSidebarOpen}
          toggleSidebar={toggleSidebar}>
          <p>Some content</p>
        </CollapsibleSidebar>
        <div className="flex flex-col flex-1">
          <div className="flex-1 p-2">
            <Canvas />
          </div>
          <CollapsibleSidebar
            className="bg-zinc-950"
            direction="horizontal"
            isOpen={isBottomBarOpen}
            minHeight="h-[25px]"
            maxHeight="h-[100px]"
            toggleSidebar={toggleBottombar}>
            <div className="flex items-center gap-6 ml-[18px]">
              <p className="text-[14px]">Deploy</p>
              <p className="text-[14px]">Build</p>
              <p className="text-[14px]">Build Scheduler</p>
              <p className="text-[14px]">Alerts</p>
            </div>
          </CollapsibleSidebar>
        </div>
      </div>
    </div>

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
