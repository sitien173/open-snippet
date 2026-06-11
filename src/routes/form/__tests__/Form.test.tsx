import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi, describe, test, expect, beforeEach } from "vitest";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { Text } from "../fields/Text";
import { Textarea } from "../fields/Textarea";
import { Choice } from "../fields/Choice";
import { NumberField } from "../fields/NumberField";
import { FormRoute } from "../index";
import { VarDecl, Snippet } from "../../../lib/snippets";

// Mock Tauri window APIs
const mockClose = vi.fn();
const mockSetSize = vi.fn();

vi.mock("@tauri-apps/api/window", () => {
  return {
    getCurrentWindow: () => ({
      close: mockClose,
      setSize: mockSetSize,
    }),
    LogicalSize: class {
      width: number;
      height: number;
      constructor(width: number, height: number) {
        this.width = width;
        this.height = height;
      }
    },
  };
});



describe("Form Fields", () => {
  test("Text component renders input with label, value, required, autoFocus, and calls onChange", async () => {
    const user = userEvent.setup();
    const decl: VarDecl = { name: "username", kind: "text", label: "User Name", required: true };
    const onChange = vi.fn();

    render(<Text decl={decl} value="initial" onChange={onChange} autoFocus={true} />);

    const input = screen.getByLabelText("User Name");
    expect(input).toBeInTheDocument();
    expect(input).toHaveValue("initial");
    expect(input).toBeRequired();
    expect(input).toHaveFocus();

    await user.type(input, "s");
    expect(onChange).toHaveBeenCalledWith("initials");
  });

  test("Textarea component renders textarea with label, value, and calls onChange", async () => {
    const user = userEvent.setup();
    const decl: VarDecl = { name: "bio", kind: "textarea", label: "Biography" };
    const onChange = vi.fn();

    render(<Textarea decl={decl} value="hello" onChange={onChange} />);

    const textarea = screen.getByLabelText("Biography");
    expect(textarea).toBeInTheDocument();
    expect(textarea).toHaveValue("hello");

    await user.type(textarea, "!");
    expect(onChange).toHaveBeenCalledWith("hello!");
  });

  test("Choice component renders select with options, label, and calls onChange", async () => {
    const user = userEvent.setup();
    const decl: VarDecl = { name: "color", kind: "choice", label: "Select Color", options: ["Red", "Blue"] };
    const onChange = vi.fn();

    render(<Choice decl={decl} value="Red" onChange={onChange} />);

    const select = screen.getByLabelText("Select Color");
    expect(select).toBeInTheDocument();
    expect(select).toHaveValue("Red");

    await user.selectOptions(select, "Blue");
    expect(onChange).toHaveBeenCalledWith("Blue");
  });

  test("Choice component renders inline error when options list is empty", () => {
    const decl: VarDecl = { name: "color", kind: "choice", label: "Select Color", options: [] };
    const onChange = vi.fn();

    render(<Choice decl={decl} value="" onChange={onChange} />);

    expect(screen.getByText(/no options defined/i)).toBeInTheDocument();
  });

  test("NumberField component renders input[type=number] and stores string value", async () => {
    const user = userEvent.setup();
    const decl: VarDecl = { name: "age", kind: "number", label: "Age", required: true };
    const onChange = vi.fn();

    render(<NumberField decl={decl} value="25" onChange={onChange} />);

    const input = screen.getByLabelText("Age");
    expect(input).toBeInTheDocument();
    expect(input).toHaveAttribute("type", "number");
    expect(input).toHaveValue(25);
    expect(input).toBeRequired();

    await user.type(input, "6");
    expect(onChange).toHaveBeenCalledWith("256");
  });
});

