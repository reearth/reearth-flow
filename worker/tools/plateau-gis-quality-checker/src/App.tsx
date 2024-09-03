import { CaretDown, SlidersHorizontal } from "@phosphor-icons/react";
import { invoke } from "@tauri-apps/api/tauri";
import { ReactNode, useState } from "react";
import { debug } from "tauri-plugin-log-api";

import { FileSelector, Button, FlowLogo } from "./components";

import "./index.css";

async function onButtonClick() {
  debug("start flow");
  try {
    const result = await invoke("run_flow", {
      workflowId: "Try to get the path from the input file.",
      params: {
        cityGmlPath: "Try to get the path from the input file.",
        codelistsPath: "Try to get the path from the input file.", // optional: Not necessary if cityGmlPath is Zip
        schemasPath: "Try to get the path from the input file.", // optional: Not necessary if cityGmlPath is Zip
        outputPath: "Try to get the path from the input file.",
      },
    });
    debug(JSON.stringify(result));
  } catch (e) {
    debug(JSON.stringify(e));
  }
}

function App() {
  const [workflowPath, setWorkflowPath] = useState<string>("");
  const [cityGmlPath, setCityGmlPath] = useState<string>("");
  const [outputPath, setOutputPath] = useState<string>("");
  const [codelistsPath, setCodelistsPath] = useState<string>("");
  const [schemasPath, setSchemasPath] = useState<string>("");

  const [showOptionalSettings, setShowOptionalSettings] = useState<boolean>(false);

  // const [workflowProgress, setworkflowProgress] = useState<number>(0);

  const handleRunWorkflow = () => {
    if (workflowPath && cityGmlPath) {
      // setworkflowProgress(10);
      console.log("workflowPath", workflowPath);
      console.log("cityGmlPath", cityGmlPath);
      console.log("codelistsPath", codelistsPath);
      console.log("schemasPath", schemasPath);
      console.log("outputPath", outputPath);
    }
  };

  console.log("workflowPath", workflowPath);

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
        className={`h-[142px] w-[500px] overflow-hidden rounded-lg border bg-secondary transition-all duration-300 ease-linear ${outputPath ? (showOptionalSettings ? "h-[699px]" : "h-[469px]") : cityGmlPath ? "h-[369px]" : workflowPath ? "h-[256px]" : undefined}`}>
        <div className="flex flex-col justify-center gap-6 p-6">
          <StepWrapper step="1">
            <FileSelector label="ワークフローファイルを選択" onFilePathSelected={setWorkflowPath} />
          </StepWrapper>
          <StepWrapper step="2">
            <FileSelector label="CityGMLファイルを選択" onFilePathSelected={setCityGmlPath} />
          </StepWrapper>
          <StepWrapper step="3">
            <FileSelector label="Select an output directory" directorySelect onFilePathSelected={setOutputPath} />
          </StepWrapper>
          {/* <div> */}
          <div className="flex items-center justify-between" onClick={() => setShowOptionalSettings((o) => !o)}>
            <p className="text-sm font-thin">任意設定</p>
            <CaretDown />
          </div>
          <div className={`flex flex-col gap-6 ${showOptionalSettings ? "block" : "hidden"}`}>
            <StepWrapper step="4">
              <FileSelector label="Select a Codelists file" onFilePathSelected={setCodelistsPath} />
            </StepWrapper>
            <StepWrapper step="5">
              <FileSelector label="Select a Schemas file" onFilePathSelected={setSchemasPath} />
            </StepWrapper>
          </div>
          {/* </div> */}
          <Button className="self-end" variant="outline" onClick={handleRunWorkflow}>
            発行
          </Button>
        </div>
        {/* <Progress className={`${!workflowProgress ? "bg-transparent" : undefined}`} value={workflowProgress} /> */}
      </div>
    </div>
  );
}
export { App };

const StepWrapper = ({ step, children }: { step: string; children: ReactNode }) => {
  return (
    <div className="flex flex-col justify-center gap-4">
      <p>第{step}</p>
      {children}
    </div>
  );
};
