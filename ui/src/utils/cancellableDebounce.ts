type DebouncedFunction<T extends (...args: unknown[]) => void> = {
  (...args: Parameters<T>): void;
  cancel: () => void;
};

export function cancellableDebounce<T extends (...args: unknown[]) => void>(
  func: T,
  wait: number
): DebouncedFunction<T> {
  let timeout: ReturnType<typeof setTimeout>;

  const debounced = function (...args: Parameters<T>) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };

    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };

  debounced.cancel = () => {
    clearTimeout(timeout);
  };

  return debounced as DebouncedFunction<T>;
}
