import type { licenseContent } from "./content";

export type LicenseValue = keyof typeof licenseContent;

export type LicenseOption = {
  value: LicenseValue;
  label: string;
  // Canonical text of the license, linked from the dropdown.
  url: string;
};

export type LicenseGroupKey = "open" | "shareAlike" | "limited";

// The licenses Re:Earth Visualizer offers, under the same headings and in the
// same order, from most to least permissive. Labels are license names and are
// not translated; group headings and summaries are, where they are rendered.
export const licenseGroups: {
  key: LicenseGroupKey;
  licenses: LicenseOption[];
}[] = [
  {
    key: "open",
    licenses: [
      {
        value: "cc-by-4.0",
        label: "Creative Commons Attribution 4.0 International",
        url: "https://creativecommons.org/licenses/by/4.0/",
      },
      {
        value: "mit",
        label: "MIT License",
        url: "https://opensource.org/license/mit",
      },
      {
        value: "apache-2.0",
        label: "Apache License 2.0",
        url: "https://www.apache.org/licenses/LICENSE-2.0",
      },
      {
        value: "o-uda-1.0",
        label: "Open Use of Data Agreement v1.0",
        url: "https://cdla.dev/open-use-of-data-agreement-v1-0/",
      },
      {
        value: "odc-by-1.0",
        label: "Open Data Commons Attribution License v1.0",
        url: "https://opendatacommons.org/licenses/by/1-0/",
      },
    ],
  },
  {
    key: "shareAlike",
    licenses: [
      {
        value: "cc-by-sa-4.0",
        label: "Creative Commons Attribution-ShareAlike 4.0 International",
        url: "https://creativecommons.org/licenses/by-sa/4.0/",
      },
      {
        value: "gpl-3.0",
        label: "GNU General Public License v3.0",
        url: "https://www.gnu.org/licenses/gpl-3.0.html",
      },
      {
        value: "lgpl-3.0",
        label: "GNU Lesser General Public License v3.0",
        url: "https://www.gnu.org/licenses/lgpl-3.0.html",
      },
      {
        value: "eupl-1.2",
        label: "European Union Public License v1.2",
        url: "https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12",
      },
      {
        value: "odbl-1.0",
        label: "Open Data Commons Open Database License v1.0",
        url: "https://opendatacommons.org/licenses/odbl/1-0/",
      },
    ],
  },
  {
    key: "limited",
    licenses: [
      {
        value: "cc-by-nd-4.0",
        label: "Creative Commons Attribution-NoDerivatives 4.0 International",
        url: "https://creativecommons.org/licenses/by-nd/4.0/",
      },
    ],
  },
];

export const licenseOptions: LicenseOption[] = licenseGroups.flatMap(
  (group) => group.licenses,
);

// Picking "no license" on creation keeps all rights reserved, which is saved
// as an empty license.
export const NO_LICENSE_VALUE = "no-license";

export type LicenseChoice = LicenseValue | typeof NO_LICENSE_VALUE;

export const isLicenseChoice = (value: unknown): value is LicenseChoice =>
  value === NO_LICENSE_VALUE ||
  licenseOptions.some((option) => option.value === value);

export const loadLicenseContent = async (
  choice: LicenseChoice,
): Promise<string> => {
  if (choice === NO_LICENSE_VALUE) return "";
  const { licenseContent } = await import("./content");
  return licenseContent[choice];
};
