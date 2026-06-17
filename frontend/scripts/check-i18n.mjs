import fs from "node:fs";
import path from "node:path";

const rootDir = path.resolve(process.cwd(), "src", "language");
const baseLocale = "en-US";
const fallbackLocale = "zh-CN";

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function flattenKeys(value, prefix = "") {
  if (!isObject(value)) {
    return prefix ? [prefix] : [];
  }

  const keys = [];
  for (const [key, child] of Object.entries(value)) {
    const next = prefix ? `${prefix}.${key}` : key;
    if (isObject(child)) {
      keys.push(...flattenKeys(child, next));
    } else {
      keys.push(next);
    }
  }
  return keys;
}

function readLocale(locale) {
  const localeDir = path.join(rootDir, locale);
  const files = fs.readdirSync(localeDir).filter((file) => file.endsWith(".json")).sort();
  const groups = {};

  for (const file of files) {
    const name = path.basename(file, ".json");
    if (name === "language" || name === "languageName") {
      continue;
    }
    groups[name] = JSON.parse(fs.readFileSync(path.join(localeDir, file), "utf8"));
  }

  return groups;
}

const locales = fs
  .readdirSync(rootDir)
  .filter((entry) => fs.statSync(path.join(rootDir, entry)).isDirectory())
  .sort((a, b) => a.localeCompare(b));

const baseGroups = readLocale(baseLocale);
const fallbackGroups = readLocale(fallbackLocale);
const groupNames = Object.keys(baseGroups).sort((a, b) => a.localeCompare(b));

let hasMissing = false;

for (const locale of locales) {
  const currentGroups = readLocale(locale);
  const missingGroups = [];
  const incompleteGroups = [];

  for (const group of groupNames) {
    const baseGroup = baseGroups[group] ?? {};
    const fallbackGroup = fallbackGroups[group] ?? {};
    const currentGroup = currentGroups[group] ?? {};

    const baseKeys = new Set(flattenKeys(baseGroup));
    const fallbackKeys = new Set(flattenKeys(fallbackGroup));
    const expectedKeys = new Set([...baseKeys, ...fallbackKeys]);
    const currentKeys = new Set(flattenKeys(currentGroup));

    if (!currentGroups[group] || currentKeys.size === 0) {
      missingGroups.push(group);
      continue;
    }

    const missingKeys = [...expectedKeys].filter((key) => !currentKeys.has(key)).sort((a, b) => a.localeCompare(b));
    if (missingKeys.length > 0) {
      incompleteGroups.push(`${group} (${currentKeys.size}/${expectedKeys.size})`);
    }
  }

  if (missingGroups.length === 0 && incompleteGroups.length === 0) {
    continue;
  }

  hasMissing = true;
  console.log(`${locale}`);
  if (missingGroups.length > 0) {
    console.log(`  missing groups: ${missingGroups.join(", ")}`);
  }
  if (incompleteGroups.length > 0) {
    console.log(`  incomplete groups: ${incompleteGroups.join(", ")}`);
  }
}

if (!hasMissing) {
  console.log("All locales are structurally complete.");
}
