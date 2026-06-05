import type { LucideIcon } from "@lucide/vue";

export interface PluginMenuItem {
  id: string;
  label: string;
  divider?: boolean;
  danger?: boolean;
  disabled?: boolean;
  icon?: LucideIcon;
}
