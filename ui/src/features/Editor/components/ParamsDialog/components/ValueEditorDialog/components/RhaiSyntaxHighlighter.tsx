import { useMemo } from "react";

type TokenType =
  | "keyword"
  | "function"
  | "namespace"
  | "string"
  | "number"
  | "operator"
  | "comment"
  | "identifier"
  | "punctuation"
  | "default";

type Token = {
  type: TokenType;
  content: string;
};

const RHAI_KEYWORDS = [
  "if", "else", "while", "for", "loop", "break", "continue", "return",
  "let", "const", "fn", "private", "import", "export", "as", "true", "false", "null"
];

const RHAI_FUNCTIONS = [
  // Global environment functions (custom Re:Earth Flow functions)
  "env", "get",
  // File namespace functions - essential for path manipulation
  "extract_filename", "extract_filename_without_ext", "join_path",
  // String namespace functions - useful for regex operations
  "extract_single_by_regex",
  // JSON namespace functions - valuable for data processing
  "find_value_by_json_path", "exists_value_by_json_path",
  // Key DateTime namespace functions - common date operations
  "extract_year", "extract_month", "extract_day", 
  "add_year", "add_month", "add_day",
  // Standard Rhai math functions
  "round", "floor", "ceil", "abs", "sqrt", "min", "max", "pow", "sin", "cos", "tan",
  // Standard Rhai string functions
  "len", "is_empty", "contains", "starts_with", "ends_with", "to_string", "to_upper", "to_lower",
  // Standard Rhai array functions
  "push", "pop", "shift", "unshift", "insert", "remove"
];

const RHAI_OPERATORS = [
  "==", "!=", "<=", ">=", "<", ">", "&&", "||", "!", "+", "-", "*", "/", "%", "**", 
  "=", "+=", "-=", "*=", "/=", "%=", "**=", "?", ":"
];

const RHAI_NAMESPACES = ["file", "str", "json", "datetime"];

type Props = {
  code: string;
  className?: string;
};

const RhaiSyntaxHighlighter: React.FC<Props> = ({ code, className = "" }) => {
  const tokens = useMemo(() => {
    if (!code) return [];
    
    const result: Token[] = [];
    let i = 0;
    
    while (i < code.length) {
      const char = code[i];
      
      // Skip whitespace but preserve it
      if (/\s/.test(char)) {
        let whitespace = "";
        while (i < code.length && /\s/.test(code[i])) {
          whitespace += code[i];
          i++;
        }
        result.push({ type: "default", content: whitespace });
        continue;
      }
      
      // Single line comments
      if (char === "/" && code[i + 1] === "/") {
        let comment = "";
        while (i < code.length && code[i] !== "\n") {
          comment += code[i];
          i++;
        }
        result.push({ type: "comment", content: comment });
        continue;
      }
      
      // String literals (double quotes)
      if (char === '"') {
        let string = '"';
        i++;
        while (i < code.length) {
          const c = code[i];
          string += c;
          if (c === '"' && code[i - 1] !== "\\") {
            i++;
            break;
          }
          i++;
        }
        result.push({ type: "string", content: string });
        continue;
      }
      
      // String literals (single quotes)  
      if (char === "'") {
        let string = "'";
        i++;
        while (i < code.length) {
          const c = code[i];
          string += c;
          if (c === "'" && code[i - 1] !== "\\") {
            i++;
            break;
          }
          i++;
        }
        result.push({ type: "string", content: string });
        continue;
      }
      
      // Numbers
      if (/\d/.test(char) || (char === "." && /\d/.test(code[i + 1]))) {
        let number = "";
        while (i < code.length && (/[\d.]/.test(code[i]))) {
          number += code[i];
          i++;
        }
        result.push({ type: "number", content: number });
        continue;
      }
      
      // Multi-character operators
      let foundOperator = false;
      for (const op of RHAI_OPERATORS.sort((a, b) => b.length - a.length)) {
        if (code.substring(i, i + op.length) === op) {
          result.push({ type: "operator", content: op });
          i += op.length;
          foundOperator = true;
          break;
        }
      }
      if (foundOperator) continue;
      
      // Punctuation
      if (/[(){}[\];,.]/.test(char)) {
        result.push({ type: "punctuation", content: char });
        i++;
        continue;
      }
      
      // Identifiers, keywords, functions
      if (/[a-zA-Z_]/.test(char)) {
        let identifier = "";
        while (i < code.length && /[a-zA-Z0-9_]/.test(code[i])) {
          identifier += code[i];
          i++;
        }
        
        // Check if it's followed by :: (namespace)
        if (code.substring(i, i + 2) === "::" && RHAI_NAMESPACES.includes(identifier)) {
          result.push({ type: "namespace", content: identifier });
          result.push({ type: "operator", content: "::" });
          i += 2;
          continue;
        }
        
        // Determine token type
        let tokenType: TokenType = "identifier";
        if (RHAI_KEYWORDS.includes(identifier)) {
          tokenType = "keyword";
        } else if (RHAI_FUNCTIONS.includes(identifier)) {
          tokenType = "function";
        }
        
        result.push({ type: tokenType, content: identifier });
        continue;
      }
      
      // Default - any other character
      result.push({ type: "default", content: char });
      i++;
    }
    
    return result;
  }, [code]);
  
  const getTokenClassName = (type: TokenType): string => {
    switch (type) {
      case "keyword":
        return "text-purple-600 dark:text-purple-400 font-medium";
      case "function":
        return "text-blue-600 dark:text-blue-400 font-medium";
      case "namespace":
        return "text-teal-600 dark:text-teal-400 font-medium";
      case "string":
        return "text-green-600 dark:text-green-400";
      case "number":
        return "text-orange-600 dark:text-orange-400";
      case "operator":
        return "text-red-600 dark:text-red-400 font-medium";
      case "comment":
        return "text-gray-500 dark:text-gray-400 italic";
      case "punctuation":
        return "text-gray-700 dark:text-gray-300";
      case "identifier":
        return "text-gray-900 dark:text-gray-100";
      default:
        return "text-gray-900 dark:text-gray-100";
    }
  };
  
  return (
    <div className={className}>
      {tokens.map((token, index) => (
        <span key={index} className={getTokenClassName(token.type)}>
          {token.content}
        </span>
      ))}
    </div>
  );
};

export default RhaiSyntaxHighlighter;