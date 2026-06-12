import { useState } from "react";
import { Snippet } from "../../lib/snippets";
import { I } from "../../lib/icons";

export interface SnippetListProps {
  snippets: Snippet[];
  onEditSnippet: (snippet: Snippet) => void;
  onCreateSnippet: () => void;
}

export function SnippetList({ snippets, onEditSnippet, onCreateSnippet }: SnippetListProps) {
  const [search, setSearch] = useState("");

  const filtered = snippets.filter((s) => {
    const query = search.toLowerCase();
    return (
      s.trigger.toLowerCase().includes(query) ||
      s.replace.toLowerCase().includes(query) ||
      s.file_relative.toLowerCase().includes(query)
    );
  });

  return (
    <div data-testid="snippet-list" className="snippet-list-container">
      {/* Header and Create Button */}
      <div className="toolbar" style={{ marginBottom: "24px" }}>
        <div className="toolbar-left">
          <h2>Snippets</h2>
        </div>
        <div className="toolbar-right">
          <button type="button" onClick={onCreateSnippet} className="btn primary" style={{ height: "36px", padding: "0 12px", fontSize: "14px" }}>
            <I.Plus style={{ width: 14, height: 14, marginRight: "6px" }} />
            Create snippet
          </button>
        </div>
      </div>

      {/* Search Bar */}
      <div className="toolbar" style={{ marginBottom: "16px" }}>
        <div className="toolbar-left" style={{ width: "100%" }}>
          <div className="search-input-group" style={{ width: "100%" }}>
            <I.Search />
            <input
              id="search-snippets"
              type="text"
              placeholder="Search by trigger, replacement, file..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              aria-label="Search snippets"
            />
          </div>
        </div>
      </div>

      {/* Table Section */}
      <div className="panel">
        <table className="table">
          <thead>
            <tr>
              <th>Trigger</th>
              <th>Replacement</th>
              <th>File</th>
              <th>Vars</th>
              <th className="right" style={{ width: "100px" }}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {filtered.length === 0 ? (
              <tr>
                <td colSpan={5} style={{ padding: "24px", textAlign: "center", color: "var(--color-text-subdued)" }}>
                  No snippets found.
                </td>
              </tr>
            ) : (
              filtered.map((s) => (
                <tr key={s.id}>
                  <td style={{ fontWeight: 600 }}>{s.trigger}</td>
                  <td
                    style={{
                      maxWidth: "300px",
                      whiteSpace: "nowrap",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      color: "var(--color-text-subdued)",
                    }}
                  >
                    {s.replace}
                  </td>
                  <td style={{ color: "var(--color-text-subdued)" }}>
                    {s.file_relative}
                  </td>
                  <td className="mono" style={{ color: "var(--color-text-placeholder)" }}>
                    {s.vars?.length || 0}
                  </td>
                  <td className="right">
                    <button
                      type="button"
                      onClick={() => onEditSnippet(s)}
                      className="btn btn-secondary sm"
                      style={{ height: "30px", padding: "0 10px", fontSize: "13px" }}
                    >
                      <I.Edit style={{ width: 12, height: 12, marginRight: "4px" }} />
                      Edit
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
