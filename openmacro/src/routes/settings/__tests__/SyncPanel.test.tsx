import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { SyncPanel } from "../SyncPanel";

describe("SyncPanel", () => {
  beforeEach(() => {
    window.__OPENMACRO_MOCK_INVOKE = vi.fn(async (cmd, args) => {
      if (cmd === "sync_status") {
        return {
          branch: "master",
          ahead: 0,
          behind: 0,
          last_tick_unix: 1710000000,
        };
      }
      if (cmd === "sync_test_connection") {
        return null;
      }
      if (cmd === "sync_init") {
        return null;
      }
      if (cmd === "sync_tick_now") {
        return {
          kind: "synced",
          dir: null,
          committed: true,
        };
      }
      throw new Error(`Unhandled command ${cmd} ${JSON.stringify(args)}`);
    });
  });

  test("test connection happy path shows success", async () => {
    const user = userEvent.setup();
    render(<SyncPanel />);

    await user.type(screen.getByLabelText(/remote url/i), "https://example.com/repo.git");
    await user.type(screen.getByLabelText(/personal access token/i), "pat-123");
    await user.click(screen.getByRole("button", { name: /test connection/i }));

    await waitFor(() => {
      expect(window.__OPENMACRO_MOCK_INVOKE).toHaveBeenCalledWith(
        "sync_test_connection",
        expect.objectContaining({
          remote: "https://example.com/repo.git",
          pat: "pat-123",
        }),
      );
      expect(screen.getByText(/connection ok/i)).toBeInTheDocument();
    });
  });

  test("missing pat for https shows inline error", async () => {
    const user = userEvent.setup();
    render(<SyncPanel />);

    await user.type(screen.getByLabelText(/remote url/i), "https://example.com/repo.git");
    await user.click(screen.getByRole("button", { name: /save & init/i }));

    expect(screen.getByText(/pat is required for https/i)).toBeInTheDocument();
    expect(window.__OPENMACRO_MOCK_INVOKE).not.toHaveBeenCalledWith(
      "sync_init",
      expect.anything(),
    );
  });

  test("sync now shows tick result", async () => {
    render(<SyncPanel />);

    fireEvent.click(screen.getByRole("button", { name: /sync now/i }));

    await waitFor(() => {
      expect(window.__OPENMACRO_MOCK_INVOKE).toHaveBeenCalledWith("sync_tick_now");
      expect(screen.getByText(/last result: synced/i)).toBeInTheDocument();
    });
  });
});
