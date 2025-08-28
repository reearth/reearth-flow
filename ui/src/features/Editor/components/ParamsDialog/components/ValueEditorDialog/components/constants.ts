export const RHAI_KEYWORDS = [
  "if",
  "else",
  "while",
  "for",
  "loop",
  "break",
  "continue",
  "return",
  "let",
  "const",
  "fn",
  "private",
  "import",
  "export",
  "as",
  "true",
  "false",
  "null",
];

export const RHAI_FUNCTIONS = [
  // Global environment functions (custom Re:Earth Flow functions)
  "env",
  "get",
  // File namespace functions - essential for path manipulation
  "extract_filename",
  "extract_filename_without_ext",
  "join_path",
  // String namespace functions - useful for regex operations
  "extract_single_by_regex",
  // JSON namespace functions - valuable for data processing
  "find_value_by_json_path",
  "exists_value_by_json_path",
  // Key DateTime namespace functions - common date operations
  "extract_year",
  "extract_month",
  "extract_day",
  "add_year",
  "add_month",
  "add_day",
  // Standard Rhai math functions
  "round",
  "floor",
  "ceil",
  "abs",
  "sqrt",
  "min",
  "max",
  "pow",
  "sin",
  "cos",
  "tan",
  // Standard Rhai string functions
  "len",
  "is_empty",
  "contains",
  "starts_with",
  "ends_with",
  "to_string",
  "to_upper",
  "to_lower",
  // Standard Rhai array functions
  "push",
  "pop",
  "shift",
  "unshift",
  "insert",
  "remove",
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
  "%",
  "**",
  "=",
  "+=",
  "-=",
  "*=",
  "/=",
  "%=",
  "**=",
  "?",
  ":",
];

export const RHAI_NAMESPACES = ["file", "str", "json", "datetime"];

export type AutocompleteSuggestion = {
  label: string;
  insertText: string;
  type: "keyword" | "function" | "namespace" | "variable" | "operator";
  description?: string;
  detail?: string;
};

