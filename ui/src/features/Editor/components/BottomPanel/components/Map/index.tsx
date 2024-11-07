import { Button, ThreeDMap, TwoDMap } from "@flow/components";

export type MapMode = "2d" | "3d";

const mapModes: MapMode[] = ["2d", "3d"];

type Props = {
  mapMode: MapMode;
  setMapMode?: (mode: MapMode) => void;
};

const Map: React.FC<Props> = ({ mapMode, setMapMode }) => {
  return (
    <div className="flex w-1/2">
      <div className="relative w-full">
        <div className="absolute left-2 top-2 z-10 flex flex-col flex-wrap rounded-md border bg-background transition-all">
          {mapModes.map((b) => (
            <Button
              className={`cursor-pointer rounded-none transition-all ${mapMode === b ? "bg-accent text-accent-foreground" : ""}`}
              variant="ghost"
              size="icon"
              key={b}
              onClick={() => mapMode !== b && setMapMode?.(b)}>
              {b}
            </Button>
          ))}
        </div>
        {mapMode === "2d" ? <TwoDMap /> : <ThreeDMap />}
      </div>
    </div>
  );
};

export { Map };
