import assert from "node:assert/strict";
import test from "node:test";
import {
  MOTD_COLOR_MAP,
  colorCodeFromCss,
  motdToExportText,
  motdToHtml,
  normalizeMotdText,
  unicodeEscape,
} from "../src/utils/motdCodes.ts";

test("motdToHtml 渲染颜色代码为内联样式", () => {
  const html = motdToHtml("§aGreen§cRed");
  assert.match(html, /color: #55ff55[^>]*>Green</);
  assert.match(html, /color: #ff5555[^>]*>Red</);
});

test("motdToHtml 渲染样式代码并支持 & 前缀", () => {
  const html = motdToHtml("&lBold &oItalic");
  assert.match(html, /font-weight: 700[^>]*>Bold </);
  assert.match(html, /font-style: italic;?[^>]*>Italic</);
});

test("motdToHtml 颜色代码重置样式修饰（Minecraft 语义）", () => {
  const html = motdToHtml("§l§aGreenAfterColor");
  // §a 重置 bold，因此 GreenAfterColor 不应带 font-weight
  assert.doesNotMatch(html, /font-weight: 700[^>]*>GreenAfterColor</);
});

test("motdToHtml 对文本做 HTML 转义", () => {
  const html = motdToHtml('§a<script>&"');
  assert.ok(!html.includes("<script>"));
  assert.ok(html.includes("&lt;script&gt;"));
  assert.ok(html.includes("&amp;"));
  assert.ok(html.includes("&quot;"));
});

test("motdToHtml 按换行拆分为多行", () => {
  const html = motdToHtml("Line1\nLine2");
  assert.equal((html.match(/<div>/g) ?? []).length, 2);
});

test("motdToHtml 空行输出占位 <br>", () => {
  assert.equal(
    motdToHtml("\na"),
    '<div><br></div><div><span style="color: #ffffff">a</span></div>',
  );
});

test("colorCodeFromCss 识别 hex 与 rgb 颜色", () => {
  assert.equal(colorCodeFromCss("#55ff55"), "a");
  assert.equal(colorCodeFromCss("rgb(85, 255, 85)"), "a");
  assert.equal(colorCodeFromCss("#FFFFFF"), "f");
  // 未知颜色回退到白色
  assert.equal(colorCodeFromCss("rgb(1, 2, 3)"), "f");
});

test("unicodeEscape 转义非 ASCII 字符", () => {
  assert.equal(unicodeEscape("§a你好"), "\\u00a7a\\u4f60\\u597d");
  assert.equal(unicodeEscape("abc123"), "abc123");
});

test("normalizeMotdText 去掉 motd= 前缀并转换字面换行", () => {
  assert.equal(normalizeMotdText("motd=§aA\\n§bB"), "§aA\n§bB");
  assert.equal(normalizeMotdText("§aA\\n§bB"), "§aA\n§bB");
});

test("motdToExportText 将换行转回字面 \\n", () => {
  assert.equal(motdToExportText("§aA\n§bB"), "§aA\\n§bB");
});

test("存储格式与编辑格式可无损往返", () => {
  const stored = "motd=§6§lTitle§r\\n§7Second Line";
  const roundtrip = motdToExportText(normalizeMotdText(stored));
  assert.equal(roundtrip, stored.replace(/^motd=/i, ""));
});

test("颜色映射覆盖全部 16 个代码", () => {
  assert.equal(Object.keys(MOTD_COLOR_MAP).length, 16);
  for (const code of "0123456789abcdef") {
    assert.ok(MOTD_COLOR_MAP[code], `missing color ${code}`);
  }
});
