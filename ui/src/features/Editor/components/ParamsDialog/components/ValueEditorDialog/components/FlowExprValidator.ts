import { type ValidationError } from "./RhaiValidator";

export type { ValidationError };

class FlowExprValidator {
  private code: string;
  private errors: ValidationError[] = [];

  constructor(code: string) {
    this.code = code;
  }

  validate(): ValidationError[] {
    this.errors = [];
    this.validateBracketMatching();
    this.validateStrings();
    return this.errors;
  }

  private validateBracketMatching(): void {
    const brackets: Record<string, string> = { "(": ")", "[": "]", "{": "}" };
    const stack: { bracket: string; line: number; column: number }[] = [];
    const lines = this.code.split("\n");

    for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
      const line = lines[lineIndex];
      let i = 0;
      let inString = false;

      while (i < line.length) {
        const char = line[i];

        if (inString) {
          if (char === "\\" && i + 1 < line.length) {
            i += 2;
            continue;
          }
          if (char === '"') inString = false;
          i++;
          continue;
        }

        if (char === '"') {
          inString = true;
          i++;
          continue;
        }

        if (char === "(" || char === "[" || char === "{") {
          stack.push({ bracket: char, line: lineIndex, column: i });
        } else if (char === ")" || char === "]" || char === "}") {
          if (stack.length === 0) {
            this.errors.push({
              line: lineIndex,
              column: i,
              length: 1,
              message: `Unmatched '${char}'`,
              severity: "error",
              type: "syntax",
            });
          } else {
            const last = stack.pop();
            const expected = last ? brackets[last.bracket] : undefined;
            if (char !== expected) {
              this.errors.push({
                line: lineIndex,
                column: i,
                length: 1,
                message: last
                  ? `Expected '${expected}' to match '${last.bracket}' at line ${last.line + 1}`
                  : `Unexpected closing bracket '${char}'`,
                severity: "error",
                type: "syntax",
              });
            }
          }
        }

        i++;
      }
    }

    for (const unclosed of stack) {
      const expected = brackets[unclosed.bracket];
      this.errors.push({
        line: unclosed.line,
        column: unclosed.column,
        length: 1,
        message: `Unmatched '${unclosed.bracket}', expected '${expected}'`,
        severity: "error",
        type: "syntax",
      });
    }
  }

  private validateStrings(): void {
    const lines = this.code.split("\n");

    for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
      const line = lines[lineIndex];
      let i = 0;

      while (i < line.length) {
        if (line[i] === '"') {
          const start = i;
          i++;
          let closed = false;

          while (i < line.length) {
            if (line[i] === "\\" && i + 1 < line.length) {
              i += 2;
              continue;
            }
            if (line[i] === '"') {
              closed = true;
              i++;
              break;
            }
            i++;
          }

          if (!closed) {
            this.errors.push({
              line: lineIndex,
              column: start,
              length: line.length - start,
              message: "Unclosed string literal",
              severity: "error",
              type: "syntax",
            });
          }
        } else {
          i++;
        }
      }
    }
  }
}

export function validateFlowExprCode(code: string): ValidationError[] {
  return new FlowExprValidator(code).validate();
}
