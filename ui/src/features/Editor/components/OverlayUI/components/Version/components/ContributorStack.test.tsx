import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, test } from "vitest";

import ContributorStack from "./ContributorStack";

const user = (id: string, name: string) => ({ id, name });

// Avatars render their initial until a photo loads, which never happens in
// jsdom, so initials stand in for avatars here.
const initials = (container: HTMLElement) =>
  Array.from(container.querySelectorAll("span"))
    .filter((el) => el.childElementCount === 0)
    .map((el) => el.textContent ?? "")
    .filter((text) => /^[A-Z]$/.test(text));

describe("ContributorStack", () => {
  test("renders nothing for a version without attribution", () => {
    const { container } = render(
      <ContributorStack contributors={[]} contributorCount={0} />,
    );

    expect(container).toBeEmptyDOMElement();
  });

  test("puts the last editor first", () => {
    const { container } = render(
      <ContributorStack
        contributors={[user("a", "Aiko"), user("b", "Ben"), user("c", "Chi")]}
        contributorCount={3}
        lastEditorId="c"
      />,
    );

    expect(initials(container)).toEqual(["C", "A", "B"]);
    expect(screen.queryByText(/^\+/)).not.toBeInTheDocument();
  });

  test("counts the contributors that do not fit in +N", () => {
    const { container } = render(
      <ContributorStack
        contributors={[
          user("a", "Aiko"),
          user("b", "Ben"),
          user("c", "Chi"),
          user("d", "Dan"),
          user("e", "Eri"),
        ]}
        contributorCount={5}
      />,
    );

    expect(initials(container)).toHaveLength(3);
    expect(screen.getByText("+2")).toBeInTheDocument();
  });

  test("counts contributors missing from the list in +N", () => {
    // The stored list is capped and deleted users are left out, so the count
    // can exceed the list.
    const { container } = render(
      <ContributorStack
        contributors={[user("a", "Aiko"), user("b", "Ben")]}
        contributorCount={5}
      />,
    );

    expect(initials(container)).toHaveLength(2);
    expect(screen.getByText("+3")).toBeInTheDocument();
  });

  test("shows +N alone when no contributor resolves", () => {
    const { container } = render(
      <ContributorStack contributors={[]} contributorCount={2} />,
    );

    expect(initials(container)).toHaveLength(0);
    expect(screen.getByText("+2")).toBeInTheDocument();
  });

  test("names the contributors on hover, marking the last editor", async () => {
    render(
      <ContributorStack
        contributors={[user("a", "Aiko"), user("b", "Ben")]}
        contributorCount={4}
        lastEditorId="b"
      />,
    );

    await userEvent.hover(screen.getByLabelText("Contributors"));

    expect(await screen.findByText("Ben (last edit)")).toBeInTheDocument();
    expect(screen.getByText("Aiko")).toBeInTheDocument();
    expect(screen.getByText("2 more")).toBeInTheDocument();
  });
});
