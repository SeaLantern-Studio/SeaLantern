export function isMacOSPlatform(): boolean {
  if (typeof navigator === "undefined") return false;
  return /Macintosh|Mac OS X/i.test(navigator.userAgent);
}

export function isWindowsPlatform(): boolean {
  if (typeof navigator === "undefined") return false;
  return /Windows/i.test(navigator.userAgent);
}
