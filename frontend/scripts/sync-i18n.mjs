import fs from "node:fs";
import path from "node:path";

const rootDir = path.resolve(process.cwd(), "src", "language");
const baseLocale = "en-US";
const fallbackLocale = "zh-CN";
const localeFallbacks = {
  "zh-CN": ["en-US"],
  "zh-TW": ["zh-CN", "en-US"],
};

const keepChineseInheritedLocales = new Set(["zh-CN", "zh-TW", "ja-JP"]);

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function clone(value) {
  if (Array.isArray(value)) {
    return value.map((item) => clone(item));
  }
  if (isObject(value)) {
    return Object.fromEntries(Object.entries(value).map(([key, child]) => [key, clone(child)]));
  }
  return value;
}

function mergeMissing(target, source) {
  if (!isObject(source)) {
    return target === undefined ? clone(source) : target;
  }

  if (!isObject(target)) {
    return target === undefined ? clone(source) : target;
  }

  const next = clone(target);
  for (const [key, value] of Object.entries(source)) {
    next[key] = mergeMissing(next[key], value);
  }
  return next;
}

function replaceInheritedChinese(target, englishSource, chineseSource) {
  if (typeof target === "string") {
    if (
      typeof englishSource === "string" &&
      typeof chineseSource === "string" &&
      target === chineseSource &&
      englishSource !== chineseSource
    ) {
      return englishSource;
    }
    return target;
  }

  if (!isObject(target)) {
    return target;
  }

  const next = clone(target);
  const keys = new Set([
    ...Object.keys(target),
    ...Object.keys(isObject(englishSource) ? englishSource : {}),
    ...Object.keys(isObject(chineseSource) ? chineseSource : {}),
  ]);

  for (const key of keys) {
    next[key] = replaceInheritedChinese(
      next[key],
      isObject(englishSource) ? englishSource[key] : undefined,
      isObject(chineseSource) ? chineseSource[key] : undefined,
    );
  }

  return next;
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeJson(filePath, value) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function getLocaleDirs() {
  return fs
    .readdirSync(rootDir)
    .filter((entry) => fs.statSync(path.join(rootDir, entry)).isDirectory())
    .toSorted((a, b) => a.localeCompare(b));
}

function getGroupFiles(locale) {
  const localeDir = path.join(rootDir, locale);
  return fs
    .readdirSync(localeDir)
    .filter((file) => file.endsWith(".json") && file !== "language.json")
    .toSorted((a, b) => a.localeCompare(b));
}

const locales = getLocaleDirs();
const baseFiles = getGroupFiles(baseLocale);
const zhFiles = new Set(getGroupFiles(fallbackLocale));
const groupFiles = Array.from(new Set([...baseFiles, ...zhFiles])).toSorted((a, b) =>
  a.localeCompare(b),
);

for (const locale of locales) {
  const localeDir = path.join(rootDir, locale);
  const fallbackChain = (
    localeFallbacks[locale] ??
    (locale === baseLocale ? [fallbackLocale] : [baseLocale, fallbackLocale])
  ).filter((candidate, index, array) => candidate !== locale && array.indexOf(candidate) === index);

  for (const file of groupFiles) {
    const targetPath = path.join(localeDir, file);
    const target = fs.existsSync(targetPath) ? readJson(targetPath) : {};
    let merged = clone(target);

    for (const fallbackLocaleCode of fallbackChain) {
      const fallbackPath = path.join(rootDir, fallbackLocaleCode, file);
      if (!fs.existsSync(fallbackPath)) {
        continue;
      }
      merged = mergeMissing(merged, readJson(fallbackPath));
    }

    if (!keepChineseInheritedLocales.has(locale)) {
      const englishPath = path.join(rootDir, baseLocale, file);
      const chinesePath = path.join(rootDir, fallbackLocale, file);
      const englishSource = fs.existsSync(englishPath) ? readJson(englishPath) : undefined;
      const chineseSource = fs.existsSync(chinesePath) ? readJson(chinesePath) : undefined;
      merged = replaceInheritedChinese(merged, englishSource, chineseSource);
    }

    writeJson(targetPath, merged);
  }
}

console.log(`Synced locales: ${locales.join(", ")}`);
