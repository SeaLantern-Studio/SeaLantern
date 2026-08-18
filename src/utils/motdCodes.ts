/**
 * Minecraft MOTD 颜色代码与富文本 HTML 的双向转换
 *
 *  ServerIntroVisualizerTool，规则与游戏内服务器列表一致：
 * - 颜色代码 0-9 a-f，样式代码 l(粗体) o(斜体) n(下划线) m(删除线) r(重置)
 * - 颜色代码会重置所有样式修饰（Minecraft 原版行为）
 * - 同时接受 § 与 & 作为代码前缀
 */

/** 16 个 Minecraft 颜色代码 */
export const MOTD_COLOR_CODES = [
  "0",
  "1",
  "2",
  "3",
  "4",
  "5",
  "6",
  "7",
  "8",
  "9",
  "a",
  "b",
  "c",
  "d",
  "e",
  "f",
] as const;

/** 颜色代码 → 经典 Minecraft 调色板 CSS 色值 */
export const MOTD_COLOR_MAP: Record<string, string> = {
  "0": "#000000",
  "1": "#0000aa",
  "2": "#00aa00",
  "3": "#00aaaa",
  "4": "#aa0000",
  "5": "#aa00aa",
  "6": "#ffaa00",
  "7": "#aaaaaa",
  "8": "#555555",
  "9": "#5555ff",
  a: "#55ff55",
  b: "#55ffff",
  c: "#ff5555",
  d: "#ff55ff",
  e: "#ffff55",
  f: "#ffffff",
};

/** 彩虹渐变使用的颜色顺序 */
export const MOTD_RAINBOW_COLOR_CODES = ["c", "6", "e", "a", "b", "9", "d"] as const;

/** 缺省 MOTD 占位文本（服务器未配置 motd 时使用） */
export const DEFAULT_MOTD = "§7A Minecraft Server";

/** 工具栏格式状态 */
export interface MotdFormatState {
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strike: boolean;
  colorCode: string;
}

/** motdToHtml / htmlToMotd 内部跟踪的当前文本修饰状态 */
interface MotdRunState {
  color: string;
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strike: boolean;
}

/** 颜色与样式代码全集（颜色 0-9 a-f + 样式 r l o n m） */
const MOTD_ALL_CODES = `${MOTD_COLOR_CODES.join("")}rlomn`;

function rgbToHex(color: string): string | null {
  const m = color.replace(/\s+/g, "").match(/^rgb\((\d+),(\d+),(\d+)\)$/i);
  if (!m) return null;
  return `#${Number(m[1]).toString(16).padStart(2, "0")}${Number(m[2]).toString(16).padStart(2, "0")}${Number(m[3]).toString(16).padStart(2, "0")}`.toLowerCase();
}

/** 将任意 CSS 颜色映射回最接近的 Minecraft 颜色代码 */
export function colorCodeFromCss(color: string): string {
  const normalized = color.trim().toLowerCase();
  const hex = normalized.startsWith("#") ? normalized : rgbToHex(normalized);
  if (!hex) return "f";
  const found = Object.entries(MOTD_COLOR_MAP).find(([, v]) => v.toLowerCase() === hex);
  return found?.[0] ?? "f";
}

/** 非 ASCII 字符转 \uXXXX 转义，保证 server.properties 编码安全 */
export function unicodeEscape(text: string): string {
  let out = "";
  for (let i = 0; i < text.length; i += 1) {
    const code = text.charCodeAt(i);
    out += code > 0x7f ? `\\u${code.toString(16).padStart(4, "0")}` : text[i];
  }
  return out;
}

