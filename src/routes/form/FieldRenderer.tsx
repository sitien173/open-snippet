import React from "react";
import { VarDecl } from "../../lib/snippets";
import { Text } from "./fields/Text";
import { Textarea } from "./fields/Textarea";
import { Choice } from "./fields/Choice";
import { NumberField } from "./fields/NumberField";

type FieldRendererProps = {
  decl: VarDecl;
  value: string;
  onChange: (v: string) => void;
  autoFocus?: boolean;
  error?: string;
};

export const FieldRenderer: React.FC<FieldRendererProps> = ({
  decl,
  value,
  onChange,
  autoFocus,
  error,
}) => {
  switch (decl.kind) {
    case "text":
      return <Text decl={decl} value={value} onChange={onChange} autoFocus={autoFocus} error={error} />;
    case "textarea":
      return <Textarea decl={decl} value={value} onChange={onChange} autoFocus={autoFocus} error={error} />;
    case "choice":
      return <Choice decl={decl} value={value} onChange={onChange} autoFocus={autoFocus} error={error} />;
    case "number":
      return <NumberField decl={decl} value={value} onChange={onChange} autoFocus={autoFocus} error={error} />;
    default:
      // Skip non-form kinds (datetime, clipboard, cursor, shell, form) defensively.
      return null;
  }
};
