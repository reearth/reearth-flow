import {
  columnFilteringFeature,
  columnResizingFeature,
  columnSizingFeature,
  columnVisibilityFeature,
  createFilteredRowModel,
  createSortedRowModel,
  globalFilteringFeature,
  rowPaginationFeature,
  rowSelectionFeature,
  rowSortingFeature,
  tableFeatures,
} from "@tanstack/react-table";
import type {
  ColumnDef,
  FilterFn,
  Row,
  RowData,
  Table,
} from "@tanstack/react-table";

/**
 * The feature set shared by every table in the app.
 *
 * react-table v9 makes features opt-in, and the generics of `ColumnDef`, `Row`
 * and `Table` are all keyed by the feature set. Column definitions built for one
 * feature set are therefore not assignable to a table built with another, and
 * `useDataColumnizer` feeds both `DataTable` and `VirtualizedTable`. One shared
 * set keeps them interchangeable, at the cost of registering a few features a
 * given table may not use.
 *
 * `globalFilteringFeature` and `filteredRowModel` both require
 * `columnFilteringFeature` — see `FeatureSlotPrereqs` in table-core.
 */
export const appTableFeatures = tableFeatures({
  columnFilteringFeature,
  // VirtualizedTable reads `header.getSize()` / `column.getSize()` and sets
  // `columnResizeMode`. `columnResizingFeature` requires `columnSizingFeature`.
  columnResizingFeature,
  columnSizingFeature,
  columnVisibilityFeature,
  globalFilteringFeature,
  rowPaginationFeature,
  rowSelectionFeature,
  rowSortingFeature,
  filteredRowModel: createFilteredRowModel(),
  sortedRowModel: createSortedRowModel(),
  // No `paginatedRowModel` on purpose. Every table here paginates server-side:
  // it reads and writes `pagination` state (hence `rowPaginationFeature`) but
  // renders whatever rows it is handed. Registering the row model would make
  // `getRowModel()` slice to `pageSize` — `_createPaginatedRowModel` slices
  // unconditionally and does *not* honour `manualPagination`; only
  // `table_getPaginatedRowModel` checks that flag before calling it. A table
  // that registered the model without also setting `manualPagination: true`
  // would silently show its first 10 rows, and still typecheck.
});

export type AppTableFeatures = typeof appTableFeatures;

/** `ColumnDef` bound to the app's feature set. */
export type AppColumnDef<TData extends RowData, TValue = unknown> = ColumnDef<
  AppTableFeatures,
  TData,
  TValue
>;

/** `FilterFn` bound to the app's feature set. */
export type AppFilterFn<TData extends RowData> = FilterFn<
  AppTableFeatures,
  TData
>;

/** `Row` bound to the app's feature set. */
export type AppRow<TData extends RowData> = Row<AppTableFeatures, TData>;

/** `Table` bound to the app's feature set. */
export type AppTable<TData extends RowData> = Table<AppTableFeatures, TData>;
