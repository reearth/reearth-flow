class ProjectCorruptionError extends Error {
  constructor(message = "Project data is corrupted") {
    super(message);
    this.name = "ProjectCorruptionError";
  }
}

export { ProjectCorruptionError };
