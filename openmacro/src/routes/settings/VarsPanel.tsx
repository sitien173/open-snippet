import { VarDecl, VarKind } from "../../lib/snippets";

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
      <h3>Variables</h3>
      {vars.map((v, index) => (
        <div key={index} className="var-row" style={{ marginBottom: "1rem", display: "flex", gap: "0.5rem", alignItems: "center" }}>
          <div>
            <label htmlFor={`var-name-${index}`}>Var Name</label>
            <input
              id={`var-name-${index}`}
              type="text"
              placeholder="var name"
              value={v.name}
              onChange={(e) => handleNameChange(index, e.target.value)}
            />
          </div>

          <div>
            <label htmlFor={`var-kind-${index}`}>Kind</label>
            <select
              id={`var-kind-${index}`}
              value={v.kind}
              onChange={(e) => handleKindChange(index, e.target.value as VarKind)}
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
            <div>
              <label htmlFor={`var-options-${index}`}>Options</label>
              <input
                id={`var-options-${index}`}
                type="text"
                placeholder="comma-separated options"
                value={(v.options || []).join(",")}
                onChange={(e) => handleOptionsChange(index, e.target.value)}
              />
            </div>
          )}

          <button type="button" onClick={() => handleRemove(index)}>
            Remove
          </button>
        </div>
      ))}
      <button type="button" onClick={handleAdd}>
        Add Variable
      </button>
    </div>
  );
}
