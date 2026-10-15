import { describe, expect, test } from "vitest";

import { licenseContent } from "./content";
import {
  NO_LICENSE_VALUE,
  isLicenseChoice,
  licenseOptions,
  loadLicenseContent,
} from "./licenses";

describe("license templates", () => {
  test("every option has text, and every text an option", () => {
    expect(licenseOptions.map((o) => o.value).sort()).toEqual(
      Object.keys(licenseContent).sort(),
    );
  });

  test.each(licenseOptions.map((o) => o.value))(
    "loads the full text of %s",
    async (value) => {
      const text = await loadLicenseContent(value);
      expect(text.length).toBeGreaterThan(500);
    },
  );

  test("no license is saved as an empty license", async () => {
    expect(await loadLicenseContent(NO_LICENSE_VALUE)).toBe("");
  });

  test("recognises only known choices", () => {
    expect(isLicenseChoice("mit")).toBe(true);
    expect(isLicenseChoice(NO_LICENSE_VALUE)).toBe(true);
    expect(isLicenseChoice("cc-by-nc-4.0")).toBe(false);
    expect(isLicenseChoice(undefined)).toBe(false);
  });
});
