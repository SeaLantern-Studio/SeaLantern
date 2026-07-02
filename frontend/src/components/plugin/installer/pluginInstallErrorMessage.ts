import { i18n } from "@language";
import type { PluginInstallIssue } from "@type/plugin";

function readStringArg(args: Record<string, unknown> | undefined, key: string): string {
  const value = args?.[key];
  return typeof value === "string" ? value : "";
}

function readStringArrayArg(args: Record<string, unknown> | undefined, key: string): string[] {
  const value = args?.[key];
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === "string")
    : [];
}

export function formatPluginInstallIssue(issue?: PluginInstallIssue | null): string | null {
  if (!issue) {
    return null;
  }

  if (issue.code === "plugins.install.issue.incompatible_sealantern_version") {
    const requiredVersion = readStringArg(issue.args, "required_version");
    const currentVersion = readStringArg(issue.args, "current_version");
    if (requiredVersion && currentVersion) {
      return i18n.t("plugins.install_issue_incompatible_sealantern_version", {
        requiredVersion,
        currentVersion,
      });
    }
  }

  if (issue.code === "plugins.install.issue.requests_trusted_capabilities") {
    const permissions = readStringArrayArg(issue.args, "permissions");
    return i18n.t("plugins.install_issue_requests_trusted_capabilities", {
      permissions: permissions.join(", "),
    });
  }

  if (issue.code === "plugins.install.issue.exceeds_standard_sandbox") {
    const permissions = readStringArrayArg(issue.args, "permissions");
    return i18n.t("plugins.install_issue_exceeds_standard_sandbox", {
      permissions: permissions.join(", "),
    });
  }

  return i18n.te(issue.code) ? i18n.t(issue.code, issue.args) : null;
}
