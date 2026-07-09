export const SECTION_SIGN = "\u00a7";

export type MotdEdition = "java" | "bedrock";

type JavaColorCode =
  | "0"
  | "1"
  | "2"
  | "3"
  | "4"
  | "5"
  | "6"
  | "7"
  | "8"
  | "9"
  | "a"
  | "b"
  | "c"
  | "d"
  | "e"
  | "f";

type JavaFormatCode = "k" | "l" | "m" | "n" | "o" | "r";

type BedrockExtraColorCode = "g" | "h" | "i" | "j" | "p" | "q" | "s" | "t" | "u" | "v" | "w";

type BedrockFormatCode = "k" | "l" | "o" | "r";

type JavaControlCode = JavaColorCode | JavaFormatCode;
type BedrockControlCode = JavaColorCode | BedrockExtraColorCode | BedrockFormatCode | "m" | "n";

export interface MotdColorOption {
  key: string;
  code: string;
  labelKey: string;
  color: string;
}

export interface MotdFormatOption {
  key: string;
  code: string;
  labelKey: string;
}

export interface MotdToken {
  text: string;
  color: string;
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strikethrough: boolean;
  obfuscated: boolean;
}

interface MotdStyleState {
  color: string;
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strikethrough: boolean;
  obfuscated: boolean;
}

interface ColorMeta {
  labelKey: string;
  color: string;
}

const DEFAULT_COLOR = "#ffffff";

const JAVA_COLOR_MAP: Record<JavaColorCode, ColorMeta> = {
  "0": { labelKey: "config.next_v1.motd.color_black", color: "#000000" },
  "1": { labelKey: "config.next_v1.motd.color_dark_blue", color: "#0000aa" },
  "2": { labelKey: "config.next_v1.motd.color_dark_green", color: "#00aa00" },
  "3": { labelKey: "config.next_v1.motd.color_dark_aqua", color: "#00aaaa" },
  "4": { labelKey: "config.next_v1.motd.color_dark_red", color: "#aa0000" },
  "5": { labelKey: "config.next_v1.motd.color_dark_purple", color: "#aa00aa" },
  "6": { labelKey: "config.next_v1.motd.color_gold", color: "#ffaa00" },
  "7": { labelKey: "config.next_v1.motd.color_gray", color: "#aaaaaa" },
  "8": { labelKey: "config.next_v1.motd.color_dark_gray", color: "#555555" },
  "9": { labelKey: "config.next_v1.motd.color_blue", color: "#5555ff" },
  a: { labelKey: "config.next_v1.motd.color_green", color: "#55ff55" },
  b: { labelKey: "config.next_v1.motd.color_aqua", color: "#55ffff" },
  c: { labelKey: "config.next_v1.motd.color_red", color: "#ff5555" },
  d: { labelKey: "config.next_v1.motd.color_light_purple", color: "#ff55ff" },
  e: { labelKey: "config.next_v1.motd.color_yellow", color: "#ffff55" },
  f: { labelKey: "config.next_v1.motd.color_white", color: "#ffffff" },
};

const BEDROCK_EXTRA_COLOR_MAP: Record<BedrockExtraColorCode | "m" | "n", ColorMeta> = {
  g: { labelKey: "config.next_v1.motd.color_minecoin_gold", color: "#ddd605" },
  h: { labelKey: "config.next_v1.motd.color_material_quartz", color: "#e3d4d1" },
  i: { labelKey: "config.next_v1.motd.color_material_iron", color: "#cecaca" },
  j: { labelKey: "config.next_v1.motd.color_material_netherite", color: "#443a3b" },
  m: { labelKey: "config.next_v1.motd.color_material_redstone", color: "#971607" },
  n: { labelKey: "config.next_v1.motd.color_material_copper", color: "#b4684d" },
  p: { labelKey: "config.next_v1.motd.color_material_gold", color: "#deb12d" },
  q: { labelKey: "config.next_v1.motd.color_material_emerald", color: "#11a036" },
  s: { labelKey: "config.next_v1.motd.color_material_diamond", color: "#2cbaa8" },
  t: { labelKey: "config.next_v1.motd.color_material_lapis", color: "#21497b" },
  u: { labelKey: "config.next_v1.motd.color_material_amethyst", color: "#9a5cc6" },
  v: { labelKey: "config.next_v1.motd.color_material_resin", color: "#eb7114" },
  w: { labelKey: "config.next_v1.motd.color_party_blue", color: "#8cb3ff" },
};

