import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, test } from "vitest";

import type { AppColumnDef } from "@flow/lib/table/features";

import { DataTable } from ".";

type Row = { name: string; size: number };

const data: Row[] = [
  { name: "beta", size: 2 },
  { name: "alpha", size: 3 },
  { name: "gamma", size: 1 },
];

// Sorting, row selection and column visibility are opt-in features in
// react-table v9, and a feature registered without its row model still
// typechecks while quietly doing nothing. These assert behaviour, not markup.
const columns: AppColumnDef<Row>[] = [
  {
    accessorKey: "name",
    header: ({ column }) => (
      <button onClick={() => column.toggleSorting()}>Name</button>
    ),
  },
  { accessorKey: "size", header: "Size" },
];

const bodyRowText = () =>
  screen
    .getAllByRole("row")
    .slice(1) // drop the header row
    .map((row) => within(row).getAllByRole("cell")[0].textContent);

describe("DataTable", () => {
  test("renders a row per datum", () => {
    render(<DataTable columns={columns} data={data} />);

    expect(bodyRowText()).toEqual(["beta", "alpha", "gamma"]);
  });

  // v8 registered the paginated row model only when `enablePagination` was set.
  // This pins the un-paginated case so that intent survives the v9 port.
  //
  // Note this test alone does NOT prove the shared feature set is safe:
  // DataTable also sets `manualPagination: true`, which short-circuits the
  // slice, so it passes either way. The tables that lack that flag are the
  // exposed ones — see LogsTable's test for the case with teeth.
  test("renders every row when pagination is disabled", () => {
    const many = Array.from({ length: 25 }, (_, i) => ({
      name: `row-${i}`,
      size: i,
    }));

    render(<DataTable columns={columns} data={many} />);

    expect(bodyRowText()).toHaveLength(25);
  });

  test("sorts rows when a header toggles sorting", async () => {
    const user = userEvent.setup();
    render(<DataTable columns={columns} data={data} />);

    await user.click(screen.getByRole("button", { name: "Name" }));
    expect(bodyRowText()).toEqual(["alpha", "beta", "gamma"]);

    await user.click(screen.getByRole("button", { name: "Name" }));
    expect(bodyRowText()).toEqual(["gamma", "beta", "alpha"]);
  });

  test("filters rows from the search input", async () => {
    const user = userEvent.setup();
    render(<DataTable columns={columns} data={data} showFiltering />);

    await user.type(screen.getByPlaceholderText("Search..."), "alp");

    expect(bodyRowText()).toEqual(["alpha"]);
  });

  test("marks a row selected when clicked", async () => {
    const user = userEvent.setup();
    render(<DataTable columns={columns} data={data} />);

    const row = screen.getAllByRole("row")[1];
    expect(row).not.toHaveAttribute("data-state", "selected");

    await user.click(row);

    expect(row).toHaveAttribute("data-state", "selected");
  });
});