describe("Form Container Route", () => {
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
      { name: "ignored_var", kind: "datetime", label: "Ignored" },
    ],
    source_file: "snippets.yaml",
    file_relative: "snippets.yaml",
  };

  let mockInvoke: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    delete window.__OPENMACRO_MOCK_INVOKE;
    mockClose.mockClear();
    mockSetSize.mockClear();

    mockInvoke = vi.fn().mockImplementation((cmd, _args) => {
      if (cmd === "list_snippets") {
        return Promise.resolve([mockSnippet]);
      }
      if (cmd === "form_submit" || cmd === "form_cancel") {
        return Promise.resolve(null);
      }
      return Promise.reject(new Error(`Unhandled mock command: ${cmd}`));
    });
    window.__OPENMACRO_MOCK_INVOKE = mockInvoke;
  });

  const renderFormRoute = () => {
    return render(
      <MemoryRouter initialEntries={["/form/greet-snippet-id"]}>
        <Routes>
          <Route path="/form/:snippetId" element={<FormRoute />} />
        </Routes>
      </MemoryRouter>
    );
  };

  test("loads snippet on mount, renders form fields, skips non-form kinds, autofocusses first field", async () => {
    renderFormRoute();

    // Wait for list_snippets to load
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("list_snippets");
    });

    // Check rendered fields
    expect(await screen.findByLabelText("First Name")).toBeInTheDocument();
    expect(screen.getByLabelText("Notes")).toBeInTheDocument();
    expect(screen.getByLabelText("Role")).toBeInTheDocument();
    expect(screen.getByLabelText("Age")).toBeInTheDocument();
    expect(screen.queryByLabelText("Ignored")).not.toBeInTheDocument();

    // Check autofocus on first field ("First Name")
    expect(screen.getByLabelText("First Name")).toHaveFocus();
  });

  test("Enter key in input submits form if valid", async () => {
    const user = userEvent.setup();
    renderFormRoute();

    const nameInput = await screen.findByLabelText("First Name");
    await user.type(nameInput, "John");

    // Press Enter in First Name text field
    await user.type(nameInput, "{Enter}");

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("form_submit", {
        snippetId: "greet-snippet-id",
        values: {
          name: "John",
          notes: "",
          role: "Admin", // default to first option
          age: "",
        },
      });
      expect(mockClose).toHaveBeenCalled();
    });
  });

  test("Enter key in textarea does NOT submit form", async () => {
    const user = userEvent.setup();
    renderFormRoute();

    const nameInput = await screen.findByLabelText("First Name");
    await user.type(nameInput, "John");

    const notesTextarea = screen.getByLabelText("Notes");
    await user.type(notesTextarea, "Line 1{Enter}Line 2");

    // Should not have submitted yet
    expect(mockInvoke).not.toHaveBeenCalledWith("form_submit", expect.any(Object));

    // Submit using submit button
    const submitBtn = screen.getByRole("button", { name: /submit/i });
    await user.click(submitBtn);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("form_submit", {
        snippetId: "greet-snippet-id",
        values: {
          name: "John",
          notes: "Line 1\nLine 2",
          role: "Admin",
          age: "",
        },
      });
      expect(mockClose).toHaveBeenCalled();
    });
  });

  test("Escape key cancels form", async () => {
    const user = userEvent.setup();
    renderFormRoute();

    // Wait for form to load
    await screen.findByLabelText("First Name");

    // Press Esc on active element
    await user.keyboard("{Escape}");

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("form_cancel", {
        snippetId: "greet-snippet-id",
      });
      expect(mockClose).toHaveBeenCalled();
    });
  });

  test("required field validation blocks submit, shows error, focuses first invalid field", async () => {
    const user = userEvent.setup();
    renderFormRoute();

    // Wait for form to load
    const nameInput = await screen.findByLabelText("First Name");

    // Try to submit with name empty
    const submitBtn = screen.getByRole("button", { name: /submit/i });
    await user.click(submitBtn);

    // Should show validation error
    const errorMsg = await screen.findByText(/first name is required/i);
    expect(errorMsg).toBeInTheDocument();

    // Verify submit IPC was not called
    expect(mockInvoke).not.toHaveBeenCalledWith("form_submit", expect.any(Object));

    // Name input should be focused
    expect(nameInput).toHaveFocus();

    // Type something to make it valid
    await user.type(nameInput, "Alice");

    // Submit again
    await user.click(submitBtn);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("form_submit", {
        snippetId: "greet-snippet-id",
        values: {
          name: "Alice",
          notes: "",
          role: "Admin",
          age: "",
        },
      });
      expect(mockClose).toHaveBeenCalled();
    });
  });

  test("Cancel button cancels form", async () => {
    const user = userEvent.setup();
    renderFormRoute();

    await screen.findByLabelText("First Name");

    const cancelBtn = screen.getByRole("button", { name: /cancel/i });
    await user.click(cancelBtn);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("form_cancel", {
        snippetId: "greet-snippet-id",
      });
      expect(mockClose).toHaveBeenCalled();
    });
  });
});
