import { render } from "@testing-library/react";
import axe from "axe-core";
import { SnippetEditor } from "../SnippetEditor";
import { Snippet } from "../../../lib/snippets";

const mockSnippet: Snippet = {
  id: "snippets/default.yaml::greet",
  trigger: "greet",
  effective_trigger: "greet",
  trigger_literal: false,
  replace: "Hello $|$",
  vars: [],
  source_file: "F:/projects_new/textblaze/snippets/default.yaml",
  file_relative: "snippets/default.yaml",
};

describe("SnippetEditor A11y", () => {
  test("should have no accessibility violations", async () => {
    const { container } = render(<SnippetEditor snippet={mockSnippet} allSnippets={[]} />);
    
    // axe.run accepts a DOM Element or Document, container is a HTMLDivElement
    const results = await axe.run(container);
    expect(results.violations).toEqual([]);
  });
});
