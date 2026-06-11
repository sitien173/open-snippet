import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";
import { SnippetEditor } from "../SnippetEditor";
import { Snippet } from "../../../lib/snippets";
import { Settings } from "../index";
import { MemoryRouter } from "react-router-dom";

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

  test("shows read-only effective trigger hint when trigger_literal is false", async () => {
    render(<SnippetEditor snippet={{ ...mockSnippet, trigger: "greet", effective_trigger: ":greet", trigger_literal: false }} allSnippets={otherSnippets} />);
    
    // Should show the effective trigger hint
    expect(screen.getByText(/:greet/)).toBeInTheDocument();
    expect(screen.getByText(/effective trigger/i)).toBeInTheDocument();
  });

  test("hides effective trigger hint when trigger_literal is true", async () => {
    render(<SnippetEditor snippet={{ ...mockSnippet, trigger: "greet", effective_trigger: "greet", trigger_literal: true }} allSnippets={otherSnippets} />);
    
    // Should NOT show the effective trigger hint
    expect(screen.queryByText(/effective trigger/i)).not.toBeInTheDocument();
  });

  test("toggling trigger_literal to true hides hint and includes it in save payload", async () => {
    const mockInvoke = vi.fn().mockResolvedValue(null);
    window.__OPENMACRO_MOCK_INVOKE = mockInvoke;

    const user = userEvent.setup();
    render(<SnippetEditor snippet={{ ...mockSnippet, trigger: "greet", effective_trigger: ":greet", trigger_literal: false }} allSnippets={otherSnippets} />);
    
    expect(screen.getByText(/:greet/)).toBeInTheDocument();

    const literalToggle = screen.getByLabelText(/exact match/i); // assuming label "Exact match" or "Literal"
    await user.click(literalToggle);

    expect(screen.queryByText(/:greet/)).not.toBeInTheDocument();

    const saveButton = screen.getByRole("button", { name: /save/i });
    await user.click(saveButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("save_snippet", expect.objectContaining({
        payload: expect.objectContaining({
          trigger: "greet",
          trigger_literal: true,
          original_trigger_literal: false
        })
      }));
    });
  });

  test("uses the triggerPrefix prop for the effective trigger hint", () => {
    render(
      <SnippetEditor
        snippet={{ ...mockSnippet, trigger: "greet", trigger_literal: false }}
        allSnippets={otherSnippets}
        triggerPrefix="!!"
      />
    );
    expect(screen.getByText(/!!greet/)).toBeInTheDocument();
  });

  test("editing the trigger input updates the effective trigger hint live", async () => {
    const user = userEvent.setup();
    render(
      <SnippetEditor
        snippet={{ ...mockSnippet, trigger: "greet", trigger_literal: false }}
        allSnippets={otherSnippets}
        triggerPrefix=":"
      />
    );
    expect(screen.getByText(/:greet/)).toBeInTheDocument();

    const triggerInput = screen.getByLabelText(/trigger/i);
    await user.clear(triggerInput);
    await user.type(triggerInput, "hello");

    expect(screen.getByText(/:hello/)).toBeInTheDocument();
  });

  test("a trigger already starting with the prefix is not double-prepended in the hint", async () => {
    const user = userEvent.setup();
    render(
      <SnippetEditor
        snippet={{ ...mockSnippet, trigger: ":greet", trigger_literal: false }}
        allSnippets={otherSnippets}
        triggerPrefix=":"
      />
    );
    expect(screen.getByText(/:greet/)).toBeInTheDocument();
    expect(screen.queryByText(/::greet/)).not.toBeInTheDocument();

    const triggerInput = screen.getByLabelText(/trigger/i);
    await user.clear(triggerInput);
    await user.type(triggerInput, ":hello");

    expect(screen.getByText(/:hello/)).toBeInTheDocument();
    expect(screen.queryByText(/::hello/)).not.toBeInTheDocument();
  });

  test("after the prefix setting is saved, opening the editor uses the new prefix", async () => {
    const mockInvoke = vi.fn().mockImplementation(async (cmd: string) => {
      if (cmd === "get_prefs") {
        return { paused: false, autostart: false, max_expansion_len: 1000, shell_consent: false };
      }
      if (cmd === "get_store_settings") {
        return { trigger_prefix: ":" };
      }
      if (cmd === "list_snippets") {
        return [mockSnippet];
      }
      if (cmd === "set_store_settings") {
        return null;
      }
      return null;
    });
    window.__OPENMACRO_MOCK_INVOKE = mockInvoke;

    const user = userEvent.setup();
    render(
      <MemoryRouter>
        <Settings />
      </MemoryRouter>
    );

    // Wait for settings to load and render snippets
    await waitFor(() => {
      expect(screen.getByText(/greet/i)).toBeInTheDocument();
    });

    // Go to preferences tab
    const prefTabButton = screen.getByRole("button", { name: /preferences/i });
    await user.click(prefTabButton);

    // Edit prefix and save
    const prefixInput = screen.getByLabelText(/trigger prefix/i);
    expect(prefixInput).toHaveValue(":");

    await user.clear(prefixInput);
    await user.type(prefixInput, "!!");
    
    const savePrefixBtn = screen.getByRole("button", { name: /save prefix/i });
    await user.click(savePrefixBtn);

    // Wait for set_store_settings call
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("set_store_settings", { settings: { trigger_prefix: "!!", expand_mode: "manual" } });
    });

    // Switch back to snippets tab
    const snippetsTabButton = screen.getByRole("button", { name: /snippets/i });
    await user.click(snippetsTabButton);

    // Open editor by clicking edit on "greet"
    const editButton = screen.getByRole("button", { name: /edit/i });
    await user.click(editButton);

    // Verify that the editor uses the new prefix (!!greet)
    await waitFor(() => {
      expect(screen.getByText(/!!greet/)).toBeInTheDocument();
    });
  });
});
