import { render, screen } from "@testing-library/react";
import axe from "axe-core";
import { vi, describe, test, expect, beforeEach } from "vitest";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { FormRoute } from "../index";
import { Snippet } from "../../../lib/snippets";

const mockSnippet: Snippet = {
  id: "greet-snippet-id",
  trigger: "greet",
  effective_trigger: "greet",
  trigger_literal: false,
  replace: "hello {{name}}",
  vars: [
    { name: "name", kind: "text", label: "First Name", required: true },
    { name: "notes", kind: "textarea", label: "Notes" },
    { name: "role", kind: "choice", label: "Role", options: ["Admin", "User"] },
    { name: "age", kind: "number", label: "Age" },
  ],
  source_file: "snippets.yaml",
  file_relative: "snippets.yaml",
};

describe("FormRoute A11y", () => {
  beforeEach(() => {
    delete window.__OPENMACRO_MOCK_INVOKE;
  });

  test("should have no accessibility violations", async () => {
    window.__OPENMACRO_MOCK_INVOKE = vi.fn().mockImplementation((cmd) => {
      if (cmd === "list_snippets") {
        return Promise.resolve([mockSnippet]);
      }
      return Promise.resolve(null);
    });

    const { container } = render(
      <MemoryRouter initialEntries={["/form/greet-snippet-id"]}>
        <Routes>
          <Route path="/form/:snippetId" element={<FormRoute />} />
        </Routes>
      </MemoryRouter>
    );

    // Wait for the form to finish loading and render fields
    await screen.findByLabelText("First Name");

    const results = await axe.run(container);
    expect(results.violations).toEqual([]);
  });
});
