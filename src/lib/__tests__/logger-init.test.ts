import { describe, test, expect, beforeEach, vi } from "vitest";
import log from "loglevel";
import { getLogger, getRing, initFromConfig } from "../logger";

describe("logger initialization", () => {
  beforeEach(() => {
    delete window.__OPENMACRO_MOCK_INVOKE;
    vi.stubEnv("VITE_LOG_VERBOSE", "0");
    vi.stubEnv("VITE_LOG_LEVEL", "");
  });

  test("initFromConfig applies IPC level and per-module levels", async () => {
    window.__OPENMACRO_MOCK_INVOKE = vi.fn().mockResolvedValue({
      level: "warn",
      modules: {
        "test-init-sub": "error",
      },
    });

    await initFromConfig();

    expect(log.getLevel()).toBe(log.levels.WARN);
    expect(getLogger("test-init-sub").getLevel()).toBe(log.levels.ERROR);
  });

  test("VITE_LOG_LEVEL env wins over IPC config", async () => {
    window.__OPENMACRO_MOCK_INVOKE = vi.fn().mockResolvedValue({
      level: "warn",
      modules: {},
    });
    vi.stubEnv("VITE_LOG_LEVEL", "debug");

    await initFromConfig();

    expect(log.getLevel()).toBe(log.levels.DEBUG);
  });

  test("VITE_LOG_VERBOSE === '1' disables redaction", async () => {
    window.__OPENMACRO_MOCK_INVOKE = vi.fn().mockResolvedValue({
      level: "debug",
      modules: {},
    });
    vi.stubEnv("VITE_LOG_VERBOSE", "1");

    await initFromConfig();

    const subLog = getLogger("verbose-test-init");
    subLog.setLevel("debug");
    subLog.debug("test", { secret: "keep-me" });

    const ring = getRing();
    const last = ring[ring.length - 1];
    expect(last.fields?.secret).toBe("keep-me");
  });

  test("IPC rejection keeps default info level without crashing", async () => {
    window.__OPENMACRO_MOCK_INVOKE = vi.fn().mockRejectedValue(new Error("IPC failed"));

    // Should not throw
    await expect(initFromConfig()).resolves.not.toThrow();

    // Default level should be INFO
    expect(log.getLevel()).toBe(log.levels.INFO);
  });
});
