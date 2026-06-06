import React from "react";
import { VarDecl } from "../../../lib/snippets";

export type FieldProps = {
  decl: VarDecl;
  value: string;
  onChange: (v: string) => void;
  autoFocus?: boolean;
  error?: string;
};

export const Textarea: React.FC<FieldProps> = ({ decl, value, onChange, autoFocus, error }) => {
  const id = `field-${decl.name}`;
  return (
    <div className={`form-field ${error ? "has-error" : ""}`}>
      {decl.label && <label htmlFor={id}>{decl.label}</label>}
      <textarea
        id={id}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        required={decl.required}
        autoFocus={autoFocus}
      />
      {error && <span className="field-error-msg">{error}</span>}
    </div>
  );
};
