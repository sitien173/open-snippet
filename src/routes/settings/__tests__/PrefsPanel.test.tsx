import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";
import { PrefsPanel } from "../PrefsPanel";

describe("PrefsPanel", () => {
  beforeEach(() => {
    delete window.__OPENMACRO_MOCK_INVOKE;
  });

  test("renders trigger prefix, shows preview, and persists changes", async () => {
    const mockInvoke = vi.fn().mockImplementation(async (cmd: string) => {
      if (cmd === "get_prefs") {
        return { paused: false, autostart: false, max_expansion_len: 1000, shell_consent: false };
      }
      if (cmd === "get_store_settings") {
        return { trigger_prefix: ":" };
      }
      if (cmd === "list_snippets") {
        return [];
      }
      if (cmd === "set_store_settings") {
        return null;
      }
      return null;
    });
    window.__OPENMACRO_MOCK_INVOKE = mockInvoke;

    const user = userEvent.setup();
    render(<PrefsPanel />);

    // Wait for load
    await waitFor(() => {
      expect(screen.queryByText(/Loading/i)).not.toBeInTheDocument();
    });

    const prefixInput = screen.getByLabelText(/trigger prefix/i);
    expect(prefixInput).toHaveValue(":");

    // Check preview
    expect(screen.getByText(/:email/i)).toBeInTheDocument();

    await user.clear(prefixInput);
    await user.type(prefixInput, "!!");

    expect(prefixInput).toHaveValue("!!");
    expect(screen.getByText(/!!email/i)).toBeInTheDocument();

    // blur to save (or maybe a button?)
    // The prompt says "Load current settings, persist edits, show save/error state, and preview a bare trigger like email using the current prefix. Let backend validation be the source of truth"
    // I'll assume we use a save button or onBlur for the prefix. I will look for a save button for prefix.
    const saveButton = screen.getByRole("button", { name: /save prefix/i });
    await user.click(saveButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("set_store_settings", { settings: { trigger_prefix: "!!" } });
    });
  });

  test("invalid prefix shows backend error", async () => {
    const mockInvoke = vi.fn().mockImplementation(async (cmd: string) => {
      if (cmd === "get_prefs") {
        return { paused: false, autostart: false, max_expansion_len: 1000, shell_consent: false };
      }
      if (cmd === "get_store_settings") {
        return { trigger_prefix: ":" };
      }
      if (cmd === "list_snippets") {
        return [];
      }
      if (cmd === "set_store_settings") {
        return Promise.reject("Invalid prefix format from backend");
      }
      return null;
    });
    window.__OPENMACRO_MOCK_INVOKE = mockInvoke;

    const user = userEvent.setup();
    render(<PrefsPanel />);

    // Wait for load
    await waitFor(() => {
      expect(screen.queryByText(/Loading/i)).not.toBeInTheDocument();
    });

    const prefixInput = screen.getByLabelText(/trigger prefix/i);
    const saveButton = screen.getByRole("button", { name: /save prefix/i });

    await user.clear(prefixInput);
    await user.type(prefixInput, "123");
    await user.click(saveButton);

    await waitFor(() => {
      expect(screen.getByText(/Invalid prefix format from backend/i)).toBeInTheDocument();
    });
  });
});
