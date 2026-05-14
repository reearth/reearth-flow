// Flow Expression Language (FEL) constants

export const RHAI_KEYWORDS = [
  "if",
  "else",
  "let",
  "true",
  "false",
  "null",
  "in",
];

export const RHAI_FUNCTIONS = [
  "value",
  "env",
  "str",
  "int",
  "float",
  "bool",
  "list",
  "map",
  "Url",
];

export const FEL_METHODS = [
  "trim",
  "len",
  "parent",
  "extension",
  "name",
  "stem",
];

export const RHAI_OPERATORS = [
  "==",
  "!=",
  "<=",
  ">=",
  "<",
  ">",
  "&&",
  "||",
  "!",
  "+",
  "-",
  "*",
  "/",
  "=",
  "+=",
  "-=",
  "*=",
  "/=",
];

// No namespaces in FEL
export const RHAI_NAMESPACES: string[] = [];

export type AutocompleteSuggestion = {
  label: string;
  insertText: string;
  type: "keyword" | "function" | "namespace" | "variable" | "operator";
  description?: string;
  detail?: string;
};

export const getRhaiAutocompleteSuggestions = (
  t: (key: string) => string,
): AutocompleteSuggestion[] => [
  // Context access functions
  {
    label: "value",
    insertText: 'value("{{cursor}}")',
    type: "function",
    description: t("Read a feature attribute by name"),
    detail: "value(name: string)",
  },
  {
    label: "env",
    insertText: 'env("{{cursor}}")',
    type: "function",
    description: t("Read a workflow variable by name"),
    detail: "env(name: string)",
  },

  // Type conversion functions
  {
    label: "str",
    insertText: "str({{cursor}})",
    type: "function",
    description: t("Convert to string"),
    detail: "str(x: any) → string",
  },
  {
    label: "int",
    insertText: "int({{cursor}})",
    type: "function",
    description: t("Convert to integer (truncates floats)"),
    detail: "int(x: any) → int",
  },
  {
    label: "float",
    insertText: "float({{cursor}})",
    type: "function",
    description: t("Convert to float"),
    detail: "float(x: any) → float",
  },
  {
    label: "bool",
    insertText: "bool({{cursor}})",
    type: "function",
    description: t("Evaluate truthiness as true/false"),
    detail: "bool(x: any) → bool",
  },
  {
    label: "list",
    insertText: "list({{cursor}})",
    type: "function",
    description: t("String → chars; map → keys; array → identity"),
    detail: "list(x: any) → array",
  },
  {
    label: "map",
    insertText: 'map([["{{cursor}}", ]])',
    type: "function",
    description: t("Build map from [[key, val], …] pairs"),
    detail: "map(pairs: array) → map",
  },
  {
    label: "Url",
    insertText: 'Url("{{cursor}}")',
    type: "function",
    description: t("Construct a Url object from a string"),
    detail: "Url(s: string) → Url",
  },

  // Keywords
  {
    label: "if",
    insertText: "if {{cursor}} { } else { }",
    type: "keyword",
    description: t("If-else conditional block"),
  },
  {
    label: "else",
    insertText: "else { }",
    type: "keyword",
    description: t("Else branch"),
  },
  {
    label: "let",
    insertText: "let {{cursor}} = ; ",
    type: "keyword",
    description: t("Bind a name to a value (available after the ;)"),
  },
  {
    label: "true",
    insertText: "true",
    type: "keyword",
    description: t("Boolean true"),
  },
  {
    label: "false",
    insertText: "false",
    type: "keyword",
    description: t("Boolean false"),
  },
  {
    label: "null",
    insertText: "null",
    type: "keyword",
    description: t("Null value"),
  },

  // Operators
  {
    label: "in",
    insertText: "in ",
    type: "operator",
    description: t("Membership — array, string, or map key"),
  },
  {
    label: "==",
    insertText: "== ",
    type: "operator",
    description: t("Equal to"),
  },
  {
    label: "!=",
    insertText: "!= ",
    type: "operator",
    description: t("Not equal to"),
  },
  {
    label: "<=",
    insertText: "<= ",
    type: "operator",
    description: t("Less than or equal"),
  },
  {
    label: ">=",
    insertText: ">= ",
    type: "operator",
    description: t("Greater than or equal"),
  },
  {
    label: "&&",
    insertText: "&& ",
    type: "operator",
    description: t("Logical AND"),
  },
  {
    label: "||",
    insertText: "|| ",
    type: "operator",
    description: t("Logical OR"),
  },

  // Methods (triggered after a dot)
  {
    label: ".trim",
    insertText: ".trim()",
    type: "function",
    description: t("Remove leading/trailing whitespace"),
    detail: "string.trim() → string",
  },
  {
    label: ".len",
    insertText: ".len()",
    type: "function",
    description: t("Length of string or array"),
    detail: ".len() → int",
  },
  {
    label: ".parent",
    insertText: ".parent()",
    type: "function",
    description: t("Parent directory as a Url"),
    detail: "Url.parent() → Url",
  },
  {
    label: ".extension",
    insertText: ".extension()",
    type: "function",
    description: t("File extension without dot"),
    detail: "Url.extension() → string",
  },
  {
    label: ".name",
    insertText: ".name()",
    type: "function",
    description: t("Filename including extension"),
    detail: "Url.name() → string",
  },
  {
    label: ".stem",
    insertText: ".stem()",
    type: "function",
    description: t("Filename without extension"),
    detail: "Url.stem() → string",
  },
];
