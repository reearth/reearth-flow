import { invoke } from "@tauri-apps/api/tauri";
import { debug } from "tauri-plugin-log-api";

import "./App.css";

async function onButtonClick() {
  debug("start flow");
  try {
    const result = await invoke("run_flow", {
      workflowId:
        "Try to get the path from the input file.",
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
  return (
    <div className="App">
      <header className="App-header">
        <p>Hello Vite + React!</p>
        <p>
          Edit <code>App.tsx</code> and save to test HMR updates.
        </p>
        <p>
          <button onClick={onButtonClick}> Flow実行 </button>
          {" | "}
          <a
            className="App-link"
            href="https://vitejs.dev/guide/features.html"
            target="_blank"
            rel="noopener noreferrer">
            Vite Docs
          </a>
        </p>
      </header>
    </div>
  );
}
export { App };
