export type DiffLineType = "context" | "addition" | "deletion" | "omitted";

export interface DiffLine {
  type: DiffLineType;
  leftNumber: number | null;
  rightNumber: number | null;
  text: string;
}

function getNormalizedLines(text: string) {
  const normalized = text.replace(/\r\n/g, "\n");
  const lines = normalized.split("\n");

  if (lines.length > 0 && lines[lines.length - 1] === "") {
    lines.pop();
  }

  return lines;
}

export function buildDiffLines(originalText: string, targetText: string): DiffLine[] {
  const originalLines = getNormalizedLines(originalText);
  const targetLines = getNormalizedLines(targetText);
  const dp = Array.from({ length: originalLines.length + 1 }, () =>
    Array<number>(targetLines.length + 1).fill(0),
  );

  for (let i = originalLines.length - 1; i >= 0; i -= 1) {
    for (let j = targetLines.length - 1; j >= 0; j -= 1) {
      if (originalLines[i] === targetLines[j]) {
        dp[i][j] = dp[i + 1][j + 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i + 1][j], dp[i][j + 1]);
      }
    }
  }

  const lines: DiffLine[] = [];
  let i = 0;
  let j = 0;
  let leftNumber = 1;
  let rightNumber = 1;

  while (i < originalLines.length && j < targetLines.length) {
    if (originalLines[i] === targetLines[j]) {
      lines.push({
        type: "context",
        leftNumber,
        rightNumber,
        text: originalLines[i],
      });
      i += 1;
      j += 1;
      leftNumber += 1;
      rightNumber += 1;
      continue;
    }

    if (dp[i + 1][j] >= dp[i][j + 1]) {
      lines.push({
        type: "deletion",
        leftNumber,
        rightNumber: null,
        text: originalLines[i],
      });
      i += 1;
      leftNumber += 1;
    } else {
      lines.push({
        type: "addition",
        leftNumber: null,
        rightNumber,
        text: targetLines[j],
      });
      j += 1;
      rightNumber += 1;
    }
  }

  while (i < originalLines.length) {
    lines.push({
      type: "deletion",
      leftNumber,
      rightNumber: null,
      text: originalLines[i],
    });
    i += 1;
    leftNumber += 1;
  }

  while (j < targetLines.length) {
    lines.push({
      type: "addition",
      leftNumber: null,
      rightNumber,
      text: targetLines[j],
    });
    j += 1;
    rightNumber += 1;
  }

  return lines;
}
