import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, test, expect, beforeEach, afterEach, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";
import { LogsRoute } from "../index";
import * as logger from "../../../lib/logger";
import { RustLogEntry, FrontendLogEntry } from "../filter";

vi.mock("../../../lib/logger", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../../../lib/logger")>();
  return {
    ...actual,
    getRing: vi.fn().mockReturnValue([]),
  };
});

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => ({
    getVirtualItems: () =>
      Array.from({ length: count }, (_, index) => ({
        index,
        key: index,
        start: index * 36,
        size: 36,
      })),
    getTotalSize: () => count * 36,
    measureElement: () => {},
  }),
}));

describe("LogsRoute", () => {
  const mockRustEntries: RustLogEntry[] = [
    {
      seq: 1,
      ts_unix_ms: 1000,
      level: "INFO",
      target: "rust.db",
      message: "Rust db connected",
      fields: { host: "localhost" },
      span_path: [],
    },
  ];

  const mockFrontendEntries: FrontendLogEntry[] = [
    {
      ts: 2000,
      level: "debug",
      module: "front.auth",
      msg: "Frontend user logged in",
      fields: { user: "alice" },
    },
  ];

  beforeEach(() => {
    vi.mocked(logger.getRing).mockReturnValue(mockFrontendEntries);

    window.__OPENMACRO_MOCK_INVOKE = async (cmd) => {
      if (cmd === "get_log_ring") {
        return mockRustEntries;
      }
      return undefined;
    };

    window.URL.createObjectURL = vi.fn().mockReturnValue("blob:mock-url");
    window.URL.revokeObjectURL = vi.fn();
  });

  afterEach(() => {
    delete window.__OPENMACRO_MOCK_INVOKE;
    vi.restoreAllMocks();
  });

  test("polls and merges rust + frontend logs on mount", async () => {
    render(
      <MemoryRouter>
        <LogsRoute />
      </MemoryRouter>
    );

    await screen.findByText("Rust db connected");
    await screen.findByText("Frontend user logged in");

    expect(screen.getByRole("log")).toBeInTheDocument();
  });

  test("pause and resume stops/restarts polling", async () => {
    const setIntervalSpy = vi.spyOn(window, "setInterval");
    const clearIntervalSpy = vi.spyOn(window, "clearInterval");

    const user = userEvent.setup();
    render(
      <MemoryRouter>
        <LogsRoute />
      </MemoryRouter>
    );

    await screen.findByText("Rust db connected");
    expect(setIntervalSpy).toHaveBeenCalled();
    const initialCalls = setIntervalSpy.mock.calls.length;

    const pauseBtn = screen.getByRole("button", { name: /pause/i });
    await user.click(pauseBtn);

    expect(clearIntervalSpy).toHaveBeenCalled();
    expect(screen.getByRole("button", { name: /resume/i })).toBeInTheDocument();

    const resumeBtn = screen.getByRole("button", { name: /resume/i });
    await user.click(resumeBtn);

    expect(setIntervalSpy).toHaveBeenCalledTimes(initialCalls + 1);
    expect(screen.getByRole("button", { name: /pause/i })).toBeInTheDocument();

    setIntervalSpy.mockRestore();
    clearIntervalSpy.mockRestore();
  });

  test("Save click constructs JSON-lines blob and triggers download", async () => {
    const user = userEvent.setup();

    render(
      <MemoryRouter>
        <LogsRoute />
      </MemoryRouter>
    );

    await screen.findByText("Rust db connected");

    const dummyAnchor = {
      click: vi.fn(),
      href: "",
      download: "",
    };
    const createElementSpy = vi.spyOn(document, "createElement").mockImplementation((tagName) => {
      if (tagName === "a") return dummyAnchor as unknown as HTMLAnchorElement;
      return document.createElement(tagName);
    });

    const saveBtn = screen.getByRole("button", { name: /save/i });
    await user.click(saveBtn);

    expect(window.URL.createObjectURL).toHaveBeenCalled();
    expect(dummyAnchor.click).toHaveBeenCalled();
    expect(dummyAnchor.download).toContain("openmacro-logs-");
    expect(dummyAnchor.download).toContain(".jsonl");

    createElementSpy.mockRestore();
  });
});
