module.exports = {
  extends: ["@commitlint/config-conventional"],
  ignores: [(message) => message.startsWith("Merge ")],
  rules: {
    "type-enum": [
      2,
      "always",
      ["feat", "fix", "docs", "style", "refactor", "perf", "test", "chore", "revert", "security"],
    ],
    "type-case": [2, "always", "lower-case"],
    "subject-empty": [2, "never"],
  },
};
