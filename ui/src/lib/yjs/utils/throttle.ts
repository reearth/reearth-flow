export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number,
): T {
  let inThrottle: boolean;
  let lastResult: ReturnType<T>;

  return ((...args: any[]) => {
    if (!inThrottle) {
      lastResult = func(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
    return lastResult;
  }) as T;
}
