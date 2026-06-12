import React from "react";
import { VarDecl } from "../../../lib/snippets";

export type FieldProps = {
  decl: VarDecl;
  value: string;
  onChange: (v: string) => void;
  autoFocus?: boolean;
  error?: string;
};

export const Choice: React.FC<FieldProps> = ({ decl, value, onChange, autoFocus, error }) => {
  const id = `field-${decl.name}`;
  const hasOptions = decl.options && decl.options.length > 0;

  return (
    <div className={`form-field ${error ? "has-error" : ""}`}>
      {decl.label && <label htmlFor={id}>{decl.label}</label>}
      {!hasOptions ? (
        <div className="inline-error" style={{ color: "red" }}>
          No options defined for choice field.
        </div>
      ) : (
        <select
          id={id}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          required={decl.required}
          autoFocus={autoFocus}
        >
          {decl.options!.map((opt) => (
            <option key={opt} value={opt}>
              {opt}
            </option>
          ))}
        </select>
      )}
      {error && <span className="field-error-msg">{error}</span>}
    </div>
  );
};
