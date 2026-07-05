import { HighlightStyle, StreamLanguage, syntaxHighlighting } from "@codemirror/language";
import { tags } from "@lezer/highlight";

export const propertiesHighlightStyle = HighlightStyle.define([
  { tag: tags.comment, color: "var(--sl-text-tertiary)", fontStyle: "italic" },
  { tag: tags.propertyName, color: "var(--sl-primary)" },
  { tag: tags.definitionOperator, color: "var(--sl-text-secondary)" },
  { tag: tags.number, color: "var(--sl-warning)" },
  { tag: tags.bool, color: "var(--sl-success)" },
  { tag: tags.separator, color: "var(--sl-text-secondary)" },
]);

export const propertiesLanguage = StreamLanguage.define<{ inValue: boolean }>({
  startState() {
    return { inValue: false };
  },
  token(stream, state) {
    if (stream.sol()) {
      state.inValue = false;
      stream.eatSpace();
      if (stream.peek() === "#") {
        stream.skipToEnd();
        return "comment";
      }
    }

    if (stream.eatSpace()) {
      return null;
    }

    const ch = stream.peek();
    if (ch === "=") {
      stream.next();
      state.inValue = true;
      return "operator";
    }

    if (ch === ",") {
      stream.next();
      return "comma";
    }

    if (!state.inValue) {
      stream.eatWhile((char) => char !== "=" && char !== "#" && char !== "\n");
      return "key";
    }

    if (stream.match(/(?:true|false)\b/i)) {
      return "boolean";
    }

    if (stream.match(/[+-]?\d+(?:\.\d+)?\b/)) {
      return "number";
    }

    stream.eatWhile((char) => char !== "," && char !== "#" && char !== "\n");
    return null;
  },
  tokenTable: {
    comment: tags.comment,
    operator: tags.definitionOperator,
    number: tags.number,
    boolean: tags.bool,
    comma: tags.separator,
    key: tags.propertyName,
  },
});

export const propertiesSyntaxHighlighting = syntaxHighlighting(propertiesHighlightStyle);
