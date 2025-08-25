import { useDebugRunUrlQuery } from "./useDebugRunUrlQuery";

type Props = {
  dataUrl?: string;
};

export default ({ dataUrl = "" }: Props) => {
  const { data, isLoading } = useDebugRunUrlQuery(dataUrl);

  return {
    fileContent: data?.fileContent,
    fileType: data?.type,
    isLoading,
    error: data?.error,
  };
};