const OBFUSCATION_GLYPHS = ["#", "@", "%", "&", "*", "?", "!", "X", "M", "W"];

const DEFAULT_STYLE_STATE: MotdStyleState = {
  color: DEFAULT_COLOR,
  bold: false,
  italic: false,
  underline: false,
  strikethrough: false,
  obfuscated: false,
};

export const RESET_CODE = `${SECTION_SIGN}r`;

export const JAVA_COLOR_OPTIONS: MotdColorOption[] = buildColorOptions(JAVA_COLOR_MAP);
export const BEDROCK_COLOR_OPTIONS: MotdColorOption[] = buildColorOptions({
  ...JAVA_COLOR_MAP,
  ...BEDROCK_EXTRA_COLOR_MAP,
});

export const JAVA_FORMAT_OPTIONS: MotdFormatOption[] = [
  { key: "k", code: `${SECTION_SIGN}k`, labelKey: "config.next_v1.motd.format_obfuscated" },
  { key: "l", code: `${SECTION_SIGN}l`, labelKey: "config.next_v1.motd.format_bold" },
  { key: "m", code: `${SECTION_SIGN}m`, labelKey: "config.next_v1.motd.format_strikethrough" },
  { key: "n", code: `${SECTION_SIGN}n`, labelKey: "config.next_v1.motd.format_underline" },
  { key: "o", code: `${SECTION_SIGN}o`, labelKey: "config.next_v1.motd.format_italic" },
];

export const BEDROCK_FORMAT_OPTIONS: MotdFormatOption[] = [
  { key: "k", code: `${SECTION_SIGN}k`, labelKey: "config.next_v1.motd.format_obfuscated" },
  { key: "l", code: `${SECTION_SIGN}l`, labelKey: "config.next_v1.motd.format_bold" },
  { key: "o", code: `${SECTION_SIGN}o`, labelKey: "config.next_v1.motd.format_italic" },
];

function buildColorOptions(map: Record<string, ColorMeta>): MotdColorOption[] {
  return Object.entries(map).map(([key, value]) => ({
    key,
    code: `${SECTION_SIGN}${key}`,
    labelKey: value.labelKey,
    color: value.color,
  }));
}

function isJavaControlCode(code: string): code is JavaControlCode {
  return /^[0-9a-fklmnor]$/i.test(code);
}

function isBedrockControlCode(code: string): code is BedrockControlCode {
  return /^[0-9a-wklmnoprstuv]$/i.test(code) || code.toLowerCase() === "q";
}

function cloneStyleState(state: MotdStyleState): MotdStyleState {
  return { ...state };
}

function obfuscateText(text: string): string {
  return Array.from(text)
    .map((char, index) => {
      if (/\s/.test(char)) {
        return char;
      }

      const glyphIndex = (char.charCodeAt(0) + index) % OBFUSCATION_GLYPHS.length;
      return OBFUSCATION_GLYPHS[glyphIndex];
    })
    .join("");
}

function applyJavaCode(state: MotdStyleState, code: JavaControlCode): MotdStyleState {
  if (code === "r") {
    return cloneStyleState(DEFAULT_STYLE_STATE);
  }

  if (code in JAVA_COLOR_MAP) {
    return {
      color: JAVA_COLOR_MAP[code as JavaColorCode].color,
      bold: false,
      italic: false,
      underline: false,
      strikethrough: false,
      obfuscated: false,
    };
  }

  const next = cloneStyleState(state);
  if (code === "k") {
    next.obfuscated = true;
  }
  if (code === "l") {
    next.bold = true;
  }
  if (code === "m") {
    next.strikethrough = true;
  }
  if (code === "n") {
    next.underline = true;
  }
  if (code === "o") {
    next.italic = true;
  }
  return next;
}

