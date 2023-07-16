import { $, cd } from 'zx';

export const lsIgnoredFiles = async (dir: string) => {
    await cd(dir);
    const files =
        await $`git ls-files --ignored --exclude-standard --others --directory --no-empty-directory`;
    return String(files).split('\n');
};
