const VARIABLE_REFERENCE =
  /^\s*variables\s*(?:\[\s*r?"[^"]*"\s*\]|\.get\(\s*r?"[^"]*"\s*\))\s*$/;

export const inlineCodeType = (
  text: string,
  flowExprAllowed: boolean,
): "flowExpr" | "string" =>
  flowExprAllowed && VARIABLE_REFERENCE.test(text) ? "flowExpr" : "string";
