import {
  CaretUp,
  CornersIn,
  CornersOut,
  Globe,
  GridNine,
  Minus,
} from "@phosphor-icons/react";
import { memo } from "react";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import { DataTable, GeoMap } from "./components";
import useHooks from "./hooks";

const DebugPreview: React.FC = () => {
  const t = useT();
  const {
    outputURLs,
    expanded,
    minimized,
    fileContent,
    fileType,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleSelectedDataChange,
  } = useHooks();

  return outputURLs ? (
    <Tabs
      className={`pointer-events-auto w-[45vw] min-w-[700px] rounded border bg-secondary transition-all ${minimized ? "h-[36px]" : expanded ? "h-[85vh] w-[90vw]" : "h-[500px]"}`}
      defaultValue="data-viewer">
      <div className="relative flex items-center p-1">
        <div className="absolute left-1 top-1">
          <Select
            defaultValue={outputURLs[0]}
            onValueChange={handleSelectedDataChange}>
            <SelectTrigger className="h-[26px] max-w-[200px] border-none">
              <SelectValue placeholder={t("Select Data to Preview")} />
            </SelectTrigger>
            <SelectContent>
              {outputURLs.map((url) => (
                <SelectItem key={url} value={url}>
                  {url.split("/").pop()}
                  {t(" (Output data)")}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="flex w-full items-center justify-center gap-1">
          <TabsList className="gap-2">
            <TabsTrigger
              className="gap-1"
              value="data-viewer"
              onClick={handleTabChange}>
              <GridNine />
              {t("Data Viewer")}
            </TabsTrigger>
            <TabsTrigger
              className="gap-1"
              value="3d-viewer"
              onClick={handleTabChange}>
              <Globe />
              {t("3D Viewer")}
            </TabsTrigger>
          </TabsList>
        </div>
        <div className="absolute right-1 top-1 flex items-center gap-2 ">
          <div
            className="cursor-pointer rounded p-1 hover:bg-primary"
            onClick={handleMinimize}>
            {minimized ? <CaretUp weight="light" /> : <Minus weight="light" />}
          </div>
          {!minimized && (
            <div
              className="cursor-pointer rounded p-1 hover:bg-primary"
              onClick={handleExpand}>
              {expanded ? (
                <CornersIn weight="light" />
              ) : (
                <CornersOut weight="light" />
              )}
            </div>
          )}
        </div>
      </div>
      <TabsContent
        className="h-[calc(100%-35px)] overflow-scroll"
        value="data-viewer">
        <DataTable fileContent={fileContent} fileType={fileType} />
      </TabsContent>
      <TabsContent className="h-[calc(100%-35px)] px-1 pb-2" value="3d-viewer">
        <GeoMap fileContent={fileContent} fileType={fileType} />
      </TabsContent>
    </Tabs>
  ) : null;
};

export default memo(DebugPreview);
