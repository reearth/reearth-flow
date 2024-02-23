import { Button } from "@flow/components";
import { Menubar } from "@flow/components/menubar";

import Github from "../../stories_examples/assets/github.svg";

import EditMenu from "./components/Edit";
import FileMenu from "./components/File";
import HelpMenu from "./components/Help";
import ReadersMenu from "./components/Readers";
import RunMenu from "./components/Run";
import ToolsMenu from "./components/Tools";
import TransformersMenu from "./components/Transformers";
import ViewMenu from "./components/View";
import WritersMenu from "./components/Writers";

export default function MenubarComponent() {
  return (
    <Menubar
      style={{
        color: "#dbdbdb",
        border: "none",
        borderBottom: "0.5px solid #dbdbdb",
        borderRadius: 0,
        position: "relative",
      }}>
      <FileMenu />
      <EditMenu />
      <ViewMenu />
      <ReadersMenu />
      <TransformersMenu />
      <WritersMenu />
      <RunMenu />
      <ToolsMenu />
      <HelpMenu />
      <div
        style={{
          position: "absolute",
          right: 0,
          marginRight: "10px",
          display: "flex",
          gap: "10px",
          justifyContent: "space-between",
          alignItems: "center",
        }}>
        <p>yokohamaRiver.fmw - CDED -&#62; NONE - Flow 2024</p>
        <Button size="icon" variant="ghost" style={{ height: "30px", width: "30px" }}>
          <img src={Github} alt="Github" width="25px" height="25px" />
        </Button>
      </div>
    </Menubar>
  );
}
