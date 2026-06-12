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
      expect(mockInvoke).toHaveBeenCalledWith("set_store_settings", { settings: { trigger_prefix: "!!", expand_mode: "manual" } });
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

  test("fresh install (expand_mode present, expand_mode_missing is false) shows no migration notice and renders correct radio state", async () => {
    const mockInvoke = vi.fn().mockImplementation(async (cmd: string) => {
      if (cmd === "get_prefs") {
        return { paused: false, autostart: false, max_expansion_len: 1000, shell_consent: false };
      }
      if (cmd === "get_store_settings") {
        return { trigger_prefix: ":", expand_mode: "manual", expand_mode_missing: false };
      }
      if (cmd === "list_snippets") {
        return [];
      }
      return null;
    });
    window.__OPENMACRO_MOCK_INVOKE = mockInvoke;

    render(<PrefsPanel />);

    await waitFor(() => {
      expect(screen.queryByText(/Loading/i)).not.toBeInTheDocument();
    });

    // Verify migration notice is NOT shown
    expect(screen.queryByText(/Snippets now expand on Tab\/Enter/i)).not.toBeInTheDocument();

    // Verify radio buttons
    const manualRadio = screen.getByLabelText(/Manual/i) as HTMLInputElement;
    const autoRadio = screen.getByLabelText(/Auto/i) as HTMLInputElement;

    expect(manualRadio).toBeInTheDocument();
    expect(manualRadio.checked).toBe(true);
    expect(autoRadio).toBeInTheDocument();
    expect(autoRadio.checked).toBe(false);
  });

  test("saving expand_mode persists selection and preserves trigger_prefix", async () => {
    const mockInvoke = vi.fn().mockImplementation(async (cmd: string) => {
      if (cmd === "get_prefs") {
        return { paused: false, autostart: false, max_expansion_len: 1000, shell_consent: false };
      }
      if (cmd === "get_store_settings") {
        return { trigger_prefix: ";;", expand_mode: "manual", expand_mode_missing: false };
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

    await waitFor(() => {
      expect(screen.queryByText(/Loading/i)).not.toBeInTheDocument();
    });

    const autoRadio = screen.getByLabelText(/Auto/i);
    await user.click(autoRadio);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("set_store_settings", {
        settings: { trigger_prefix: ";;", expand_mode: "auto" }
      });
    });
  });

  test("saving trigger_prefix preserves expand_mode", async () => {
    const mockInvoke = vi.fn().mockImplementation(async (cmd: string) => {
      if (cmd === "get_prefs") {
        return { paused: false, autostart: false, max_expansion_len: 1000, shell_consent: false };
      }
      if (cmd === "get_store_settings") {
        return { trigger_prefix: ":", expand_mode: "auto", expand_mode_missing: false };
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

    await waitFor(() => {
      expect(screen.queryByText(/Loading/i)).not.toBeInTheDocument();
    });

    const prefixInput = screen.getByLabelText(/trigger prefix/i);
    const saveButton = screen.getByRole("button", { name: /save prefix/i });

    await user.clear(prefixInput);
    await user.type(prefixInput, "!!");
    await user.click(saveButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("set_store_settings", {
        settings: { trigger_prefix: "!!", expand_mode: "auto" }
      });
    });
  });

  test("upgrade (expand_mode_missing is true) shows migration notice, and dismiss persists setting to manual", async () => {
    const mockInvoke = vi.fn().mockImplementation(async (cmd: string) => {
      if (cmd === "get_prefs") {
        return { paused: false, autostart: false, max_expansion_len: 1000, shell_consent: false };
      }
      if (cmd === "get_store_settings") {
        return { trigger_prefix: ":", expand_mode: "manual", expand_mode_missing: true };
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

    await waitFor(() => {
      expect(screen.queryByText(/Loading/i)).not.toBeInTheDocument();
    });

    // Migration notice should be shown
    const noticeText = screen.getByText(/Snippets now expand on Tab\/Enter. Change this in Settings./i);
    expect(noticeText).toBeInTheDocument();

    // Dismiss the notice
    const dismissButton = screen.getByRole("button", { name: /dismiss migration notice/i });
    await user.click(dismissButton);

    // Verify it called set_store_settings with expand_mode explicitly manual, and trigger_prefix preserved
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("set_store_settings", {
        settings: { trigger_prefix: ":", expand_mode: "manual" }
      });
    });

    // Notice should disappear
    expect(screen.queryByText(/Snippets now expand on Tab\/Enter/i)).not.toBeInTheDocument();
  });

  test("autostart save failure rolls back toggle and shows error", async () => {
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
      if (cmd === "set_prefs") {
        return Promise.reject("Backend rejected autostart");
      }
      return null;
    });
    window.__OPENMACRO_MOCK_INVOKE = mockInvoke;

    const user = userEvent.setup();
    render(<PrefsPanel />);

    await waitFor(() => {
      expect(screen.queryByText(/Loading/i)).not.toBeInTheDocument();
    });

    const autostartCheckbox = screen.getByLabelText(/Start on system boot/i) as HTMLInputElement;
    expect(autostartCheckbox.checked).toBe(false);

    await user.click(autostartCheckbox);

    await waitFor(() => {
      expect(screen.getByText(/Error saving/i)).toBeInTheDocument();
    });

    expect(autostartCheckbox.checked).toBe(false);
  });
});
