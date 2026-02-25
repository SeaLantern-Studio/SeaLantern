export type JavaSource = "adoptium" | "openjdk";

export type OsType = "windows" | "mac" | "linux";
export type ArchType = "x64" | "aarch64";

export interface JavaDownloadInfo {
  url: string;
  versionName: string;
  supported: boolean;
  unsupportedReason?: string;
}

export interface SystemInfo {
  os: OsType;
  arch: ArchType;
}

export function detectSystem(): SystemInfo {
  let os: OsType = "windows";
  if (navigator.userAgent.indexOf("Mac") !== -1) os = "mac";
  if (navigator.userAgent.indexOf("Linux") !== -1) os = "linux";

  let arch: ArchType = "x64";
  if (navigator.userAgent.indexOf("aarch64") !== -1 || navigator.userAgent.indexOf("arm64") !== -1) {
    arch = "aarch64";
  }

  return { os, arch };
}

export function isSourceSupported(source: JavaSource, system: SystemInfo): boolean {
  if (source === "openjdk" && system.os === "windows" && system.arch === "aarch64") {
    return false;
  }
  return true;
}

export function getUnsupportedReason(source: JavaSource, system: SystemInfo): string | null {
  if (source === "openjdk" && system.os === "windows" && system.arch === "aarch64") {
    return `不支持的架构: Windows ARM64`;
  }
  return null;
}

export function getAdoptiumDownloadUrl(version: string, system: SystemInfo): string {
  const baseUrl = "https://api.adoptium.net/v3/binary/latest";
  const releaseType = "ga";
  const adoptiumOs = system.os === "mac" ? "mac" : system.os;
  return `${baseUrl}/${version}/${releaseType}/${adoptiumOs}/${system.arch}/jdk/hotspot/normal/eclipse`;
}

type OsArchUrls = { x64: string; aarch64: string };
type OsUrls = { windows: OsArchUrls; mac: OsArchUrls; linux: OsArchUrls };

const OPENJDK_URLS: Record<string, OsUrls> = {
  "17": {
    windows: {
      x64: "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_windows-x64_bin.zip",
      aarch64: "",
    },
    mac: {
      x64: "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_macos-x64_bin.tar.gz",
      aarch64: "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_macos-aarch64_bin.tar.gz",
    },
    linux: {
      x64: "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_linux-x64_bin.tar.gz",
      aarch64: "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_linux-aarch64_bin.tar.gz",
    },
  },
  "21": {
    windows: {
      x64: "https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_windows-x64_bin.zip",
      aarch64: "",
    },
    mac: {
      x64: "https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_macos-x64_bin.tar.gz",
      aarch64: "https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_macos-aarch64_bin.tar.gz",
    },
    linux: {
      x64: "https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_linux-x64_bin.tar.gz",
      aarch64: "https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_linux-aarch64_bin.tar.gz",
    },
  },
};

export function getOpenJdkDownloadUrl(version: string, system: SystemInfo): string | null {
  const versionUrls = OPENJDK_URLS[version] || OPENJDK_URLS["17"];
  const osUrls = versionUrls[system.os] || versionUrls["windows"];
  const url = osUrls[system.arch];
  return url || null;
}

export function getJavaDownloadInfo(version: string, source: JavaSource): JavaDownloadInfo {
  const system = detectSystem();

  if (!isSourceSupported(source, system)) {
    return {
      url: "",
      versionName: "",
      supported: false,
      unsupportedReason: getUnsupportedReason(source, system) || "不支持的平台/架构组合",
    };
  }

  if (source === "adoptium") {
    return {
      url: getAdoptiumDownloadUrl(version, system),
      versionName: `jdk-${version}`,
      supported: true,
    };
  }

  const url = getOpenJdkDownloadUrl(version, system);
  if (!url) {
    return {
      url: "",
      versionName: "",
      supported: false,
      unsupportedReason: `OpenJDK 不支持 ${system.os} ${system.arch}`,
    };
  }

  return {
    url,
    versionName: `jdk-${version}-openjdk`,
    supported: true,
  };
}
