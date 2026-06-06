import { describe, test, expect } from "vitest";
import {
  mergeEntries,
  applyFilter,
  RustLogEntry,
  FrontendLogEntry,
  LogFilter,
  UiEntry,
} from "../filter";

describe("logs filter utilities", () => {
  describe("mergeEntries", () => {
    test("time-sorts mixed-source input ascending", () => {
      const rust: RustLogEntry[] = [
        {
          seq: 1,
          ts_unix_ms: 1000,
          level: "INFO",
          target: "rust-target-1",
          message: "rust msg 1",
          fields: {},
          span_path: [],
        },
        {
          seq: 3,
          ts_unix_ms: 3000,
          level: "WARN",
          target: "rust-target-3",
          message: "rust msg 3",
          fields: {},
          span_path: [],
        },
      ];

      const frontend: FrontendLogEntry[] = [
        {
          ts: 2000,
          level: "debug",
          module: "front-mod-2",
          msg: "front msg 2",
          fields: {},
        },
        {
          ts: 4000,
          level: "error",
          module: "front-mod-4",
          msg: "front msg 4",
          fields: {},
        },
      ];

      const merged = mergeEntries(rust, frontend);

      expect(merged).toHaveLength(4);
      expect(merged[0].ts).toBe(1000);
      expect(merged[0].source).toBe("rust");
      expect(merged[0].level).toBe("info");

      expect(merged[1].ts).toBe(2000);
      expect(merged[1].source).toBe("frontend");
      expect(merged[1].level).toBe("debug");

      expect(merged[2].ts).toBe(3000);
      expect(merged[2].source).toBe("rust");

      expect(merged[3].ts).toBe(4000);
      expect(merged[3].source).toBe("frontend");
    });

    test("stable-tie ordering rule preserves relative positions on same ts", () => {
      const rust: RustLogEntry[] = [
        {
          seq: 1,
          ts_unix_ms: 1000,
          level: "INFO",
          target: "rust-1",
          message: "rust-1",
          fields: {},
          span_path: [],
        },
        {
          seq: 2,
          ts_unix_ms: 1000,
          level: "WARN",
          target: "rust-2",
          message: "rust-2",
          fields: {},
          span_path: [],
        },
      ];

      const frontend: FrontendLogEntry[] = [
        {
          ts: 1000,
          level: "debug",
          module: "front-1",
          msg: "front-1",
          fields: {},
        },
      ];

      // rust-1, rust-2, front-1 all have ts=1000
      // since mergeEntries combines normalized rust entries then frontend entries,
      // the merged order should be rust-1, rust-2, front-1.
      const merged = mergeEntries(rust, frontend);

      expect(merged).toHaveLength(3);
      expect(merged[0].msg).toBe("rust-1");
      expect(merged[1].msg).toBe("rust-2");
      expect(merged[2].msg).toBe("front-1");
    });

    test("handles empty inputs correctly", () => {
      expect(mergeEntries([], [])).toEqual([]);
      expect(mergeEntries([], [{ ts: 1, level: "info", module: "m", msg: "msg" }])).toHaveLength(1);
    });
  });

  describe("applyFilter", () => {
    const entries: UiEntry[] = [
      {
        source: "rust",
        ts: 1000,
        level: "info",
        module: "rust.main",
        msg: "Started application",
        fields: { version: "1.0.0" },
      },
      {
        source: "frontend",
        ts: 2000,
        level: "debug",
        module: "front.auth",
        msg: "User typing password",
        fields: { user: "john_doe" },
      },
      {
        source: "rust",
        ts: 3000,
        level: "error",
        module: "rust.db",
        msg: "Connection failed to database",
        fields: { port: 5432 },
      },
      {
        source: "frontend",
        ts: 4000,
        level: "warn",
        module: "front.settings",
        msg: "Config warning",
        fields: { missing_key: "theme" },
      },
    ];

    const defaultFilter: LogFilter = {
      source: "all",
      minLevel: "trace",
      moduleQuery: "",
      searchQuery: "",
    };

    test("filters by source criterion", () => {
      const rustOnly = applyFilter(entries, { ...defaultFilter, source: "rust" });
      expect(rustOnly).toHaveLength(2);
      expect(rustOnly.every(e => e.source === "rust")).toBe(true);

      const frontOnly = applyFilter(entries, { ...defaultFilter, source: "frontend" });
      expect(frontOnly).toHaveLength(2);
      expect(frontOnly.every(e => e.source === "frontend")).toBe(true);

      const all = applyFilter(entries, { ...defaultFilter, source: "all" });
      expect(all).toHaveLength(4);
    });

    test("filters by minLevel criterion", () => {
      const infoOrHigher = applyFilter(entries, { ...defaultFilter, minLevel: "info" });
      // debug is lower priority than info. info, warn, error are info or higher.
      expect(infoOrHigher).toHaveLength(3);
      expect(infoOrHigher.map(e => e.level)).toEqual(["info", "error", "warn"]);

      const errorOrHigher = applyFilter(entries, { ...defaultFilter, minLevel: "error" });
      expect(errorOrHigher).toHaveLength(1);
      expect(errorOrHigher[0].level).toBe("error");
    });

    test("filters by moduleQuery case-insensitive substring", () => {
      const frontMod = applyFilter(entries, { ...defaultFilter, moduleQuery: "FRONT" });
      expect(frontMod).toHaveLength(2);
      expect(frontMod.map(e => e.module)).toEqual(["front.auth", "front.settings"]);

      const dbMod = applyFilter(entries, { ...defaultFilter, moduleQuery: "db" });
      expect(dbMod).toHaveLength(1);
      expect(dbMod[0].module).toBe("rust.db");
    });

    test("filters by searchQuery case-insensitive substring in msg and stringified fields", () => {
      // Search in msg
      const queryMsg = applyFilter(entries, { ...defaultFilter, searchQuery: "APPLICATION" });
      expect(queryMsg).toHaveLength(1);
      expect(queryMsg[0].msg).toBe("Started application");

      // Search in stringified fields
      const queryField = applyFilter(entries, { ...defaultFilter, searchQuery: "john_doe" });
      expect(queryField).toHaveLength(1);
      expect(queryField[0].fields).toEqual({ user: "john_doe" });

      const queryNumberField = applyFilter(entries, { ...defaultFilter, searchQuery: "5432" });
      expect(queryNumberField).toHaveLength(1);
      expect(queryNumberField[0].module).toBe("rust.db");
    });

    test("combines criteria narrowing", () => {
      const combined = applyFilter(entries, {
        source: "rust",
        minLevel: "info",
        moduleQuery: "db",
        searchQuery: "connection",
      });
      expect(combined).toHaveLength(1);
      expect(combined[0].module).toBe("rust.db");

      const combinedNoMatch = applyFilter(entries, {
        source: "frontend",
        minLevel: "info",
        moduleQuery: "db",
        searchQuery: "connection",
      });
      expect(combinedNoMatch).toHaveLength(0);
    });

    test("handles empty inputs and default filters without filtering anything", () => {
      expect(applyFilter([], defaultFilter)).toEqual([]);
      expect(applyFilter(entries, defaultFilter)).toHaveLength(4);
    });
  });
});
