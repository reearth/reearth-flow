// import { open } from "@tauri-apps/api/shell";
import { invoke } from "@tauri-apps/api/tauri";
import { ReactNode, useEffect, useState } from "react";
import { debug } from "tauri-plugin-log-api";

import { FileSelector, Button, FlowLogo, Label, WorkflowSelector, Workflow, Loading } from "./components";

import "./index.css";

const workflows: Workflow[] = await invoke("get_quality_check_workflows", {});

function App() {
  const [workflowId, setWorkflowId] = useState<string>("");
  const [cityGmlPath, setCityGmlPath] = useState<string>("");
  const [cityGmlIsDir, setCityGmlIsDir] = useState<boolean>(false);
  const [outputPath, setOutputPath] = useState<string>("");
  const [codelistsPath, setCodelistsPath] = useState<string>("");
  const [schemasPath, setSchemasPath] = useState<string>("");

  const [showOptionalSettings, setShowOptionalSettings] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  // const [isCompleted, setIsCompleted] = useState<boolean>(false);

  useEffect(() => {
    if (isLoading) {
      // TODO: The timeout is needed because Tauri would freeze
      // the app before the loading screen could render
      // on invoking run_flow. After backend is fixed, this
      // timeout should be removed.
      const timeoutId = setTimeout(() => {
        (async () => {
          debug("start flow");
          try {
            const result = await invoke("run_flow", {
              workflowId,
              params: {
                cityGmlPath,
                codelistsPath, // optional: Not necessary if cityGmlPath is Zip
                schemasPath, // optional: Not necessary if cityGmlPath is Zip
                outputPath,
              },
            });
            debug(JSON.stringify(result));
          } catch (e) {
            debug(JSON.stringify(e));
          }
          setIsLoading(false);
          // handleDirectoryOpen(outputPath);
        })();
      }, 500);

      return () => clearTimeout(timeoutId);
    }
  }, [cityGmlPath, codelistsPath, outputPath, schemasPath, workflowId, isLoading]);

  const handleRunWorkflow = () => {
    if (workflowId && cityGmlPath) setIsLoading(true);
  };

  const handleReset = () => {
    setWorkflowId("");
    setCityGmlPath("");
    setOutputPath("");
    setCodelistsPath("");
    setSchemasPath("");
    setShowOptionalSettings(false);
  };

  // const handleDirectoryOpen = async (path: string) => {
  //   try {
  //     await open(path);
  //     console.log(`Opened directory: ${path}`);
  //   } catch (error) {
  //     console.error("Failed to open directory:", error);
  //   }
  // };

  return (
    <div className="dark relative h-screen bg-card text-zinc-300">
      <div className="relative flex h-[53px] items-center justify-between border-b bg-secondary px-4 py-2">
        <div />
        <p className="absolute left-1/2 mx-auto -translate-x-1/2 text-xl font-thin text-white">
          PLATEAU 品質検査ツール
        </p>
        {/* <Button size="icon" variant="ghost">
          <SlidersHorizontal size={22} onClick={handleReset} />
        </Button> */}
        <Button variant="outline" size="sm" onClick={handleReset}>
          <p>リセット</p>
        </Button>
      </div>
      <div className={`flex justify-center gap-4 transition-all`}>
        <div
          className={`flex w-[500px] flex-col overflow-hidden rounded-lg border bg-secondary transition-all duration-300 ease-linear ${outputPath ? (showOptionalSettings ? "mt-[12vh] h-auto w-[800px]" : "mt-[12vh] h-auto") : cityGmlPath ? "mt-[15vh] h-[245px]" : workflowId ? "mt-[20vh] h-[170px]" : "mt-[25vh] h-[98px]"}`}>
          <div className="flex">
            <div className="flex w-full max-w-[500px] flex-col justify-center gap-6 px-6 pt-6">
              {/* <Label className="">一般設定</Label> */}
              <StepWrapper>
                <WorkflowSelector
                  workflows={workflows}
                  selectedWorkflowId={workflowId}
                  onWorkflowIdSelect={setWorkflowId}
                />
              </StepWrapper>
              <StepWrapper>
                <FileSelector
                  label="CityGMLファイル又はディレクトリを選択"
                  selectedPath={cityGmlPath}
                  directorySelect={cityGmlIsDir}
                  onDirectoryToggle={setCityGmlIsDir}
                  onFilePathSelected={setCityGmlPath}
                />
              </StepWrapper>
              <StepWrapper>
                <FileSelector
                  label="アウトプットディレクトリを選択"
                  directorySelect
                  selectedPath={outputPath}
                  onFilePathSelected={setOutputPath}
                />
              </StepWrapper>
              <div className="flex gap-2">
                <Label className="font-thin" htmlFor="任意設定を表示">
                  任意設定を表示
                </Label>
                <input
                  type="checkbox"
                  name="optional"
                  id="optional-settings"
                  checked={showOptionalSettings}
                  onChange={(e) => setShowOptionalSettings(!!e.target.checked)}
                />
              </div>
            </div>
            <div
              className={`flex flex-1 flex-col justify-between py-6 pr-6 ${outputPath && showOptionalSettings ? "block" : "hidden"}`}>
              <div className="flex flex-col gap-4">
                <div className="flex max-w-[272px] flex-col gap-6">
                  <StepWrapper>
                    <FileSelector
                      label="Select a Codelists file"
                      directorySelect
                      selectedPath={codelistsPath}
                      onFilePathSelected={setCodelistsPath}
                    />
                  </StepWrapper>
                  <StepWrapper>
                    <FileSelector
                      label="Select a Schemas file"
                      directorySelect
                      selectedPath={schemasPath}
                      onFilePathSelected={setSchemasPath}
                    />
                  </StepWrapper>
                </div>
              </div>
            </div>
          </div>
          <Button className="mb-6 mr-6 self-end" variant="outline" size="lg" onClick={handleRunWorkflow}>
            実行
          </Button>
        </div>
      </div>
      <Loading show={isLoading} />
    </div>
  );
}
export { App };

const StepWrapper = ({ children }: { children: ReactNode }) => {
  return (
    <div className="flex flex-col justify-center gap-4">
      {/* <p>第{step}</p> */}
      {children}
    </div>
  );
};
