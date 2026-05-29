import { useMemo } from "react";

import {
  FLOWEXPR_BUILTIN_FUNCTIONS,
  FLOWEXPR_KEYWORDS,
  FLOWEXPR_MATH_NAMESPACE,
  FLOWEXPR_OPERATORS,
} from "./flowExprConstants";

type TokenType =
  | "keyword"
  | "function"
  | "namespace"
  | "string"
  | "number"
  | "operator"
  | "identifier"
  | "punctuation"
  | "default";

type Token = {
  type: TokenType;
  content: string;
};

type Props = {
  code: string;
  className?: string;
};

const FlowExprSyntaxHighlighter: React.FC<Props> = ({
  code,
  className = "",
}) => {
  const tokens = useMemo(() => {
    if (!code) return [];

    const result: Token[] = [];
    let i = 0;

    while (i < code.length) {
      const char = code[i];

      // Whitespace
      if (/\s/.test(char)) {
        let whitespace = "";
        while (i < code.length && /\s/.test(code[i])) {
          whitespace += code[i];
          i++;
        }
        result.push({ type: "default", content: whitespace });
        continue;
      }

      // String literals (double-quoted only in FlowExpr)
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

      // Numbers (integer and float)
      if (/\d/.test(char) || (char === "." && /\d/.test(code[i + 1]))) {
        let number = "";
        while (i < code.length && /[\d.]/.test(code[i])) {
          number += code[i];
          i++;
        }
        result.push({ type: "number", content: number });
        continue;
      }

      // Multi-character operators (longest match first)
      let foundOperator = false;
      for (const op of FLOWEXPR_OPERATORS.sort((a, b) => b.length - a.length)) {
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

      // Identifiers, keywords, functions, math namespace
      if (/[a-zA-Z_]/.test(char)) {
        let identifier = "";
        while (i < code.length && /[a-zA-Z0-9_]/.test(code[i])) {
          identifier += code[i];
          i++;
        }

        // math:: namespace
        if (
          identifier === FLOWEXPR_MATH_NAMESPACE &&
          code.substring(i, i + 2) === "::"
        ) {
          result.push({ type: "namespace", content: identifier });
          result.push({ type: "operator", content: "::" });
          i += 2;
          continue;
        }

        let tokenType: TokenType = "identifier";
        if (FLOWEXPR_KEYWORDS.includes(identifier)) {
          tokenType = "keyword";
        } else if (FLOWEXPR_BUILTIN_FUNCTIONS.includes(identifier)) {
          tokenType = "function";
        }

        result.push({ type: tokenType, content: identifier });
        continue;
      }

      result.push({ type: "default", content: char });
      i++;
    }

    return result;
  }, [code]);

  const getTokenClassName = (type: TokenType): string => {
    switch (type) {
      case "keyword":
        return "text-purple-600 dark:text-purple-400";
      case "function":
        return "text-blue-600 dark:text-blue-400";
      case "namespace":
        return "text-teal-600 dark:text-teal-400";
      case "string":
        return "text-green-600 dark:text-green-400";
      case "number":
        return "text-orange-600 dark:text-orange-400";
      case "operator":
        return "text-red-600 dark:text-red-400";
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
        <span
          key={index}
          className={getTokenClassName(token.type)}
          style={{
            fontFamily: "inherit",
            fontSize: "inherit",
            fontWeight: "inherit",
            letterSpacing: "inherit",
          }}>
          {token.content}
        </span>
      ))}
    </div>
  );
};

export default FlowExprSyntaxHighlighter;
