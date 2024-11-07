import {
  Button,
  SupportedVisualizations,
  ThreeDMap,
  TwoDMap,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

export type MapMode = Extract<SupportedVisualizations, "3d-map" | "2d-map">;

type Props = {
  mapMode: MapMode;
  setMapMode?: (mode: MapMode) => void;
};

const Map: React.FC<Props> = ({ mapMode, setMapMode }) => {
  const t = useT();
  const mapModes: { key: MapMode; value: string }[] = [
    { key: "2d-map", value: t("2D") },
    { key: "3d-map", value: t("3D") },
  ];
  return (
    <div className="relative w-full">
      <div className="absolute left-2 top-2 z-10 flex flex-col flex-wrap rounded-md border bg-background transition-all">
        {mapModes.map((b) => (
          <Button
            className={`cursor-pointer rounded-none transition-all ${mapMode === b.key ? "bg-accent text-accent-foreground" : ""}`}
            variant="ghost"
            size="icon"
            key={b.key}
            onClick={() => mapMode !== b.key && setMapMode?.(b.key)}>
            {b.value}
          </Button>
        ))}
      </div>
      {mapMode === "3d-map" ? <ThreeDMap /> : <TwoDMap />}
    </div>
  );
};

export { Map };
