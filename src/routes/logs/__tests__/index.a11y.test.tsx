import { render, screen } from "@testing-library/react";
import axe from "axe-core";
import { vi, describe, test, expect, beforeEach, afterEach } from "vitest";
import { MemoryRouter } from "react-router-dom";
import { LogsRoute } from "../index";

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

describe("LogsRoute A11y", () => {
  beforeEach(() => {
    window.__OPENMACRO_MOCK_INVOKE = async (cmd) => {
      if (cmd === "get_log_ring") {
        return [];
      }
      return undefined;
    };
  });

  afterEach(() => {
    delete window.__OPENMACRO_MOCK_INVOKE;
  });

  test("should have no accessibility violations", async () => {
    const { container } = render(
      <MemoryRouter>
        <LogsRoute />
      </MemoryRouter>
    );

    const logContainer = await screen.findByRole("log");
    expect(logContainer).toBeInTheDocument();

    const results = await axe.run(container);
    expect(results.violations).toEqual([]);
  });
});
