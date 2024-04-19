import { createRoot } from "react-dom/client";

import App from "./App.tsx";
import "./index.css";
import loadConfig from "./config";

loadConfig().finally(async () => {
  const element = document.getElementById("root");
  if (!element) throw new Error("root element is not found");

  const root = createRoot(element);
  root.render(<App />);
});

// ReactDOM.createRoot(document.getElementById("root")!).render(
//   <React.StrictMode>
//     <App />
//   </React.StrictMode>,
// );
