import { VarDecl, VarKind } from "../../lib/snippets";
import { I } from "../../lib/icons";

export interface VarsPanelProps {
  vars: VarDecl[];
  onChange: (vars: VarDecl[]) => void;
}

export function VarsPanel({ vars, onChange }: VarsPanelProps) {
  const handleAdd = () => {
    const newVar: VarDecl = { name: "", kind: "text" };
    onChange([...vars, newVar]);
  };

  const handleRemove = (index: number) => {
    const updated = vars.filter((_, i) => i !== index);
    onChange(updated);
  };

  const handleNameChange = (index: number, val: string) => {
    const updated = vars.map((v, i) => (i === index ? { ...v, name: val } : v));
    onChange(updated);
  };

  const handleKindChange = (index: number, val: VarKind) => {
    const updated = vars.map((v, i) => {
      if (i === index) {
        const newVar: VarDecl = { ...v, kind: val };
        if (val === "choice") {
          newVar.options = newVar.options || [];
        } else {
          delete newVar.options;
        }
        return newVar;
      }
      return v;
    });
    onChange(updated);
  };

  const handleOptionsChange = (index: number, val: string) => {
    const updated = vars.map((v, i) => {
      if (i === index) {
        return {
          ...v,
          options: val.split(",").map((s) => s.trim()),
        };
      }
      return v;
    });
    onChange(updated);
  };

  return (
    <div data-testid="vars-panel" className="vars-panel">
      <div className="toolbar" style={{ marginBottom: "16px" }}>
        <div className="toolbar-left">
          <h3>Variables</h3>
        </div>
        <div className="toolbar-right">
          <button type="button" onClick={handleAdd} className="btn btn-secondary sm">
            <I.Plus style={{ width: 14, height: 14, marginRight: "4px" }} />
            Add variable
          </button>
        </div>
      </div>

      {vars.length === 0 ? (
        <div style={{ padding: "16px", border: "1px dashed var(--color-border-subdued)", borderRadius: "8px", textAlign: "center", color: "var(--color-text-subdued)", marginBottom: "16px" }}>
          No variables defined for this snippet.
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: "12px", marginBottom: "16px" }}>
          {vars.map((v, index) => (
            <div
              key={index}
              className="var-row"
              style={{
                display: "grid",
                gridTemplateColumns: v.kind === "choice" ? "1fr 120px 1.5fr auto" : "1fr 120px auto",
                gap: "12px",
                alignItems: "end",
                padding: "12px",
                border: "1px solid var(--color-border-subdued)",
                borderRadius: "8px",
                backgroundColor: "var(--color-surface-subtle)"
              }}
            >
              <div className="field" style={{ marginBottom: 0 }}>
                <label htmlFor={`var-name-${index}`} style={{ fontSize: "12px" }}>Variable name</label>
                <input
                  id={`var-name-${index}`}
                  type="text"
                  placeholder="var name"
                  value={v.name}
                  onChange={(e) => handleNameChange(index, e.target.value)}
                  style={{ height: "36px", padding: "6px 10px" }}
                />
              </div>

              <div className="field" style={{ marginBottom: 0 }}>
                <label htmlFor={`var-kind-${index}`} style={{ fontSize: "12px" }}>Kind</label>
                <select
                  id={`var-kind-${index}`}
                  value={v.kind}
                  onChange={(e) => handleKindChange(index, e.target.value as VarKind)}
                  style={{ height: "36px", padding: "6px 10px" }}
                >
                  <option value="text">text</option>
                  <option value="textarea">textarea</option>
                  <option value="choice">choice</option>
                  <option value="number">number</option>
                  <option value="datetime">datetime</option>
                  <option value="clipboard">clipboard</option>
                  <option value="cursor">cursor</option>
                  <option value="shell">shell</option>
                  <option value="form">form</option>
                </select>
              </div>

              {v.kind === "choice" && (
                <div className="field" style={{ marginBottom: 0 }}>
                  <label htmlFor={`var-options-${index}`} style={{ fontSize: "12px" }}>Options (comma-separated)</label>
                  <input
                    id={`var-options-${index}`}
                    type="text"
                    placeholder="comma-separated options"
                    value={(v.options || []).join(",")}
                    onChange={(e) => handleOptionsChange(index, e.target.value)}
                    style={{ height: "36px", padding: "6px 10px" }}
                  />
                </div>
              )}

              <button
                type="button"
                onClick={() => handleRemove(index)}
                aria-label="Remove"
                className="btn btn-destructive"
                style={{ height: "36px", padding: "0 10px", fontSize: "13px", display: "inline-flex", alignItems: "center" }}
              >
                <I.Trash style={{ width: 14, height: 14 }} />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
