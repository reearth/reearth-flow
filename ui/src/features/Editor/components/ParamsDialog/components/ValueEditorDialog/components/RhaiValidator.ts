// Validator for the Flow Expression Language (FEL)

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
    this.validateMultiLineBracketMatching();

    const lines = this.code.split("\n");
    for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
      this.validateLine(lines[lineIndex], lineIndex);
    }
  }

  private validateLine(line: string, lineIndex: number): void {
    let i = 0;
    while (i < line.length) {
      const char = line[i];

      if (/\s/.test(char)) {
        i++;
        continue;
      }

      if (char === '"' || char === "'") {
        const result = this.validateString(line, i, lineIndex, char);
        i = result.nextIndex;
        continue;
      }

      if (char === "(") {
        this.validateSingleLineBracketMatching(line, i, lineIndex, char);
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

  private validateMultiLineBracketMatching(): void {
    const brackets = { "(": ")", "[": "]", "{": "}" };
    const stack: { bracket: string; line: number; column: number }[] = [];
    const lines = this.code.split("\n");

    for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
      const line = lines[lineIndex];
      let i = 0;

      while (i < line.length) {
        const char = line[i];

        if (char === '"' || char === "'") {
          const result = this.skipString(line, i, char);
          i = result.nextIndex;
          continue;
        }

        if (char === "(" || char === "[" || char === "{") {
          stack.push({ bracket: char, line: lineIndex, column: i });
        }

        if (char === ")" || char === "]" || char === "}") {
          if (stack.length === 0) {
            this.addError({
              line: lineIndex,
              column: i,
              length: 1,
              message: `Unmatched '${char}'`,
              severity: "error",
              type: "syntax",
            });
          } else {
            const last = stack.pop();
            if (last) {
              const expectedClosing =
                brackets[last.bracket as keyof typeof brackets];
              if (char !== expectedClosing) {
                this.addError({
                  line: lineIndex,
                  column: i,
                  length: 1,
                  message: `Expected '${expectedClosing}' to match '${last.bracket}' at line ${last.line + 1}`,
                  severity: "error",
                  type: "syntax",
                });
              }
            }
          }
        }

        i++;
      }
    }

    for (const unclosed of stack) {
      const expectedClosing =
        brackets[unclosed.bracket as keyof typeof brackets];
      this.addError({
        line: unclosed.line,
        column: unclosed.column,
        length: 1,
        message: `Unmatched '${unclosed.bracket}', expected '${expectedClosing}'`,
        severity: "error",
        type: "syntax",
      });
    }
  }

  private validateSingleLineBracketMatching(
    line: string,
    startIndex: number,
    lineIndex: number,
    openBracket: string,
  ): void {
    const closeBracket = { "(": ")" }[openBracket];
    if (!closeBracket) return;

    let depth = 1;
    let i = startIndex + 1;

    while (i < line.length && depth > 0) {
      const char = line[i];
      if (char === '"' || char === "'") {
        const result = this.skipString(line, i, char);
        i = result.nextIndex;
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
    const lines = this.code.split("\n");

    for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
      const line = lines[lineIndex];
      this.validateContextFunctionCalls(line, lineIndex);
    }
  }

  // value("name") and env("name") must receive exactly one string argument
  private validateContextFunctionCalls(line: string, lineIndex: number): void {
    const fnNames = ["value", "env", "Url"];

    for (const fn of fnNames) {
      const pattern = new RegExp(`\\b${fn}\\s*\\(\\s*([^)]*)\\)`, "g");
      const matches = line.matchAll(pattern);

      for (const match of matches) {
        const arg = match[1].trim();
        if (!arg) {
          const matchIndex = match.index ?? 0;
          this.addError({
            line: lineIndex,
            column: matchIndex,
            length: match[0].length,
            message: `${fn}() requires a string argument`,
            severity: "error",
            type: "semantic",
          });
        } else if (!arg.startsWith('"') && !arg.startsWith("'")) {
          // Allow variable references (identifiers) as arguments without error,
          // but warn when it looks like a bare word that should be quoted
          if (/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(arg)) {
            const matchIndex = match.index ?? 0;
            this.addError({
              line: lineIndex,
              column: matchIndex,
              length: match[0].length,
              message: `${fn}() argument looks like an unquoted string — did you mean ${fn}("${arg}")?`,
              severity: "warning",
              type: "semantic",
            });
          }
        }
      }
    }
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

export function validateRhaiCode(code: string): ValidationError[] {
  const validator = new RhaiValidator(code);
  return validator.validate();
}
