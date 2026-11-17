import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import { ArchiveNameDialog } from "./ArchiveNameDialog";

const renderDialog = (defaultName = "Tokyo buildings", isOpen = true) => {
  const onConfirm = vi.fn();
  const onCancel = vi.fn();
  const view = render(
    <ArchiveNameDialog
      isOpen={isOpen}
      defaultName={defaultName}
      fileCount={3}
      onConfirm={onConfirm}
      onCancel={onCancel}
    />,
  );
  const rerender = (open: boolean, name = defaultName) =>
    view.rerender(
      <ArchiveNameDialog
        isOpen={open}
        defaultName={name}
        fileCount={3}
        onConfirm={onConfirm}
        onCancel={onCancel}
      />,
    );
  return { onConfirm, onCancel, rerender };
};

const nameField = () => screen.getByLabelText("File name") as HTMLInputElement;
const download = () => screen.getByRole("button", { name: "Download" });

describe("ArchiveNameDialog", () => {
  test("suggests the name in a form that needs no quoting", () => {
    renderDialog("Shapefile to CityGML (PLATEAU bldg LOD1) (import)");
    expect(nameField().value).toBe(
      "Shapefile_to_CityGML_PLATEAU_bldg_LOD1_import",
    );
  });

  test("keeps spaces the user types", () => {
    const { onConfirm } = renderDialog();

    fireEvent.change(nameField(), { target: { value: "my results" } });
    fireEvent.click(download());

    expect(onConfirm).toHaveBeenCalledWith("my results.zip");
  });

  test("downloads under the name typed, with the extension added", () => {
    const { onConfirm } = renderDialog();

    fireEvent.change(nameField(), { target: { value: "results" } });
    fireEvent.click(download());

    expect(onConfirm).toHaveBeenCalledWith("results.zip");
  });

  test("makes a typed name safe to save under", () => {
    const { onConfirm } = renderDialog();

    fireEvent.change(nameField(), { target: { value: " udx/bldg.zip " } });
    fireEvent.click(download());

    expect(onConfirm).toHaveBeenCalledWith("udx_bldg.zip");
  });

  test("cannot download without a name", () => {
    const { onConfirm } = renderDialog();

    fireEvent.change(nameField(), { target: { value: "   " } });

    expect(download()).toBeDisabled();
    fireEvent.submit(nameField().form as HTMLFormElement);
    expect(onConfirm).not.toHaveBeenCalled();
  });

  test("starts each opening from the suggested name again", () => {
    const { rerender } = renderDialog("first");

    fireEvent.change(nameField(), { target: { value: "edited" } });
    rerender(false);
    rerender(true, "second");

    expect(nameField().value).toBe("second");
  });

  test("cancels", () => {
    const { onCancel, onConfirm } = renderDialog();

    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

    expect(onCancel).toHaveBeenCalled();
    expect(onConfirm).not.toHaveBeenCalled();
  });
});
