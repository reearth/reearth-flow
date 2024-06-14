import Canvas from "@flow/features/Editor";
import { useCurrentProject } from "@flow/stores";

function Editor() {
  const [currentProject] = useCurrentProject();

  return (
    <div className="flex flex-col bg-zinc-900 text-zinc-300 h-screen">
      <Canvas workflow={currentProject?.workflow} />
    </div>
  );
}

export { Editor };
