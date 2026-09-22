import { render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import type { AwarenessUser, WorkflowVariable } from "@flow/types";

import { WorkflowVariablesTable } from "./WorkflowVariablesTable";

// A reader (or a locked project) must not be able to drag rows into a new
// order. The rest of the row has to survive that: the awareness indicator
// lives on this row variant only, so it must not be lost along the way.

const workflowVariables = [
  { id: "v1", name: "alpha" },
  { id: "v2", name: "beta" },
] as unknown as WorkflowVariable[];

const columns = [
  {
    id: "name",
    header: "Name",
    accessorKey: "name",
    cell: ({ row }: { row: { original: WorkflowVariable } }) =>
      row.original.name,
  },
] as never;

const editingUser = {
  clientId: 3,
  userName: "Someone Else",
  color: "rgb(1, 2, 3)",
} as unknown as AwarenessUser;

const renderTable = (readonly: boolean) =>
  render(
    <WorkflowVariablesTable
      workflowVariables={workflowVariables}
      columns={columns}
      onReorder={vi.fn()}
      readonly={readonly}
      variableEditMap={{ v1: [editingUser] }}
    />,
  );

// dnd-kit marks each draggable row with this, and puts the grab handle in a
// cursor-grab element. Both disappear together when dragging is suppressed.
const sortableRows = (container: HTMLElement) =>
  container.querySelectorAll('[aria-roledescription="sortable"]');
const grabHandles = (container: HTMLElement) =>
  container.querySelectorAll(".cursor-grab");

describe("WorkflowVariablesTable reorder restrictions", () => {
  test("readonly rows offer no way to start a drag", () => {
    const { container } = renderTable(true);

    expect(sortableRows(container)).toHaveLength(0);
    expect(grabHandles(container)).toHaveLength(0);
  });

  test("editable rows are draggable", () => {
    const { container } = renderTable(false);

    // Same props, only readonly differs — so readonly is what removes dragging.
    expect(sortableRows(container)).toHaveLength(workflowVariables.length);
    expect(grabHandles(container)).toHaveLength(workflowVariables.length);
  });

  test("readonly keeps the rows and their awareness indicators", () => {
    const { container } = renderTable(true);

    // Losing reorder must not quietly cost a reader the collaborator markers.
    expect(screen.getByText("alpha")).toBeInTheDocument();
    expect(screen.getByText("beta")).toBeInTheDocument();
    expect(
      container.querySelector('[style*="rgb(1, 2, 3)"]'),
    ).toBeInTheDocument();
  });
});
