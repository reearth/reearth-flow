import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ProjectFormFields } from "./ProjectFormFields";
import useProjectForm from "./useProjectForm";

vi.mock("@flow/lib/gql", () => ({
  useProject: () => ({
    useCheckProjectAlias: () => ({
      availability: undefined,
      errorMessage: undefined,
    }),
  }),
  useWorkspace: () => ({
    useGetWorkspacePolicy: (workspaceId: string) => ({
      policy: { workspaceId, canCreatePrivateProject: true },
    }),
  }),
}));

const Harness: React.FC = () => {
  const form = useProjectForm({ workspaceId: "ws" });
  return <ProjectFormFields form={form} />;
};

describe("ProjectFormFields", () => {
  it("fills the alias as the name is typed", () => {
    render(<Harness />);
    const name = screen.getByLabelText("Project Name");
    const alias = screen.getByLabelText("Alias (optional)");

    fireEvent.change(name, { target: { value: "Tokyo roads" } });
    expect(alias).toHaveValue("Tokyo-roads");
    fireEvent.change(name, { target: { value: "Tokyoroads" } });
    expect(alias).toHaveValue("Tokyoroads");
  });

  it("turns a space typed into the alias into a dash", () => {
    render(<Harness />);
    const alias = screen.getByLabelText("Alias (optional)");

    fireEvent.change(alias, { target: { value: "my alias" } });
    expect(alias).toHaveValue("my-alias");
  });

  it("picks a visibility from anywhere on its card", () => {
    render(<Harness />);
    const publicRadio = screen.getByRole("radio", { name: /^Public/ });
    const privateRadio = screen.getByRole("radio", { name: /^Private/ });
    expect(publicRadio).toBeChecked();

    fireEvent.click(
      screen.getByText("Only members of the workspace can see it."),
    );
    expect(privateRadio).toBeChecked();
    expect(publicRadio).not.toBeChecked();
  });
});