function applyBedrockCode(state: MotdStyleState, code: BedrockControlCode): MotdStyleState {
  if (code === "r") {
    return cloneStyleState(DEFAULT_STYLE_STATE);
  }

  if (code in JAVA_COLOR_MAP) {
    return {
      color: JAVA_COLOR_MAP[code as JavaColorCode].color,
      bold: false,
      italic: false,
      underline: false,
      strikethrough: false,
      obfuscated: false,
    };
  }

  if (code in BEDROCK_EXTRA_COLOR_MAP) {
    return {
      color: BEDROCK_EXTRA_COLOR_MAP[code as keyof typeof BEDROCK_EXTRA_COLOR_MAP].color,
      bold: false,
      italic: false,
      underline: false,
      strikethrough: false,
      obfuscated: false,
    };
  }

  const next = cloneStyleState(state);
  if (code === "k") {
    next.obfuscated = true;
  }
  if (code === "l") {
    next.bold = true;
  }
  if (code === "o") {
    next.italic = true;
  }
  return next;
}

export function getEditionColorOptions(edition: MotdEdition): MotdColorOption[] {
  return edition === "bedrock" ? BEDROCK_COLOR_OPTIONS : JAVA_COLOR_OPTIONS;
}

export function getEditionFormatOptions(edition: MotdEdition): MotdFormatOption[] {
  return edition === "bedrock" ? BEDROCK_FORMAT_OPTIONS : JAVA_FORMAT_OPTIONS;
}

export function getEditionNoteKey(edition: MotdEdition): string {
  return edition === "bedrock"
    ? "config.next_v1.motd.mode_note_bedrock"
    : "config.next_v1.motd.mode_note_java";
}

export function detectMotdEdition(value: string): MotdEdition {
  const normalized = decodeMotdForEditor(value);
  return new RegExp(`${SECTION_SIGN}[ghijpqsuvw]`, "i").test(normalized) ? "bedrock" : "java";
}

export function decodeMotdForEditor(value: string): string {
  return value.replace(/\\u00a7/gi, SECTION_SIGN).replace(/\\n/g, "\n");
}

export function encodeMotdForSource(value: string): string {
  return value.replace(/\r\n?/g, "\n").replace(/\n/g, "\\n");
}

export function extractMotdFromSource(source: string): string {
  const lines = source.split(/\r?\n/);

  for (const line of lines) {
    if (/^\s*[#!]/.test(line)) {
      continue;
    }

    const match = line.match(/^\s*motd\s*[:=](.*)$/i);
    if (match) {
      return decodeMotdForEditor(match[1].replace(/^\s*/, ""));
    }
  }

  return "";
}

export function updateMotdInSource(source: string, value: string): string {
  const encoded = encodeMotdForSource(value);
  let replaced = false;

  const updated = source.replace(
    /^((?:\s*)motd\s*)([:=])(\s*)([^\r\n]*)/im,
    (_, prefix, separator, spacing) => {
      replaced = true;
      return `${prefix}${separator}${spacing}${encoded}`;
    },
  );

  if (replaced) {
    return updated;
  }

  if (!source) {
    return `motd=${encoded}`;
  }

  const newline = source.includes("\r\n") ? "\r\n" : "\n";
  return source.endsWith("\n") || source.endsWith("\r")
    ? `${source}motd=${encoded}`
    : `${source}${newline}motd=${encoded}`;
}

export function parseMotdPreview(value: string, edition: MotdEdition): MotdToken[][] {
  const normalized = decodeMotdForEditor(value).replace(/\r\n?/g, "\n");
  const lines: MotdToken[][] = [[]];
  let buffer = "";
  let state = cloneStyleState(DEFAULT_STYLE_STATE);

  const pushBuffer = () => {
    if (!buffer) {
      return;
    }

    lines[lines.length - 1].push({
      text: state.obfuscated ? obfuscateText(buffer) : buffer,
      color: state.color,
      bold: state.bold,
      italic: state.italic,
      underline: state.underline,
      strikethrough: state.strikethrough,
      obfuscated: state.obfuscated,
    });
    buffer = "";
  };

  for (let index = 0; index < normalized.length; index += 1) {
    const char = normalized[index];
    const nextChar = normalized[index + 1]?.toLowerCase() ?? "";

    const isControl =
      edition === "bedrock" ? isBedrockControlCode(nextChar) : isJavaControlCode(nextChar);

    if (char === SECTION_SIGN && isControl) {
      pushBuffer();
      state =
        edition === "bedrock"
          ? applyBedrockCode(state, nextChar as BedrockControlCode)
          : applyJavaCode(state, nextChar as JavaControlCode);
      index += 1;
      continue;
    }

    if (char === "\n") {
      pushBuffer();
      lines.push([]);
      continue;
    }

    buffer += char;
  }

  pushBuffer();

  return lines;
}
