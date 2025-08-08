import { debounce } from "lodash-es";
import { useEffect, useRef, useState } from "react";

type Props = {
  initialSearchTerm: string;
  delay: number;
  onDebounced: (term: string) => void;
};

export default ({
  initialSearchTerm = "",
  delay = 300,
  onDebounced,
}: Props) => {
  const lastSearchTerm = useRef(initialSearchTerm);
  const [searchTerm, setSearchTerm] = useState(initialSearchTerm);
  const [isDebouncing, setIsDebouncing] = useState<boolean>(false);

  const onDebouncedRef = useRef(onDebounced);
  useEffect(() => {
    onDebouncedRef.current = onDebounced;
  }, [onDebounced]);

  const debounced = useRef(
    debounce((term: string) => {
      onDebouncedRef.current(term);
      setIsDebouncing(false);
    }, delay),
  );

  useEffect(() => {
    if (searchTerm === lastSearchTerm.current) return;
    lastSearchTerm.current = searchTerm;
    setIsDebouncing(true);
    debounced.current(searchTerm);
  }, [searchTerm, initialSearchTerm]);

  return {
    searchTerm,
    isDebouncing,
    setSearchTerm,
  };
};
