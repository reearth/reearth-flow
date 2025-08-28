import { RHAI_KEYWORDS, RHAI_FUNCTIONS, RHAI_NAMESPACES } from "./constants";

export type ValidationError = {
  line: number;
  column: number;
  length: number;
  message: string;
  severity: "error" | "warning";
  type: "syntax" | "semantic" | "undefined-variable" | "invalid-function";
};

export class RhaiValidator {
  private code: string;
  private errors: ValidationError[] = [];

  constructor(code: string) {
    this.code = code;
  }

  validate(): ValidationError[] {
    this.errors = [];
    this.validateSyntax();
    this.validateSemantics();
    return this.errors;
  }

  private validateSyntax(): void {
    const lines = this.code.split("\n");

    for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
      const line = lines[lineIndex];
      this.validateLine(line, lineIndex);
    }
  }

  private validateLine(line: string, lineIndex: number): void {
    let i = 0;

    while (i < line.length) {
      const char = line[i];

      // Skip whitespace
      if (/\s/.test(char)) {
        i++;
        continue;
      }

      // Skip comments
      if (char === "/" && line[i + 1] === "/") {
        break; // Rest of line is comment
      }

      // String validation
      if (char === '"' || char === "'") {
        const stringResult = this.validateString(line, i, lineIndex, char);
        i = stringResult.nextIndex;
        continue;
      }

      // Parentheses and bracket matching
      if (char === "(" || char === "[" || char === "{") {
        this.validateBracketMatching(line, i, lineIndex, char);
      }

      // Identifier validation (keywords, functions, variables)
      if (/[a-zA-Z_]/.test(char)) {
        const identifierResult = this.validateIdentifier(line, i, lineIndex);
        i = identifierResult.nextIndex;
        continue;
      }

      // Namespace validation
      if (this.isNamespaceOperator(line, i)) {
        this.validateNamespace(line, i, lineIndex);
        i += 2; // Skip ::
        continue;
      }

      i++;
    }
  }

  private validateString(
    line: string,
    startIndex: number,
    lineIndex: number,
    quote: string,
  ): { nextIndex: number } {
    let i = startIndex + 1;
    let escaped = false;

    while (i < line.length) {
      const char = line[i];

      if (escaped) {
        escaped = false;
      } else if (char === "\\") {
        escaped = true;
      } else if (char === quote) {
        return { nextIndex: i + 1 };
      }
      i++;
    }

    // Unclosed string
    this.addError({
      line: lineIndex,
      column: startIndex,
      length: line.length - startIndex,
      message: "Unclosed string literal",
      severity: "error",
      type: "syntax",
    });

    return { nextIndex: line.length };
  }

  private validateIdentifier(
    line: string,
    startIndex: number,
    lineIndex: number,
  ): { nextIndex: number } {
    let identifier = "";
    let i = startIndex;

    // Extract identifier
    while (i < line.length && /[a-zA-Z0-9_]/.test(line[i])) {
      identifier += line[i];
      i++;
    }

    // Check for namespace prefix (identifier followed by ::)
    if (line.substring(i, i + 2) === "::") {
      if (!RHAI_NAMESPACES.includes(identifier)) {
        this.addError({
          line: lineIndex,
          column: startIndex,
          length: identifier.length,
          message: `Unknown namespace '${identifier}'. Available namespaces: ${RHAI_NAMESPACES.join(", ")}`,
          severity: "error",
          type: "undefined-variable",
        });
      }
      return { nextIndex: i };
    }

    // Check if it's followed by parentheses (function call)
    const nextNonWhitespace = this.findNextNonWhitespace(line, i);
    if (nextNonWhitespace?.char === "(") {
      // Function call validation
      if (
        !RHAI_KEYWORDS.includes(identifier) &&
        !RHAI_FUNCTIONS.includes(identifier)
      ) {
        // Check if it might be a namespaced function call
        const fullFunction = this.extractFullFunctionName(line, startIndex);
        if (!this.isValidNamespacedFunction(fullFunction)) {
          this.addError({
            line: lineIndex,
            column: startIndex,
            length: identifier.length,
            message: `Unknown function '${identifier}'`,
            severity: "warning", // Warning since it might be a user-defined function
            type: "invalid-function",
          });
        }
      }
    }

    // Check for common context variable typos
    if (
      identifier.startsWith("__") &&
      !this.isValidContextVariable(identifier)
    ) {
      const suggestions = this.getContextVariableSuggestions(identifier);
      this.addError({
        line: lineIndex,
        column: startIndex,
        length: identifier.length,
        message: `Unknown context variable '${identifier}'${suggestions ? `. Did you mean: ${suggestions}?` : ""}`,
        severity: "warning",
        type: "undefined-variable",
      });
    }

    return { nextIndex: i };
  }

  private validateBracketMatching(
    line: string,
    startIndex: number,
    lineIndex: number,
    openBracket: string,
  ): void {
    const closeBracket = { "(": ")", "[": "]", "{": "}" }[openBracket];
    let depth = 1;
    let i = startIndex + 1;

    while (i < line.length && depth > 0) {
      const char = line[i];

      // Skip strings
      if (char === '"' || char === "'") {
        const stringResult = this.skipString(line, i, char);
        i = stringResult.nextIndex;
        continue;
      }

      if (char === openBracket) depth++;
      if (char === closeBracket) depth--;
      i++;
    }

    if (depth > 0) {
      this.addError({
        line: lineIndex,
        column: startIndex,
        length: 1,
        message: `Unmatched '${openBracket}'`,
        severity: "error",
        type: "syntax",
      });
    }
  }

  private validateSemantics(): void {
    // Check for common semantic errors
    this.validateVariableUsage();
    this.validateFunctionCalls();
  }

  private validateVariableUsage(): void {
    const lines = this.code.split("\n");
    const declaredVariables = new Set<string>();

    // First pass: collect variable declarations
    for (const line of lines) {
      const letMatch = line.match(/\blet\s+([a-zA-Z_][a-zA-Z0-9_]*)/g);
      const constMatch = line.match(/\bconst\s+([a-zA-Z_][a-zA-Z0-9_]*)/g);

      [letMatch, constMatch].forEach((matches) => {
        matches?.forEach((match) => {
          const varName = match.split(/\s+/)[1];
          if (varName) declaredVariables.add(varName);
        });
      });
    }

    // Add context variables
    declaredVariables.add("__value");
    declaredVariables.add("__feature_type");
    declaredVariables.add("__feature_id");
    declaredVariables.add("__lod");
  }

  private validateFunctionCalls(): void {
    const lines = this.code.split("\n");

    for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
      const line = lines[lineIndex];

      // Check for env.get() calls
      const envGetMatches = line.matchAll(/env\.get\s*\(\s*([^)]*)\)/g);
      for (const match of envGetMatches) {
        const arg = match[1].trim();
        if (!arg.startsWith('"') && !arg.startsWith("'")) {
          const matchIndex = match.index ?? 0;
          this.addError({
            line: lineIndex,
            column: matchIndex + match[0].indexOf("(") + 1,
            length: arg.length,
            message: "env.get() requires a string argument",
            severity: "error",
            type: "semantic",
          });
        }
      }

      // Check for JSON path functions
      const jsonPathMatches = line.matchAll(
        /(json::(?:find_value_by_json_path|exists_value_by_json_path))\s*\(\s*([^,)]+)(?:,\s*([^)]+))?\)/g,
      );
      for (const match of jsonPathMatches) {
        const jsonPathArg = match[3]?.trim();
        if (
          jsonPathArg &&
          !jsonPathArg.startsWith('"') &&
          !jsonPathArg.startsWith("'")
        ) {
          const matchIndex = match.index ?? 0;
          this.addError({
            line: lineIndex,
            column: matchIndex + match[0].lastIndexOf(",") + 1,
            length: jsonPathArg.length,
            message: "JSONPath must be a string literal",
            severity: "warning",
            type: "semantic",
          });
        }
      }
    }
  }

  // Helper methods
  private isNamespaceOperator(line: string, index: number): boolean {
    return line.substring(index, index + 2) === "::";
  }

  private validateNamespace(
    line: string,
    index: number,
    lineIndex: number,
  ): void {
    // Find the identifier before ::
    let start = index - 1;
    while (start >= 0 && /[a-zA-Z0-9_]/.test(line[start])) {
      start--;
    }
    start++;

    const namespace = line.substring(start, index);
    if (namespace && !RHAI_NAMESPACES.includes(namespace)) {
      this.addError({
        line: lineIndex,
        column: start,
        length: namespace.length,
        message: `Unknown namespace '${namespace}'`,
        severity: "error",
        type: "undefined-variable",
      });
    }
  }

  private findNextNonWhitespace(
    line: string,
    startIndex: number,
  ): { char: string; index: number } | null {
    for (let i = startIndex; i < line.length; i++) {
      if (!/\s/.test(line[i])) {
        return { char: line[i], index: i };
      }
    }
    return null;
  }

  private extractFullFunctionName(line: string, startIndex: number): string {
    // Look backwards for potential namespace
    let backStart = startIndex - 1;

    // Skip whitespace
    while (backStart >= 0 && /\s/.test(line[backStart])) {
      backStart--;
    }

    // Check for ::
    if (
      backStart >= 1 &&
      line.substring(backStart - 1, backStart + 1) === "::"
    ) {
      // Find namespace start
      let nsStart = backStart - 2;
      while (nsStart >= 0 && /[a-zA-Z0-9_]/.test(line[nsStart])) {
        nsStart--;
      }
      nsStart++;

      // Extract function name
      let funcEnd = startIndex;
      while (funcEnd < line.length && /[a-zA-Z0-9_]/.test(line[funcEnd])) {
        funcEnd++;
      }

      return line.substring(nsStart, funcEnd);
    }

    return "";
  }

  private isValidNamespacedFunction(fullFunctionName: string): boolean {
    // Check against known namespaced functions in constants
    const validFunctions = [
      "env.get",
      "env.set",
      "file::join_path",
      "file::extract_filename",
      "file::extract_filename_without_ext",
      "json::find_value_by_json_path",
      "json::exists_value_by_json_path",
      "str::extract_single_by_regex",
      "datetime::extract_year",
      "datetime::extract_month",
      "datetime::extract_day",
      "datetime::add_year",
      "datetime::add_month",
      "datetime::add_day",
    ];

    return validFunctions.includes(fullFunctionName);
  }

  private isValidContextVariable(identifier: string): boolean {
    const validContextVars = [
      "__value",
      "__feature_type",
      "__feature_id",
      "__lod",
    ];
    return validContextVars.includes(identifier);
  }

  private getContextVariableSuggestions(identifier: string): string | null {
    const validContextVars = [
      "__value",
      "__feature_type",
      "__feature_id",
      "__lod",
    ];
    const suggestions = validContextVars.filter(
      (v) =>
        v.toLowerCase().includes(identifier.toLowerCase().substring(2)) ||
        this.levenshteinDistance(identifier, v) <= 2,
    );

    return suggestions.length > 0 ? suggestions.join(", ") : null;
  }

  private levenshteinDistance(str1: string, str2: string): number {
    const matrix = Array(str2.length + 1)
      .fill(null)
      .map(() => Array(str1.length + 1).fill(null));

    for (let i = 0; i <= str1.length; i++) matrix[0][i] = i;
    for (let j = 0; j <= str2.length; j++) matrix[j][0] = j;

    for (let j = 1; j <= str2.length; j++) {
      for (let i = 1; i <= str1.length; i++) {
        const indicator = str1[i - 1] === str2[j - 1] ? 0 : 1;
        matrix[j][i] = Math.min(
          matrix[j][i - 1] + 1,
          matrix[j - 1][i] + 1,
          matrix[j - 1][i - 1] + indicator,
        );
      }
    }

    return matrix[str2.length][str1.length];
  }

  private skipString(
    line: string,
    startIndex: number,
    quote: string,
  ): { nextIndex: number } {
    let i = startIndex + 1;
    let escaped = false;

    while (i < line.length) {
      const char = line[i];
      if (escaped) {
        escaped = false;
      } else if (char === "\\") {
        escaped = true;
      } else if (char === quote) {
        return { nextIndex: i + 1 };
      }
      i++;
    }

    return { nextIndex: line.length };
  }

  private addError(error: ValidationError): void {
    this.errors.push(error);
  }
}

// Utility function for external use
export function validateRhaiCode(code: string): ValidationError[] {
  const validator = new RhaiValidator(code);
  return validator.validate();
}
