import { RenderFallback } from "@flow/components";
import { MapLibre } from "@flow/components/visualizations/MapLibre";
import { useT } from "@flow/lib/i18n";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

type Props = {
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
};

const TwoDViewer: React.FC<Props> = ({ fileContent, fileType }) => {
  const t = useT();
  return (
    <RenderFallback
      message={t("2D Viewer Could Not Be Loaded. Check if the data is valid.")}
      textSize="sm">
      <MapLibre fileContent={fileContent} fileType={fileType} />
    </RenderFallback>
  );
};

export { TwoDViewer };
