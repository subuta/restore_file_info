import { bash } from './bash.mjs';
import { getArch } from './process.mjs';
import { Container } from '@dagger.io/dagger';

// restore_file_info: Restore file info for better build performance.
export const installRfi = async (container: Container): Promise<Container> => {
  return container
    .withExec(
      bash(
        `curl -L ${getDownloadUrl()} --output '/usr/local/bin/restore_file_info' && chmod +x /usr/local/bin/restore_file_info`
      )
    )
    .sync();
};

export const getDownloadUrl = () => {
  return `https://github.com/subuta/restore_file_info/releases/download/v0.1.0/${getFilename()}`;
};

export const getFilename = () => {
  if (getArch() === 'x86_64') {
    return 'restore_file_info_x64';
  } else if (getArch() === 'aarch64') {
    return 'restore_file_info_arm64';
  }
  return 'restore_file_info_arm64';
};
