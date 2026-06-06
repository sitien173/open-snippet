import { describe, test, expect, beforeEach, vi } from "vitest";
import { getLogger, getRing, initFromConfig } from "../logger";

describe("logger frontend module", () => {
  beforeEach(() => {
    // Clear the ring before each test. Since getRing returns a slice, we need a way to clear it,
    // or we can test it relative to current length. Let's make sure our tests are robust.
    vi.stubEnv("VITE_LOG_VERBOSE", "0");
    vi.stubEnv("VITE_LOG_LEVEL", "debug");
  });

  test("getLogger('x').debug('msg', fields) pushes into the ring", async () => {
    // Clear mock invoke so initFromConfig doesn't crash or hang if called.
    // For this test we don't necessarily call initFromConfig, we just use the logger directly.
    const ringBefore = getRing();
    const initialLen = ringBefore.length;
    const log = getLogger("test-module");
    log.setLevel("debug");

    log.debug("hello world", { foo: "bar" });

    const ring = getRing();
    expect(ring.length).toBe(initialLen + 1);
    const lastEntry = ring[ring.length - 1];
    expect(lastEntry.msg).toContain("hello world");
    expect(lastEntry.module).toBe("test-module");
    expect(lastEntry.level).toBe("debug");
    expect(lastEntry.fields).toEqual({ foo: "bar" });
    expect(typeof lastEntry.ts).toBe("number");
  });

  test("ring buffer is capped at 1000 and is FIFO", () => {
    const log = getLogger("fifo-module");
    log.setLevel("debug");

    // Push 1005 log entries
    for (let i = 0; i < 1005; i++) {
      log.debug(`msg ${i}`);
    }

    const ring = getRing();
    expect(ring.length).toBe(1000);
    // The first entry should be index 5 because 0, 1, 2, 3, 4 were evicted
    expect(ring[0].msg).toContain("msg 5");
    expect(ring[999].msg).toContain("msg 1004");
  });

  test("redacts sensitive keys case-insensitively and respects full-string match", () => {
    const log = getLogger("redact-module");
    log.setLevel("debug");

    // Test sensitive key: 'value' (case insensitive)
    log.debug("test value", { VALUE: "secretValue" });
    let ring = getRing();
    let last = ring[ring.length - 1];
    expect(last.fields?.VALUE).toBe("<redacted len=11>");

    // Test non-sensitive key containing sensitive word: 'bodyguard'
    log.debug("test bodyguard", { bodyguard: "safe" });
    ring = getRing();
    last = ring[ring.length - 1];
    expect(last.fields?.bodyguard).toBe("safe");

    // Test non-string sensitive value
    log.debug("test non-string secret", { secret: 12345 });
    ring = getRing();
    last = ring[ring.length - 1];
    expect(last.fields?.secret).toBe("<redacted>");

    // Test other sensitive keys: body, content, clipboard, token, password, secret
    const sensitive = {
      body: "bodyText",
      content: "contentVal",
      clipboard: "clipVal",
      token: "tokenVal",
      password: "pass",
      secret: "secVal"
    };
    log.debug("test all", sensitive);
    ring = getRing();
    last = ring[ring.length - 1];
    expect(last.fields).toEqual({
      body: "<redacted len=8>",
      content: "<redacted len=10>",
      clipboard: "<redacted len=7>",
      token: "<redacted len=8>",
      password: "<redacted len=4>",
      secret: "<redacted len=6>"
    });
  });

  test("correctly counts code-points for unicode in redaction", () => {
    const log = getLogger("unicode-module");
    log.setLevel("debug");

    // "😊🔑" is 2 characters in JS string.length, but 2 code points.
    // Let's also check a flag emoji or complex unicode.
    // "😊" is 1 emoji (code point U+1F60A). "🔑" is 1 emoji (code point U+1F511).
    // string.length is 4, but [...v].length is 2.
    log.debug("unicode test", { secret: "😊🔑" });
    const ring = getRing();
    const last = ring[ring.length - 1];
    expect(last.fields?.secret).toBe("<redacted len=2>");
  });

  test("verboseContent=true disables redaction", async () => {
    // Setup window mock invoke for initFromConfig
    window.__OPENMACRO_MOCK_INVOKE = vi.fn().mockResolvedValue({
      level: "debug",
      modules: {}
    });

    vi.stubEnv("VITE_LOG_VERBOSE", "1");
    await initFromConfig();

    const log = getLogger("verbose-module");
    log.setLevel("debug");

    log.debug("verbose test", { secret: "my-secret-token" });
    const ring = getRing();
    const last = ring[ring.length - 1];
    expect(last.fields?.secret).toBe("my-secret-token");
  });
});