export const RHAI_AUTOCOMPLETE_SUGGESTIONS: AutocompleteSuggestion[] = [
  // Rhai Keywords
  {
    label: "if",
    insertText: "if ",
    type: "keyword",
    description: "Conditional statement",
  },
  {
    label: "else",
    insertText: "else ",
    type: "keyword",
    description: "Alternative condition",
  },
  {
    label: "while",
    insertText: "while ",
    type: "keyword",
    description: "While loop",
  },
  {
    label: "for",
    insertText: "for ",
    type: "keyword",
    description: "For loop",
  },
  {
    label: "loop",
    insertText: "loop ",
    type: "keyword",
    description: "Infinite loop",
  },
  {
    label: "break",
    insertText: "break;",
    type: "keyword",
    description: "Exit loop",
  },
  {
    label: "continue",
    insertText: "continue;",
    type: "keyword",
    description: "Skip iteration",
  },
  {
    label: "return",
    insertText: "return ",
    type: "keyword",
    description: "Return value",
  },
  {
    label: "let",
    insertText: "let ",
    type: "keyword",
    description: "Variable declaration",
  },
  {
    label: "const",
    insertText: "const ",
    type: "keyword",
    description: "Constant declaration",
  },
  {
    label: "fn",
    insertText: "fn ",
    type: "keyword",
    description: "Function definition",
  },
  {
    label: "true",
    insertText: "true",
    type: "keyword",
    description: "Boolean true",
  },
  {
    label: "false",
    insertText: "false",
    type: "keyword",
    description: "Boolean false",
  },
  {
    label: "null",
    insertText: "null",
    type: "keyword",
    description: "Null value",
  },

  // Re:Earth Flow Environment Functions
  {
    label: "env.get",
    insertText: 'env.get("")',
    type: "function",
    description: "Get environment variable or context",
    detail: "env.get(name: string)",
  },
  {
    label: "env.set",
    insertText: 'env.set("", )',
    type: "function",
    description: "Set environment variable",
    detail: "env.set(name: string, value: any)",
  },

  // Context Variables
  {
    label: "__value",
    insertText: "__value",
    type: "variable",
    description: "Current feature attributes",
  },
  {
    label: "__feature_type",
    insertText: "__feature_type",
    type: "variable",
    description: "Current feature type",
  },
  {
    label: "__feature_id",
    insertText: "__feature_id",
    type: "variable",
    description: "Current feature ID",
  },
  {
    label: "__lod",
    insertText: "__lod",
    type: "variable",
    description: "Level of detail value",
  },

  // File Namespace
  {
    label: "file::",
    insertText: "file::",
    type: "namespace",
    description: "File operations namespace",
  },
  {
    label: "file::join_path",
    insertText: "file::join_path(, )",
    type: "function",
    description: "Join two path segments",
    detail: "file::join_path(path1: string, path2: string)",
  },
  {
    label: "file::extract_filename",
    insertText: "file::extract_filename()",
    type: "function",
    description: "Extract filename from path",
    detail: "file::extract_filename(path: string)",
  },
  {
    label: "file::extract_filename_without_ext",
    insertText: "file::extract_filename_without_ext()",
    type: "function",
    description: "Extract filename without extension",
    detail: "file::extract_filename_without_ext(path: string)",
  },

  // JSON Namespace
  {
    label: "json::",
    insertText: "json::",
    type: "namespace",
    description: "JSON operations namespace",
  },
  {
    label: "json::find_value_by_json_path",
    insertText: 'json::find_value_by_json_path(, "")',
    type: "function",
    description: "Find value using JSONPath",
    detail: "json::find_value_by_json_path(content: any, jsonPath: string)",
  },
  {
    label: "json::exists_value_by_json_path",
    insertText: 'json::exists_value_by_json_path(, "")',
    type: "function",
    description: "Check if JSONPath exists",
    detail: "json::exists_value_by_json_path(content: any, jsonPath: string)",
  },

  // String Namespace
  {
    label: "str::",
    insertText: "str::",
    type: "namespace",
    description: "String operations namespace",
  },
  {
    label: "str::extract_single_by_regex",
    insertText: 'str::extract_single_by_regex("", )',
    type: "function",
    description: "Extract first regex match",
    detail: "str::extract_single_by_regex(regex: string, text: string)",
  },

  // DateTime Namespace
  {
    label: "datetime::",
    insertText: "datetime::",
    type: "namespace",
    description: "DateTime operations namespace",
  },
  {
    label: "datetime::extract_year",
    insertText: "datetime::extract_year()",
    type: "function",
    description: "Extract year from datetime",
    detail: "datetime::extract_year(datetime: string)",
  },
  {
    label: "datetime::extract_month",
    insertText: "datetime::extract_month()",
    type: "function",
    description: "Extract month from datetime",
    detail: "datetime::extract_month(datetime: string)",
  },
  {
    label: "datetime::extract_day",
    insertText: "datetime::extract_day()",
    type: "function",
    description: "Extract day from datetime",
    detail: "datetime::extract_day(datetime: string)",
  },
  {
    label: "datetime::add_year",
    insertText: "datetime::add_year(, )",
    type: "function",
    description: "Add years to datetime",
    detail: "datetime::add_year(datetime: string, years: number)",
  },
  {
    label: "datetime::add_month",
    insertText: "datetime::add_month(, )",
    type: "function",
    description: "Add months to datetime",
    detail: "datetime::add_month(datetime: string, months: number)",
  },
  {
    label: "datetime::add_day",
    insertText: "datetime::add_day(, )",
    type: "function",
    description: "Add days to datetime",
    detail: "datetime::add_day(datetime: string, days: number)",
  },

  // Standard Rhai Functions
  {
    label: "len",
    insertText: "len()",
    type: "function",
    description: "Get length of string/array",
    detail: "len(value: any)",
  },
  {
    label: "round",
    insertText: "round()",
    type: "function",
    description: "Round number",
    detail: "round(number: number)",
  },
  {
    label: "floor",
    insertText: "floor()",
    type: "function",
    description: "Round down",
    detail: "floor(number: number)",
  },
  {
    label: "ceil",
    insertText: "ceil()",
    type: "function",
    description: "Round up",
    detail: "ceil(number: number)",
  },
  {
    label: "abs",
    insertText: "abs()",
    type: "function",
    description: "Absolute value",
    detail: "abs(number: number)",
  },
  {
    label: "min",
    insertText: "min(, )",
    type: "function",
    description: "Minimum value",
    detail: "min(a: number, b: number)",
  },
  {
    label: "max",
    insertText: "max(, )",
    type: "function",
    description: "Maximum value",
    detail: "max(a: number, b: number)",
  },
  {
    label: "to_string",
    insertText: "to_string()",
    type: "function",
    description: "Convert to string",
    detail: "to_string(value: any)",
  },
  {
    label: "contains",
    insertText: "contains()",
    type: "function",
    description: "Check if contains substring",
    detail: "string.contains(substring: string)",
  },
  {
    label: "starts_with",
    insertText: "starts_with()",
    type: "function",
    description: "Check if starts with substring",
    detail: "string.starts_with(substring: string)",
  },
  {
    label: "ends_with",
    insertText: "ends_with()",
    type: "function",
    description: "Check if ends with substring",
    detail: "string.ends_with(substring: string)",
  },

  // Operators
  { label: "==", insertText: "== ", type: "operator", description: "Equal to" },
  {
    label: "!=",
    insertText: "!= ",
    type: "operator",
    description: "Not equal to",
  },
  {
    label: "<=",
    insertText: "<= ",
    type: "operator",
    description: "Less than or equal",
  },
  {
    label: ">=",
    insertText: ">= ",
    type: "operator",
    description: "Greater than or equal",
  },
  {
    label: "&&",
    insertText: "&& ",
    type: "operator",
    description: "Logical AND",
  },
  {
    label: "||",
    insertText: "|| ",
    type: "operator",
    description: "Logical OR",
  },
];
