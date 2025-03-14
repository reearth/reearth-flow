import { useEffect, useRef, useState } from "react";

import {
  fetchAndReadData,
  SupportedDataTypes,
} from "@flow/utils/fetchAndReadGeoData";

type Props = {
  dataUrl?: string;
};

export default ({ dataUrl = "" }: Props) => {
  const prevDataUrl = useRef<string | null>(null);
  const [fileContent, setFileContent] = useState<string | null>(null);
  const [fileType, setFileType] = useState<SupportedDataTypes | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!fileContent || dataUrl !== prevDataUrl.current) {
      setIsLoading(true);
      setError(null);
      (async () => {
        const { fileContent, type, error } = await fetchAndReadData(dataUrl);
        setFileContent(fileContent);
        setFileType(type);
        setError(error);
        setIsLoading(false);
      })();
      prevDataUrl.current = dataUrl;
    }
  }, [dataUrl, fileContent]);

  return {
    fileContent,
    fileType,
    isLoading,
    error,
  };
};
