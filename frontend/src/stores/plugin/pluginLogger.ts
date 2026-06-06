function log(
  scope: string,
  level: "debug" | "info" | "warn" | "error",
  message: string,
  details?: unknown,
) {
  const prefix = `[Plugin${scope ? `:${scope}` : ""}] ${message}`;
  if (details === undefined) {
    console[level](prefix);
    return;
  }
  console[level](prefix, details);
}

export const pluginLogger = {
  debug(scope: string, message: string, details?: unknown) {
    log(scope, "debug", message, details);
  },
  info(scope: string, message: string, details?: unknown) {
    log(scope, "info", message, details);
  },
  warn(scope: string, message: string, details?: unknown) {
    log(scope, "warn", message, details);
  },
  error(scope: string, message: string, details?: unknown) {
    log(scope, "error", message, details);
  },
};
