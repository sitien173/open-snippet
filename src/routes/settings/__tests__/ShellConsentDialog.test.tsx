import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";

import { ShellConsentDialog } from "../ShellConsentDialog";
import { Prefs } from "../../../lib/snippets";

const basePrefs: Prefs = {
  paused: false,
  autostart: false,
  max_expansion_len: 32768,
  shell_consent: false,
};

describe("ShellConsentDialog", () => {
  test("renders when shell consent is false", () => {
    render(
      <ShellConsentDialog
        prefs={basePrefs}
        open={true}
        setPrefs={vi.fn()}
        onClose={vi.fn()}
      />
    );

    expect(screen.getByRole("dialog", { name: /shell execution consent/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /accept/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /decline/i })).toBeInTheDocument();
  });

  test("hides when shell consent is already true", () => {
    render(
      <ShellConsentDialog
        prefs={{ ...basePrefs, shell_consent: true }}
        open={true}
        setPrefs={vi.fn()}
        onClose={vi.fn()}
      />
    );

    expect(screen.queryByRole("dialog", { name: /shell execution consent/i })).not.toBeInTheDocument();
  });

  test("accept persists shell consent", async () => {
    const user = userEvent.setup();
    const setPrefs = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();
    render(
      <ShellConsentDialog
        prefs={basePrefs}
        open={true}
        setPrefs={setPrefs}
        onClose={onClose}
      />
    );

    await user.click(screen.getByRole("button", { name: /accept/i }));

    await waitFor(() => {
      expect(setPrefs).toHaveBeenCalledWith({
        ...basePrefs,
        shell_consent: true,
      });
      expect(onClose).toHaveBeenCalled();
    });
  });
});
