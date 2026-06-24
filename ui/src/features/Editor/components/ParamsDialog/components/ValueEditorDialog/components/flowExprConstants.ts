export type AutocompleteSuggestion = {
  label: string;
  insertText: string;
  type: "keyword" | "function" | "namespace" | "variable" | "operator";
  description?: string;
  detail?: string;
};

export const FLOWEXPR_KEYWORDS = [
  "if",
  "else",
  "while",
  "for",
  "in",
  "not",
  "and",
  "or",
  "true",
  "false",
  "null",
];

export const FLOWEXPR_BUILTIN_FUNCTIONS = [
  "str",
  "int",
  "float",
  "bool",
  "list",
  "map",
  "Url",
  "attributes",
  "env",
  "print",
  "len",
  "type",
  "math",
];

export const FLOWEXPR_OPERATORS = [
  "==",
  "!=",
  "<=",
  ">=",
  "<",
  ">",
  "+",
  "-",
  "*",
  "**",
  "//",
  "/",
  "%",
  "=",
  "+=",
  "-=",
  "*=",
  "**=",
  "//=",
  "/=",
  "%=",
];

export const getFlowExprAutocompleteSuggestions = (
  t: (key: string) => string,
): AutocompleteSuggestion[] => [
  // Keywords
  {
    label: "if",
    insertText: "if {{cursor}} {\n  \n} else {\n  \n}",
    type: "keyword",
    description: t("if-else expression"),
  },
  {
    label: "else",
    insertText: "else { {{cursor}} }",
    type: "keyword",
    description: t("else branch"),
  },
  {
    label: "while",
    insertText: "while {{cursor}} {\n  \n}",
    type: "keyword",
    description: t("while loop"),
  },
  {
    label: "for",
    insertText: "for item in {{cursor}} {\n  \n}",
    type: "keyword",
    description: t("for-in loop"),
  },
  {
    label: "in",
    insertText: "in ",
    type: "keyword",
    description: t("membership test / loop iteration"),
  },
  {
    label: "not",
    insertText: "not ",
    type: "keyword",
    description: t("logical NOT"),
  },
  {
    label: "not in",
    insertText: "not in ",
    type: "keyword",
    description: t("Membership negation (x not in arr)"),
  },
  {
    label: "and",
    insertText: "and ",
    type: "keyword",
    description: t("logical AND"),
  },
  {
    label: "or",
    insertText: "or ",
    type: "keyword",
    description: t("logical OR"),
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

  // Built-in global functions
  {
    label: "attributes[]",
    insertText: 'attributes["{{cursor}}"]',
    type: "function",
    description: t("Feature attribute map — throws if key is missing"),
    detail: 'attributes["key"] → any',
  },
  {
    label: "attributes.get",
    insertText: 'attributes.get("{{cursor}}")',
    type: "function",
    description: t(
      "Feature attribute map — returns fallback/null if key is missing",
    ),
    detail: 'attributes.get("key"[, fallback]) → any',
  },
  {
    label: "env",
    insertText: 'env["{{cursor}}"]',
    type: "function",
    description: t("Read an environment variable"),
    detail: 'env["VAR_NAME"] → string',
  },
  {
    label: "Url",
    insertText: "Url({{cursor}})",
    type: "function",
    description: t("Construct a URL/path value"),
    detail: "Url(path: string) → Url",
  },
  {
    label: "str",
    insertText: "str({{cursor}})",
    type: "function",
    description: t("Convert to string"),
    detail: "str(value) → string",
  },
  {
    label: "int",
    insertText: "int({{cursor}})",
    type: "function",
    description: t("Convert to integer"),
    detail: "int(value) → integer",
  },
  {
    label: "float",
    insertText: "float({{cursor}})",
    type: "function",
    description: t("Convert to float"),
    detail: "float(value) → float",
  },
  {
    label: "bool",
    insertText: "bool({{cursor}})",
    type: "function",
    description: t("Convert to boolean"),
    detail: "bool(value) → bool",
  },
  {
    label: "list",
    insertText: "list({{cursor}})",
    type: "function",
    description: t("Convert to array"),
    detail: "list(value) → array",
  },
  {
    label: "map",
    insertText: "map({{cursor}})",
    type: "function",
    description: t("Convert to map"),
    detail: "map(value) → map",
  },
  {
    label: "print",
    insertText: "print({{cursor}})",
    type: "function",
    description: t("Debug print (returns first argument)"),
    detail: "print(...) → any",
  },
  {
    label: "len",
    insertText: "len({{cursor}})",
    type: "function",
    description: t("Length of a string, array, or map"),
    detail: "len(value) → int",
  },
  {
    label: "type",
    insertText: "type({{cursor}})",
    type: "function",
    description: t("Return the type name of a value"),
    detail: "type(value) → string",
  },

  // String methods
  {
    label: "trim",
    insertText: "trim()",
    type: "function",
    description: t("Trim leading/trailing whitespace"),
    detail: "s.trim() → string",
  },
  {
    label: "split",
    insertText: "split({{cursor}})",
    type: "function",
    description: t("Split string by separator"),
    detail: "s.split(sep: string) → array",
  },
  {
    label: "starts_with",
    insertText: "starts_with({{cursor}})",
    type: "function",
    description: t("Check if string starts with prefix"),
    detail: "s.starts_with(prefix: string) → bool",
  },
  {
    label: "ends_with",
    insertText: "ends_with({{cursor}})",
    type: "function",
    description: t("Check if string ends with suffix"),
    detail: "s.ends_with(suffix: string) → bool",
  },
  {
    label: "replace",
    insertText: "replace({{cursor}}, )",
    type: "function",
    description: t("Replace all occurrences in string"),
    detail: "s.replace(from: string, to: string) → string",
  },
  {
    label: "remove_prefix",
    insertText: "remove_prefix({{cursor}})",
    type: "function",
    description: t("Strip prefix from string (no-op if absent)"),
    detail: "s.remove_prefix(prefix: string) → string",
  },
  {
    label: "remove_suffix",
    insertText: "remove_suffix({{cursor}})",
    type: "function",
    description: t("Strip suffix from string (no-op if absent)"),
    detail: "s.remove_suffix(suffix: string) → string",
  },

  // Map methods
  {
    label: "keys",
    insertText: "keys()",
    type: "function",
    description: t("Map keys as array"),
    detail: "m.keys() → array",
  },
  {
    label: "values",
    insertText: "values()",
    type: "function",
    description: t("Map values as array"),
    detail: "m.values() → array",
  },
  {
    label: "items",
    insertText: "items()",
    type: "function",
    description: t("Map entries as [[key, value], …]"),
    detail: "m.items() → array",
  },
  {
    label: "get",
    insertText: "get({{cursor}})",
    type: "function",
    description: t("Map value by key (null if absent)"),
    detail: "m.get(key: string) → any",
  },

  // Url properties (accessed without parentheses)
  {
    label: "parent",
    insertText: "parent",
    type: "variable",
    description: t("Parent directory of URL path"),
    detail: "url.parent → Url",
  },
  {
    label: "name",
    insertText: "name",
    type: "variable",
    description: t("Final path component of URL"),
    detail: "url.name → string",
  },
  {
    label: "suffix",
    insertText: "suffix",
    type: "variable",
    description: t("File extension of URL path"),
    detail: "url.suffix → string",
  },

  // Math module (access as math.function)
  {
    label: "sin",
    insertText: "sin({{cursor}})",
    type: "function",
    description: t("Sine — math module"),
    detail: "math.sin(x: float) → float",
  },
  {
    label: "cos",
    insertText: "cos({{cursor}})",
    type: "function",
    description: t("Cosine — math module"),
    detail: "math.cos(x: float) → float",
  },
  {
    label: "floor",
    insertText: "floor({{cursor}})",
    type: "function",
    description: t("Floor — math module"),
    detail: "math.floor(x: float) → float",
  },
  {
    label: "round",
    insertText: "round({{cursor}})",
    type: "function",
    description: t("Round away from zero — math module"),
    detail: "math.round(x: float) → float",
  },
  {
    label: "log",
    insertText: "log({{cursor}})",
    type: "function",
    description: t("Natural log or log with base — math module"),
    detail: "math.log(x: float[, base: float]) → float",
  },
  {
    label: "log2",
    insertText: "log2({{cursor}})",
    type: "function",
    description: t("Base-2 logarithm — math module"),
    detail: "math.log2(x: float) → float",
  },
  {
    label: "log10",
    insertText: "log10({{cursor}})",
    type: "function",
    description: t("Base-10 logarithm — math module"),
    detail: "math.log10(x: float) → float",
  },
  {
    label: "radians",
    insertText: "radians({{cursor}})",
    type: "function",
    description: t("Degrees to radians — math module"),
    detail: "math.radians(deg: float) → float",
  },
  {
    label: "pi",
    insertText: "pi",
    type: "variable",
    description: t("π constant — math module"),
    detail: "math.pi → float (≈ 3.14159)",
  },

  // Operators
  {
    label: "==",
    insertText: "== ",
    type: "operator",
    description: t("Equal"),
  },
  {
    label: "!=",
    insertText: "!= ",
    type: "operator",
    description: t("Not equal"),
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
];
