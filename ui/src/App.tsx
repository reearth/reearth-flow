import { Button } from '@flow/components/ui/button'
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@flow/components/ui/resizable";

import MenubarComponent from '@flow/features/Menubar';

function App() {
  return (
    <div style={{ background: "#343536", height: "100vh" }}>
      <MenubarComponent />
      <ResizablePanelGroup
        direction="horizontal"
        className="min-h-[200px] rounded-lg border"
      >
        <ResizablePanel defaultSize={25} className="min-w-52">
          <div className="flex h-full items-center justify-center p-6">
            <Button size="sm">Styles are working</Button>
          </div>
        </ResizablePanel>
        <ResizableHandle />
        <ResizablePanel defaultSize={50}>
          <ResizablePanelGroup
            direction="vertical"
            className="min-h-[200px] rounded-lg border"
          >
            <ResizablePanel defaultSize={80}>

              <div className="flex justify-center p-6">
                <h1 className="text-3xl text-slate-200 font-bold underline">Re:Earth Flow</h1>
              </div>
            </ResizablePanel>
            <ResizableHandle />
            <ResizablePanel defaultSize={20}>
              <div className="flex h-full items-center justify-center p-6">
                <Button size="sm">Styles are working</Button>
              </div>
            </ResizablePanel>
          </ResizablePanelGroup>
        </ResizablePanel>
        <ResizableHandle />
        <ResizablePanel defaultSize={25} className="min-w-52">
          <div className="flex h-full justify-center p-3 gap-3">
            <Button size="sm">Styles are working</Button>
            <Button size="sm">Styles are working</Button>
          </div>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  )
}

export default App

