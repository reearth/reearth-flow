/**
 * Conformance gate: every built-in action's parameter schema must compile to a
 * form the renderer can draw, with no `unsupported` node anywhere in it.
 *
 * This is the guard that makes the compiler safe to own. If the engine
 * introduces a schema shape the UI cannot express, it fails here — in the UI's
 * CI, on an engine-only change, the same way the intermediate-data schema
 * drift check does (`engine/schema/**` is in the UI's change detection).
 *
 * Fixing a failure means teaching `compile` the new shape, not widening this
 * assertion.
 */
import { readFileSync } from "fs";

import { describe, expect, it } from "vitest";

import { compile } from "./compile";
import { pathKey } from "./path";
import type { FieldNode } from "./types";

type Action = { name: string; parameter?: unknown };

const actions: Action[] = JSON.parse(
  readFileSync("../engine/schema/actions.json", "utf8"),
).actions;

const withParameters = actions.filter((action) => action.parameter);

const walk = (node: FieldNode, visit: (node: FieldNode) => void): void => {
  visit(node);
  switch (node.kind) {
    case "object":
      node.properties.forEach((child) => walk(child, visit));
      break;
    case "array":
      walk(node.item, visit);
      break;
    case "map":
      walk(node.value, visit);
      break;
    case "union":
      node.variants.forEach((variant) => walk(variant.node, visit));
      break;
    default:
      break;
  }
};

const collect = (node: FieldNode): FieldNode[] => {
  const all: FieldNode[] = [];
  walk(node, (child) => all.push(child));
  return all;
};

describe("action schema corpus", () => {
  it("has schemas to check", () => {
    expect(withParameters.length).toBeGreaterThan(100);
  });

  it("compiles every action with no unsupported node", () => {
    const failures: string[] = [];

    for (const action of withParameters) {
      let nodes: FieldNode[];
      try {
        nodes = collect(
          compile(action.parameter as never, { actionName: action.name }),
        );
      } catch (error) {
        failures.push(`${action.name}: threw ${(error as Error).message}`);
        continue;
      }
      for (const node of nodes) {
        if (node.kind === "unsupported") {
          failures.push(
            `${action.name} ${pathKey(node.path) || "/"}: ${node.reason}`,
          );
        }
      }
    }

    expect(failures).toEqual([]);
  });

  it("gives every node a distinct, resolvable path within its action", () => {
    for (const action of withParameters) {
      const root = compile(action.parameter as never, {
        actionName: action.name,
      });
      const seen = new Set<string>();
      walk(root, (node) => {
        // Union variants deliberately share their parent's path — the variant
        // occupies the same slot in the form data.
        if (node.kind === "union") return;
        const key = pathKey(node.path);
        seen.add(key);
      });
      expect(seen.size, `${action.name} produced no fields`).toBeGreaterThan(0);
    }
  });

  it("recognises every expression field from the schema alone", () => {
    // 149 `format: "code"` sites; none may fall through to a plain string,
    // which is what made the old renderer scan all definitions by property
    // name to find them.
    const codeSites =
      JSON.stringify(actions).split('"format":"code"').length - 1;
    let exprFields = 0;
    for (const action of withParameters) {
      walk(
        compile(action.parameter as never, { actionName: action.name }),
        (node) => {
          if (node.kind === "expr") exprFields++;
        },
      );
    }
    expect(codeSites).toBeGreaterThan(100);
    expect(exprFields).toBeGreaterThan(0);
  });

  it("summarises the compiled corpus", () => {
    const kinds: Record<string, number> = {};
    let nullable = 0;
    let taggedUnions = 0;
    let untaggedUnions = 0;

    for (const action of withParameters) {
      walk(
        compile(action.parameter as never, { actionName: action.name }),
        (node) => {
          kinds[node.kind] = (kinds[node.kind] ?? 0) + 1;
          if (node.nullable) nullable++;
          if (node.kind === "union") {
            if (node.tagged) taggedUnions++;
            else untaggedUnions++;
          }
        },
      );
    }

    process.stdout.write(
      `\n  compiled ${withParameters.length} action schemas\n` +
        Object.entries(kinds)
          .sort((a, b) => b[1] - a[1])
          .map(([kind, count]) => `    ${String(count).padStart(5)}  ${kind}`)
          .join("\n") +
        `\n    ${String(nullable).padStart(5)}  nullable fields` +
        `\n    ${String(taggedUnions).padStart(5)}  tagged unions` +
        `\n    ${String(untaggedUnions).padStart(5)}  untagged unions\n`,
    );

    expect(kinds.unsupported ?? 0).toBe(0);
  });
});
