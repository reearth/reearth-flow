import { SlidersHorizontal } from "@phosphor-icons/react";
import { ChangeEvent, useState } from "react";

import { Button, FlowLogo, Label, Input, Progress } from "./components";

import "./index.css";

function App() {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [fileContent, setFileContent] = useState<string>("");

  const [workflowProgress, setworkflowProgress] = useState<number>(0);

  const handleFileChange = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0] || null;
    setSelectedFile(file);

    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        const result = e.target?.result as string;
        setFileContent(result);
      };
      reader.readAsText(file);
    }
  };

  const handleRunWorkflow = () => {
    if (selectedFile) {
      setworkflowProgress(10);
      console.log("File to run:", selectedFile);
      console.log("File content:", fileContent);
    }
  };

  return (
    <div className="dark relative flex h-screen flex-col items-center justify-center bg-card p-2 text-zinc-300">
      <div className="absolute inset-x-0 top-0 flex items-center justify-between border-b bg-secondary p-2">
        <div className="float-start box-border flex rounded px-4 py-1">
          <FlowLogo className="" />
        </div>
        <p className="border-b text-xl font-thin text-white">PLATEAU GIS Quality Checker</p>
        <Button size="icon" variant="ghost">
          <SlidersHorizontal size={22} />
        </Button>
      </div>
      <div
        className={`h-[146px] w-[500px] overflow-hidden rounded-lg border bg-secondary transition-all duration-300 ease-linear ${selectedFile ? "h-[338px]" : undefined}`}>
        <div className="flex flex-col justify-center gap-6 p-6">
          <div className="flex flex-col justify-center gap-4">
            <p>Step 1:</p>
            <div className="flex flex-col gap-2 font-thin">
              <Label htmlFor="file">Select a workflow file</Label>
              <Input type="file" onChange={handleFileChange} />
            </div>
          </div>
          <div className="flex flex-col justify-center gap-4">
            <p>Step 2:</p>
            <div className="flex flex-col gap-2 font-thin">
              <Label htmlFor="file">Select some other thing since this is imporant too</Label>
              <Input type="file" onChange={handleFileChange} />
            </div>
          </div>
          <Button className="self-end" variant="outline" onClick={handleRunWorkflow}>
            Run
          </Button>
        </div>
        <Progress className={`${!workflowProgress ? "bg-transparent" : undefined}`} value={workflowProgress} />
      </div>
    </div>
  );
}
export { App };
