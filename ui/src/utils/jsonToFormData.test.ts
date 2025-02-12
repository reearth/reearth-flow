import { describe, it, expect } from "vitest";

import { jsonToFormData } from "./jsonToFormData";

describe("jsonToFormData", () => {
  it("should create FormData with a JSON file", () => {
    return new Promise<void>((resolve, reject) => {
      const json = { key: "value" };
      const formData = jsonToFormData(json);
      const file = formData.get("file") as File;
      expect(file).toBeInstanceOf(File);
      expect(file.name).toBe("untitled.json");
      expect(file.type).toBe("application/json");

      const reader = new FileReader();
      reader.onerror = () => reject(reader.error);
      reader.onload = () => {
        expect(reader.result).toBe(JSON.stringify(json));
        resolve();
      };
      reader.readAsText(file);
    });
  });

  it("should use the provided fileName", () => {
    return new Promise<void>((resolve, reject) => {
      const json = { key: "value" };
      const fileName = "test";
      const formData = jsonToFormData(json, fileName);

      const file = formData.get("file") as File;
      expect(file).toBeInstanceOf(File);
      expect(file.name).toBe(`${fileName}.json`);
      expect(file.type).toBe("application/json");

      const reader = new FileReader();
      reader.onerror = () => reject(reader.error);
      reader.onload = () => {
        expect(reader.result).toBe(JSON.stringify(json));
        resolve();
      };
      reader.readAsText(file);
    });
  });

  it("should handle an empty JSON object", () => {
    return new Promise<void>((resolve, reject) => {
      const json = {};
      const formData = jsonToFormData(json);

      const file = formData.get("file") as File;
      expect(file).toBeInstanceOf(File);
      expect(file.name).toBe("untitled.json");
      expect(file.type).toBe("application/json");

      const reader = new FileReader();
      reader.onerror = () => reject(reader.error);
      reader.onload = () => {
        expect(reader.result).toBe(JSON.stringify(json));
        resolve();
      };
      reader.readAsText(file);
    });
  });
});
