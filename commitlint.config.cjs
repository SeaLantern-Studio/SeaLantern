const isPullRequestSummaryCommit = (message) => {
  const lines = message.split("\n");
  const header = lines[0]?.trim() ?? "";
  if (!/\(#\d+\)$/.test(header)) {
    return false;
  }

  return lines.slice(1).some((line) => line.trim().startsWith("* "));
};

module.exports = {
  extends: ["@commitlint/config-conventional"],
  ignores: [(message) => message.startsWith("Merge "), isPullRequestSummaryCommit],
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
