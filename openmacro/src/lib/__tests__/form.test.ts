import { vi, describe, test, expect, beforeEach } from "vitest";
import { formSubmit, formCancel } from "../form";

describe("form lib IPC wrapper", () => {
  beforeEach(() => {
    delete window.__OPENMACRO_MOCK_INVOKE;
  });

  test("formSubmit calls form_submit IPC with correct arguments", async () => {
    const mockInvoke = vi.fn().mockResolvedValue(undefined);
    window.__OPENMACRO_MOCK_INVOKE = mockInvoke;

    await formSubmit("test-id", { foo: "bar" });

    expect(mockInvoke).toHaveBeenCalledWith("form_submit", {
      snippetId: "test-id",
      values: { foo: "bar" },
    });
  });

  test("formCancel calls form_cancel IPC with correct arguments", async () => {
    const mockInvoke = vi.fn().mockResolvedValue(undefined);
    window.__OPENMACRO_MOCK_INVOKE = mockInvoke;

    await formCancel("test-id");

    expect(mockInvoke).toHaveBeenCalledWith("form_cancel", {
      snippetId: "test-id",
    });
  });
});
