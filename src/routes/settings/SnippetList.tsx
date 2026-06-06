import { useState } from "react";
import { Snippet } from "../../lib/snippets";

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
      <div
        className="list-header"
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: "1rem",
        }}
      >
        <h2>Snippets</h2>
        <button type="button" onClick={onCreateSnippet} className="create-btn">
          Create Snippet
        </button>
      </div>

      <div style={{ marginBottom: "1rem" }}>
        <label htmlFor="search-snippets" style={{ display: "none" }}>
          Search Snippets
        </label>
        <input
          id="search-snippets"
          type="text"
          placeholder="Search by trigger, replacement, file..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          style={{ width: "100%", padding: "0.5rem" }}
        />
      </div>

      <div className="table-wrapper" style={{ overflowX: "auto" }}>
        <table style={{ width: "100%", borderCollapse: "collapse" }}>
          <thead>
            <tr style={{ borderBottom: "2px solid #444", textAlign: "left" }}>
              <th style={{ padding: "0.5rem" }}>Trigger</th>
              <th style={{ padding: "0.5rem" }}>Replacement</th>
              <th style={{ padding: "0.5rem" }}>File</th>
              <th style={{ padding: "0.5rem" }}>Vars</th>
              <th style={{ padding: "0.5rem", textAlign: "right" }}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {filtered.length === 0 ? (
              <tr>
                <td colSpan={5} style={{ padding: "1rem", textAlign: "center", opacity: 0.7 }}>
                  No snippets found.
                </td>
              </tr>
            ) : (
              filtered.map((s) => (
                <tr
                  key={s.id}
                  style={{
                    borderBottom: "1px solid #333",
                  }}
                  className="snippet-row"
                >
                  <td style={{ padding: "0.5rem", fontWeight: "bold" }}>{s.trigger}</td>
                  <td
                    style={{
                      padding: "0.5rem",
                      maxWidth: "200px",
                      whiteSpace: "nowrap",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                    }}
                  >
                    {s.replace}
                  </td>
                  <td style={{ padding: "0.5rem", fontSize: "0.9rem", opacity: 0.8 }}>
                    {s.file_relative}
                  </td>
                  <td style={{ padding: "0.5rem" }}>{s.vars?.length || 0}</td>
                  <td style={{ padding: "0.5rem", textAlign: "right" }}>
                    <button
                      type="button"
                      onClick={() => onEditSnippet(s)}
                      style={{ padding: "0.25rem 0.5rem", fontSize: "0.9rem" }}
                    >
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
