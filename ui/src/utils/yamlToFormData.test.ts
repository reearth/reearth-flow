import { describe, it, expect } from "vitest";

import { yamlToFormData } from "./yamlToFormData";

describe("yamlToFormData", () => {
  it("should create FormData with a yaml file", () => {
    return new Promise<void>((resolve, reject) => {
      const yaml = "key: value";
      const formData = yamlToFormData(yaml);
      const file = formData.get("file") as File;
      expect(file).toBeInstanceOf(File);
      expect(file.name).toBe("untitled.yml");
      expect(file.type).toBe("application/x-yaml");
      const reader = new FileReader();
      reader.onerror = () => reject(reader.error);
      reader.onload = () => {
        expect(reader.result).toBe(yaml);
        resolve();
      };
      reader.readAsText(file);
    });
  });

  it("should use the provided fileName", () => {
    return new Promise<void>((resolve, reject) => {
      const yaml = "key: value";
      const fileName = "test";
      const formData = yamlToFormData(yaml, fileName);

      const file = formData.get("file") as File;
      expect(file).toBeInstanceOf(File);
      expect(file.name).toBe(`${fileName}.yml`);
      expect(file.type).toBe("application/x-yaml");

      const reader = new FileReader();
      reader.onerror = () => reject(reader.error);
      reader.onload = () => {
        expect(reader.result).toBe(yaml);
        resolve();
      };
      reader.readAsText(file);
    });
  });

  it("should handle empty yaml string", () => {
    return new Promise<void>((resolve, reject) => {
      const yaml = "";
      const formData = yamlToFormData(yaml);

      const file = formData.get("file") as File;
      expect(file).toBeInstanceOf(File);
      expect(file.name).toBe("untitled.yml");
      expect(file.type).toBe("application/x-yaml");

      const reader = new FileReader();
      reader.onerror = () => reject(reader.error);
      reader.onload = () => {
        expect(reader.result).toBe(yaml);
        resolve();
      };
      reader.readAsText(file);
    });
  });
});
