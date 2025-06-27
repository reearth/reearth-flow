export async function enableMocking({ disabled }: { disabled?: boolean } = {}) {
  if (disabled || import.meta.env.MODE !== "development") {
    return;
  }

  const { worker } = await import("./browser");

  const workerInstance = await worker.start({
    onUnhandledRequest: "bypass",
  });

  // Import test functions in development
  if (import.meta.env.DEV) {
    import("./test");
  }

  return workerInstance;
}
