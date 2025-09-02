import { useCallback, useMemo } from "react";

import { useT } from "@flow/lib/i18n";
import { VARIABLE_TYPE_OPTIONS, VarType } from "@flow/types";

export default () => {
  const t = useT();

  const getUserFacingName = useCallback(
    (type: VarType): string => {
      switch (type) {
        case "attribute_name":
          return t("Attribute Name");
        case "choice":
          return t("Choice");
        case "color":
          return t("Color");
        case "coordinate_system":
          return t("Coordinate System");
        case "database_connection":
          return t("Database Connection");
        case "datetime":
          return t("Date and Time");
        case "file_folder":
          return t("File or Folder");
        case "geometry":
          return t("Geometry");
        case "message":
          return t("Message");
        case "number":
          return t("Number");
        case "password":
          return t("Password");
        case "reprojection_file":
          return t("Reprojection File");
        case "text":
          return t("Text");
        case "web_connection":
          return t("Web Connection");
        case "yes_no":
          return t("Yes/No");
        default:
          return t("Unsupported");
      }
    },
    [t],
  );

  const userFacingName = useMemo(() => {
    return VARIABLE_TYPE_OPTIONS.reduce((acc, type) => {
      acc[type] = getUserFacingName(type);
      return acc;
    }, {} as Record<VarType, string>);
  }, [getUserFacingName]);

  return {
    getUserFacingName,
    userFacingName,
  }
}