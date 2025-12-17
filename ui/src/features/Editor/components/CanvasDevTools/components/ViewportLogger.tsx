import { useStore } from "@xyflow/react";

export default function ViewportLogger() {
  const viewport = useStore(
    (s) =>
      `x: ${s.transform[0].toFixed(2)}, y: ${s.transform[1].toFixed(
        2,
      )}, zoom: ${s.transform[2].toFixed(2)}`,
  );

  return (
    <div className="absolute top-28 right-4 flex flex-col rounded-lg border border-slate-700 bg-slate-800/95 p-2 shadow-lg">
      <span className="font-bold">Viewport Logger</span>
      {viewport}
    </div>
  );
}
