import { describe, it, expect } from "vitest";

import { yamlToFormData } from "./yamlToFormData";

describe("yamlToFormData", () => {
  it("should create FormData with a yaml file", () => {
    const yaml = "key: value";
    const formData = yamlToFormData(yaml);

    const file = formData.get("file") as File;
    expect(file).toBeInstanceOf(File);
    expect(file.name).toBe("untitled.yaml");
    expect(file.type).toBe("application/x-yaml");

    const reader = new FileReader();
    reader.onload = () => {
      expect(reader.result).toBe(yaml);
    };
    reader.readAsText(file);
  });

  it("should use the provided fileName", () => {
    const yaml = "key: value";
    const fileName = "test";
    const formData = yamlToFormData(yaml, fileName);

    const file = formData.get("file") as File;
    expect(file).toBeInstanceOf(File);
    expect(file.name).toBe(`${fileName}.yaml`);
    expect(file.type).toBe("application/x-yaml");

    const reader = new FileReader();
    reader.onload = () => {
      expect(reader.result).toBe(yaml);
    };
    reader.readAsText(file);
  });

  it("should handle empty yaml string", () => {
    const yaml = "";
    const formData = yamlToFormData(yaml);

    const file = formData.get("file") as File;
    expect(file).toBeInstanceOf(File);
    expect(file.name).toBe("untitled.yaml");
    expect(file.type).toBe("application/x-yaml");

    const reader = new FileReader();
    reader.onload = () => {
      expect(reader.result).toBe(yaml);
    };
    reader.readAsText(file);
  });
});
