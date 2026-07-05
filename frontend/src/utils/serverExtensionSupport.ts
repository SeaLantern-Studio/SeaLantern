import type { ServerInstance } from "@type/server";
import {
  formatServerCoreTypeLabel,
  serverCoreTypeSupportsPluginExtensions,
} from "@utils/serverCoreLabel";

export function serverSupportsPluginExtensions(server: ServerInstance): boolean {
  return serverCoreTypeSupportsPluginExtensions(server.core_type);
}

export function getPluginUnsupportedReason(server: ServerInstance): string {
  return `当前服务端类型 ${formatServerCoreTypeLabel(server.core_type)} 不支持插件式扩展`;
}
