export type NextInstanceSection = "players" | "extensions" | "config" | "world";

export interface NextInstancePlaceholderContent {
  eyebrow: string;
  summary: string;
  description: string;
  tracks: Array<{
    title: string;
    description: string;
  }>;
}
