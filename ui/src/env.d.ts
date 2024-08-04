import "@tanstack/react-router";

// Register the router instance for type safety
declare module "@tanstack/react-router" {
  type Register = {
    router: typeof router;
  };
}
