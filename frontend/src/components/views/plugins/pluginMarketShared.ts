import type { MarketPluginInfo } from "@api/plugin";
import { i18n } from "@language";

export type MarketPlugin = MarketPluginInfo & { _path?: string };
export type MarketFeedbackType = "success" | "warning" | "error";
export type MarketPermissionLevel = "critical" | "dangerous" | "normal";

export interface MarketFeedback {
  type: MarketFeedbackType;
  message: string;
}

export interface ValidatedMarketSource {
  url: string;
  custom: boolean;
  host: string;
  protocol: string;
}

export const MARKET_BASE_URL = "https://sealantern-studio.github.io/plugin-market";
export const MARKET_URL_KEY = "sealantern_market_url";

const CRITICAL_PERMS = new Set(["execute_program", "plugin_folder_access"]);
const DANGEROUS_PERMS = new Set(["fs", "network", "server", "console"]);

export function resolveMarketValue(value: Record<string, string> | string | undefined): string {
  if (!value) {
    return "";
  }

  if (typeof value === "string") {
    return value;
  }

  const locale = i18n.getLocale();
  const localeKey = locale.startsWith("zh") ? "zh-CN" : "en-US";
  return value[localeKey] || value["zh-CN"] || Object.values(value)[0] || "";
}

export function getMarketPermissionLevel(perm: string): MarketPermissionLevel {
  if (CRITICAL_PERMS.has(perm)) {
    return "critical";
  }
  if (DANGEROUS_PERMS.has(perm)) {
    return "dangerous";
  }
  return "normal";
}

export function validateMarketUrl(input: string): ValidatedMarketSource | null {
  const trimmed = input.trim();
  if (!trimmed) {
    return null;
  }

  try {
    const parsed = new URL(trimmed);
    const isHttps = parsed.protocol === "https:";
    const isLocalHttp =
      parsed.protocol === "http:" && ["localhost", "127.0.0.1", "::1"].includes(parsed.hostname);
    if (!isHttps && !isLocalHttp) {
      return null;
    }

    parsed.hash = "";
    return {
      url: parsed.toString().replace(/\/$/, ""),
      custom: parsed.toString().replace(/\/$/, "") !== MARKET_BASE_URL,
      host: parsed.host,
      protocol: parsed.protocol,
    };
  } catch {
    return null;
  }
}

export function resolveMarketNetworkHint(message: string): string {
  const text = message.toLowerCase();
  const looksLikeNetworkIssue =
    text.includes("download") ||
    text.includes("fetch") ||
    text.includes("network") ||
    text.includes("timeout") ||
    text.includes("proxy") ||
    text.includes("连接") ||
    text.includes("请求") ||
    text.includes("下载");

  if (!looksLikeNetworkIssue) {
    return "";
  }

  const isProxyRefused =
    text.includes("127.0.0.1:9") ||
    text.includes("actively refused") ||
    text.includes("connection refused") ||
    text.includes("proxyconnect") ||
    text.includes("proxy connect") ||
    text.includes("无法连接") ||
    text.includes("积极拒绝");

  if (isProxyRefused) {
    return i18n.t("market.network_hint_proxy");
  }
  if (text.includes("timed out") || text.includes("timeout") || text.includes("超时")) {
    return i18n.t("market.network_hint_timeout");
  }
  return i18n.t("market.network_hint_check");
}
