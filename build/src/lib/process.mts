export function getArch(): Error | string {
  switch (process.arch) {
    case 'x64':
      return 'x86_64';
    case 'arm64':
      return 'aarch64';
    default:
      return Error(`Unsupported arch "${process.arch}"`);
  }
}

export function getPlatform(platform?: string): Error | string {
  if (!platform) {
    platform = process.platform;
  }
  switch (platform) {
    case 'darwin':
      return 'apple-darwin';
    case 'win32':
      return 'pc-windows-msvc';
    case 'linux':
      return 'unknown-linux-musl';
    default:
      return Error(`Unsupported platform "${process.platform}"`);
  }
}
