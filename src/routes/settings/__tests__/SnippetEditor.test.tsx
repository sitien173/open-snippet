import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";
import { SnippetEditor } from "../SnippetEditor";
import { Snippet } from "../../../lib/snippets";

const mockSnippet: Snippet = {
  id: "snippets/default.yaml::greet",
  trigger: "greet",
  effective_trigger: "greet",
  trigger_literal: false,
  replace: "Hello $|$",
  vars: [],
  source_file: "F:/projects_new/textblaze/snippets/default.yaml",
  file_relative: "snippets/default.yaml",
};

const otherSnippets: Snippet[] = [
  {
    id: "snippets/default.yaml::other",
    trigger: "other",
    effective_trigger: "other",
    trigger_literal: false,
    replace: "Other text",
    vars: [],
    source_file: "F:/projects_new/textblaze/snippets/default.yaml",
    file_relative: "snippets/default.yaml",
  },
  {
    id: "snippets/other-file.yaml::greet",
    trigger: "greet",
    effective_trigger: "greet",
    trigger_literal: false,
    replace: "Greet in other file",
    vars: [],
    source_file: "F:/projects_new/textblaze/snippets/other-file.yaml",
    file_relative: "snippets/other-file.yaml",
  }
];

describe("SnippetEditor", () => {
  beforeEach(() => {
    delete window.__OPENMACRO_MOCK_INVOKE;
  });

  test("renders editor with a fixture snippet and typing into trigger updates state", async () => {
    const user = userEvent.setup();
    render(<SnippetEditor snippet={mockSnippet} allSnippets={otherSnippets} />);

    const triggerInput = screen.getByLabelText(/trigger/i);
    expect(triggerInput).toHaveValue("greet");

    await user.clear(triggerInput);
    await user.type(triggerInput, "newgreet");
    expect(triggerInput).toHaveValue("newgreet");
  });

  test("empty trigger shows inline error and disables save", async () => {
    const user = userEvent.setup();
    render(<SnippetEditor snippet={mockSnippet} allSnippets={otherSnippets} />);

    const triggerInput = screen.getByLabelText(/trigger/i);
    const saveButton = screen.getByRole("button", { name: /save/i });

    expect(saveButton).not.toBeDisabled();

    await user.clear(triggerInput);
    // Trigger is empty now
    expect(screen.getByText(/trigger cannot be empty/i)).toBeInTheDocument();
    expect(saveButton).toBeDisabled();
  });

  test("trigger > 32 chars shows inline error and disables save", async () => {
    const user = userEvent.setup();
    render(<SnippetEditor snippet={mockSnippet} allSnippets={otherSnippets} />);

    const triggerInput = screen.getByLabelText(/trigger/i);
    const saveButton = screen.getByRole("button", { name: /save/i });

    await user.clear(triggerInput);
    await user.type(triggerInput, "a".repeat(33));

    expect(screen.getByText(/trigger cannot exceed 32 characters/i)).toBeInTheDocument();
    expect(saveButton).toBeDisabled();
  });

  test("trigger collision in the same file shows inline error and disables save", async () => {
    const user = userEvent.setup();
    render(<SnippetEditor snippet={mockSnippet} allSnippets={otherSnippets} />);

    const triggerInput = screen.getByLabelText(/trigger/i);
    const saveButton = screen.getByRole("button", { name: /save/i });

    await user.clear(triggerInput);
    await user.type(triggerInput, "other"); // matches default.yaml::other (same file)

    expect(screen.getByText(/trigger collision/i)).toBeInTheDocument();
    expect(saveButton).toBeDisabled();
  });

  test("trigger collision in another file does NOT show error and does NOT disable save", async () => {
    const user = userEvent.setup();
    render(<SnippetEditor snippet={mockSnippet} allSnippets={otherSnippets} />);

    const triggerInput = screen.getByLabelText(/trigger/i);
    const saveButton = screen.getByRole("button", { name: /save/i });

    await user.clear(triggerInput);
    await user.type(triggerInput, "greet"); // matches other-file.yaml::greet (different file)

    expect(screen.queryByText(/trigger collision/i)).not.toBeInTheDocument();
    expect(saveButton).not.toBeDisabled();
  });

  test("vars panel allows adding, editing (choice options), and removing variables", async () => {
    const mockInvoke = vi.fn().mockResolvedValue(null);
    window.__OPENMACRO_MOCK_INVOKE = mockInvoke;

    const onSave = vi.fn();
    render(<SnippetEditor snippet={mockSnippet} allSnippets={otherSnippets} onSave={onSave} />);

    const addVarBtn = screen.getByRole("button", { name: /add variable/i });
    fireEvent.click(addVarBtn);

    // Should render a new var row with name input
    const varNameInputs = screen.getAllByPlaceholderText(/var name/i);
    expect(varNameInputs.length).toBe(1);

    fireEvent.change(varNameInputs[0], { target: { value: "myvar" } });

    const kindSelects = screen.getAllByRole("combobox", { name: /kind/i });
    fireEvent.change(kindSelects[0], { target: { value: "choice" } });

    // Filling choice options
    const optionsInputs = screen.getAllByPlaceholderText(/comma-separated options/i);
    fireEvent.change(optionsInputs[0], { target: { value: "foo,bar" } });

    // Let's add another variable
    fireEvent.click(addVarBtn);
    const varNameInputsUpdated = screen.getAllByPlaceholderText(/var name/i);
    expect(varNameInputsUpdated.length).toBe(2);

    fireEvent.change(varNameInputsUpdated[1], { target: { value: "todelete" } });

    // Remove the second variable
    const removeBtns = screen.getAllByRole("button", { name: /remove/i });
    fireEvent.click(removeBtns[1]);

    expect(screen.getAllByPlaceholderText(/var name/i).length).toBe(1);

    // Save
    const saveButton = screen.getByRole("button", { name: /save/i });
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("save_snippet", expect.objectContaining({
        payload: expect.objectContaining({
          trigger: "greet",
          vars: [
            expect.objectContaining({
              name: "myvar",
              kind: "choice",
              options: ["foo", "bar"]
            })
          ]
        })
      }));
      expect(mockInvoke).toHaveBeenCalledWith("reload_snippets");
      expect(onSave).toHaveBeenCalled();
    });
  });

  test("new snippets save into the runtime default snippet file", async () => {
    const mockInvoke = vi.fn().mockResolvedValue(null);
    window.__OPENMACRO_MOCK_INVOKE = mockInvoke;

    render(<SnippetEditor allSnippets={otherSnippets} />);

    const triggerInput = screen.getByLabelText(/trigger/i);
    const replaceInput = screen.getByLabelText(/replacement/i);
    const saveButton = screen.getByRole("button", { name: /save snippet/i });

    fireEvent.change(triggerInput, { target: { value: "brand-new" } });
    fireEvent.change(replaceInput, { target: { value: "New text" } });
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("save_snippet", expect.objectContaining({
        payload: expect.objectContaining({
          source_file: "F:/projects_new/textblaze/snippets/default.yaml",
          original_trigger: null,
          trigger: "brand-new",
          replace: "New text",
        })
      }));
      expect(mockInvoke).toHaveBeenCalledWith("reload_snippets");
    });
  });
});
