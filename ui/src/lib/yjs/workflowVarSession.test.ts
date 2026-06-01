import { describe, it, expect } from "vitest";

import { WorkflowVariable } from "@flow/types";

import { computeSessionChanges } from "./workflowVarSession";

const makeVar = (partial: Partial<WorkflowVariable>): WorkflowVariable => ({
  id: "id1",
  name: "Test Variable",
  defaultValue: "default",
  type: "text",
  required: true,
  public: true,
  ...partial,
});

describe("computeSessionChanges", () => {
  it("returns all empty arrays when current equals base", () => {
    const base = [
      makeVar({ id: "id1" }),
      makeVar({ id: "id2", name: "Var 2" }),
    ];
    expect(computeSessionChanges([...base], base)).toEqual({
      creates: [],
      updates: [],
      deletes: [],
      reorders: [],
    });
  });

  it("detects a create for a temp_ id", () => {
    const base = [makeVar({ id: "id1" })];
    const current = [...base, makeVar({ id: "temp_123", name: "New Var" })];
    const result = computeSessionChanges(current, base);
    expect(result.creates).toHaveLength(1);
    expect(result.creates[0]).toMatchObject({
      name: "New Var",
      type: "text",
      required: true,
      publicValue: true,
      index: 1,
    });
    expect(result.updates).toHaveLength(0);
    expect(result.deletes).toHaveLength(0);
  });

  it("detects a delete when a base variable is absent from current", () => {
    const base = [makeVar({ id: "id1" }), makeVar({ id: "id2" })];
    const current = [makeVar({ id: "id1" })];
    const result = computeSessionChanges(current, base);
    expect(result.deletes).toEqual(["id2"]);
    expect(result.creates).toHaveLength(0);
    expect(result.updates).toHaveLength(0);
  });

  it("detects an update when the name changes", () => {
    const base = [makeVar({ id: "id1", name: "Old Name" })];
    const current = [makeVar({ id: "id1", name: "New Name" })];
    const result = computeSessionChanges(current, base);
    expect(result.updates).toHaveLength(1);
    expect(result.updates[0]).toMatchObject({
      paramId: "id1",
      name: "New Name",
    });
    expect(result.creates).toHaveLength(0);
    expect(result.deletes).toHaveLength(0);
    expect(result.reorders).toHaveLength(0);
  });

  it("detects an update when defaultValue changes", () => {
    const base = [makeVar({ id: "id1", defaultValue: "old" })];
    const current = [makeVar({ id: "id1", defaultValue: "new" })];
    const result = computeSessionChanges(current, base);
    expect(result.updates).toHaveLength(1);
    expect(result.updates[0].defaultValue).toBe("new");
  });

  it("detects an update when config changes", () => {
    const base = [makeVar({ id: "id1", type: "number", config: { min: 0 } })];
    const current = [
      makeVar({ id: "id1", type: "number", config: { min: 5 } }),
    ];
    const result = computeSessionChanges(current, base);
    expect(result.updates).toHaveLength(1);
    expect(result.updates[0].config).toEqual({ min: 5 });
  });

  it("detects a reorder when items swap positions", () => {
    const base = [makeVar({ id: "id1" }), makeVar({ id: "id2" })];
    const current = [makeVar({ id: "id2" }), makeVar({ id: "id1" })];
    const result = computeSessionChanges(current, base);
    expect(result.reorders).toContainEqual({ paramId: "id2", newIndex: 0 });
    expect(result.reorders).toContainEqual({ paramId: "id1", newIndex: 1 });
    expect(result.creates).toHaveLength(0);
    expect(result.deletes).toHaveLength(0);
  });

  it("does not emit reorders when order is unchanged", () => {
    const base = [makeVar({ id: "id1" }), makeVar({ id: "id2" })];
    const result = computeSessionChanges([...base], base);
    expect(result.reorders).toHaveLength(0);
  });

  it("handles multiple creates", () => {
    const base: WorkflowVariable[] = [];
    const current = [
      makeVar({ id: "temp_1", name: "Alpha" }),
      makeVar({ id: "temp_2", name: "Beta" }),
    ];
    const result = computeSessionChanges(current, base);
    expect(result.creates).toHaveLength(2);
    expect(result.creates[0]).toMatchObject({ name: "Alpha", index: 0 });
    expect(result.creates[1]).toMatchObject({ name: "Beta", index: 1 });
    expect(result.deletes).toHaveLength(0);
  });

  it("handles combined create, update, delete, and reorder", () => {
    const base = [
      makeVar({ id: "id1", name: "Unchanged" }),
      makeVar({ id: "id2", name: "Will Update" }),
      makeVar({ id: "id3", name: "Will Delete" }),
    ];
    const current = [
      makeVar({ id: "id2", name: "Updated" }),
      makeVar({ id: "id1", name: "Unchanged" }),
      makeVar({ id: "temp_new", name: "New" }),
    ];
    const result = computeSessionChanges(current, base);
    expect(result.creates).toHaveLength(1);
    expect(result.creates[0].name).toBe("New");
    expect(result.updates).toHaveLength(1);
    expect(result.updates[0]).toMatchObject({
      paramId: "id2",
      name: "Updated",
    });
    expect(result.deletes).toEqual(["id3"]);
    expect(result.reorders.length).toBeGreaterThan(0);
  });

  it("maps the public field to publicValue in creates", () => {
    const current = [makeVar({ id: "temp_1", public: false })];
    const result = computeSessionChanges(current, []);
    expect(result.creates[0].publicValue).toBe(false);
  });

  it("maps the public field to publicValue in updates", () => {
    const base = [makeVar({ id: "id1", public: true })];
    const current = [makeVar({ id: "id1", public: false })];
    const result = computeSessionChanges(current, base);
    expect(result.updates[0].publicValue).toBe(false);
  });

  it("does not include temp_ ids in updates or deletes", () => {
    const base: WorkflowVariable[] = [];
    const current = [makeVar({ id: "temp_x", name: "New" })];
    const result = computeSessionChanges(current, base);
    expect(result.updates).toHaveLength(0);
    expect(result.deletes).toHaveLength(0);
  });
});
