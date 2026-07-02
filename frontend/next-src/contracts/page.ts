export const NEXT_PAGE_KINDS = {
  auth: "auth",
  home: "home",
  servers: "servers",
  downloads: "downloads",
  tunnel: "tunnel",
  plugins: "plugins",
  paint: "paint",
  developer: "developer",
  settings: "settings",
} as const;

export type NextPageKind = (typeof NEXT_PAGE_KINDS)[keyof typeof NEXT_PAGE_KINDS];
export type NextProtectedPageKind = Exclude<NextPageKind, "auth">;
export type NextShellPageKind = NextPageKind | "unknown";

export interface NextRoutePageContract {
  pageKind: NextPageKind;
}

export interface NextShellPage {
  kind: NextShellPageKind;
  title: string;
  subtitle?: string;
}

export function isNextPageKind(value: unknown): value is NextPageKind {
  return (
    typeof value === "string" && Object.values(NEXT_PAGE_KINDS).includes(value as NextPageKind)
  );
}

export function resolveNextPageKind(pageKind: unknown): NextShellPageKind {
  return isNextPageKind(pageKind) ? pageKind : "unknown";
}
