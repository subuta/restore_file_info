import { $, cd } from 'zx';

export const lsIgnoredFiles = async (dir: string) => {
  const currentDir = await $`pwd`;

  // pushd
  await cd(dir);

  const files =
    await $`git ls-files --ignored --exclude-standard --others --directory --no-empty-directory`;

  // popd
  await cd(currentDir);

  return String(files).split('\n');
};
