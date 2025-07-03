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
  const [searchTerm, setSearchTerm] = useState(initialSearchTerm);

  const onDebouncedRef = useRef(onDebounced);
  useEffect(() => {
    onDebouncedRef.current = onDebounced;
  }, [onDebounced]);

  const debounced = useRef(
    debounce((term: string) => {
      onDebouncedRef.current(term);
    }, delay),
  );

  useEffect(() => {
    debounced.current(searchTerm);
  }, [searchTerm]);

  return {
    searchTerm,
    setSearchTerm,
  };
};