/** 解码 Java Properties 风格的 \uXXXX 转义（读取时与 unicodeEscape 的写入对齐） */
export function unicodeUnescape(text: string): string {
  return text.replace(/\\u([0-9a-fA-F]{4})/g, (_, hex) => String.fromCharCode(parseInt(hex, 16)));
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

/** § 代码文本 → 内联样式 HTML（用于实时预览与编辑区渲染） */
export function motdToHtml(motd: string): string {
  const lines = motd.split("\n");
  return lines
    .map((line) => {
      const spans: string[] = [];
      let buf = "";
      let state: MotdRunState = {
        color: "f",
        bold: false,
        italic: false,
        underline: false,
        strike: false,
      };
      const flush = () => {
        if (!buf) return;
        const styleParts = [`color: ${MOTD_COLOR_MAP[state.color] ?? MOTD_COLOR_MAP.f}`];
        if (state.bold) styleParts.push("font-weight: 700");
        if (state.italic) styleParts.push("font-style: italic");
        if (state.underline || state.strike) {
          const deco = [state.underline ? "underline" : "", state.strike ? "line-through" : ""]
            .filter(Boolean)
            .join(" ");
          styleParts.push(`text-decoration: ${deco}`);
        }
        spans.push(`<span style="${styleParts.join("; ")}">${escapeHtml(buf)}</span>`);
        buf = "";
      };
      for (let i = 0; i < line.length; i += 1) {
        const ch = line[i];
        if ((ch === "§" || ch === "&") && i + 1 < line.length) {
          const code = line[i + 1].toLowerCase();
          if (MOTD_ALL_CODES.includes(code)) {
            i += 1;
            flush();
            if (code in MOTD_COLOR_MAP) {
              state = { color: code, bold: false, italic: false, underline: false, strike: false };
            } else if (code === "l") state.bold = true;
            else if (code === "o") state.italic = true;
            else if (code === "n") state.underline = true;
            else if (code === "m") state.strike = true;
            else if (code === "r")
              state = { color: "f", bold: false, italic: false, underline: false, strike: false };
            continue;
          }
        }
        buf += ch;
      }
      flush();
      return `<div>${spans.join("") || "<br>"}</div>`;
    })
    .join("");
}

/**
 * 编辑区 HTML → § 代码文本
 *
 * 依赖 DOM 解析，仅可在浏览器环境调用。用 DOMParser 替代 innerHTML
 * 以避免脚本注入，并在非浏览器环境下直接回退原始文本。
 */
export function htmlToMotd(html: string): string {
  if (typeof document === "undefined" || typeof DOMParser === "undefined") return html;
  const doc = new DOMParser().parseFromString(html, "text/html");
  const root = doc.body;
  const output: string[] = [];
  const walk = (node: Node, state: MotdRunState) => {
    if (node.nodeType === Node.TEXT_NODE) {
      output.push(node.textContent ?? "");
      return;
    }
    if (!(node instanceof HTMLElement)) return;
    if (node.tagName === "BR") {
      output.push("\n");
      return;
    }
    if (
      (node.tagName === "DIV" || node.tagName === "P") &&
      output.length &&
      output[output.length - 1] !== "\n"
    )
      output.push("\n");
    const next = { ...state };
    const code = colorCodeFromCss(node.style.color || "");
    if (code !== state.color) {
      output.push(`§${code}`);
      next.color = code;
      next.bold = false;
      next.italic = false;
      next.underline = false;
      next.strike = false;
    }
    if ((node.style.fontWeight === "700" || node.style.fontWeight === "bold") && !next.bold) {
      output.push("§l");
      next.bold = true;
    }
    if (node.style.fontStyle === "italic" && !next.italic) {
      output.push("§o");
      next.italic = true;
    }
    const td = `${node.style.textDecoration} ${node.style.textDecorationLine}`.toLowerCase();
    if (td.includes("underline") && !next.underline) {
      output.push("§n");
      next.underline = true;
    }
    if (td.includes("line-through") && !next.strike) {
      output.push("§m");
      next.strike = true;
    }
    for (const child of Array.from(node.childNodes)) walk(child, next);
  };
  const base: MotdRunState = {
    color: "f",
    bold: false,
    italic: false,
    underline: false,
    strike: false,
  };
  for (const n of Array.from(root.childNodes)) walk(n, base);
  return output
    .join("")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

/**
 * 规范化外部输入的 MOTD 文本：
 * - string：去掉 motd= 前缀，把字面 \n 拆成多行（server.properties 存储格式 → 编辑格式），解码 \uXXXX
 * - string[]：每行一个元素，仅第一个元素去 motd= 前缀，每行解码 \uXXXX，最后 join("\n")
 */
export function normalizeMotdText(raw: string | string[]): string {
  const lines = Array.isArray(raw) ? raw : raw.replace(/^motd=/i, "").split("\\n");
  return lines
    .map((line, i) => unicodeUnescape(i === 0 ? line.replace(/^motd=/i, "") : line))
    .join("\n");
}

/** 编辑格式 → server.properties 存储格式（换行转为字面 \n） */
export function motdToExportText(motd: string): string {
  return motd.replace(/\r?\n/g, "\\n");
}
