import { Container } from '@dagger.io/dagger';

export const bash = (args: string): string[] => {
  return ['bash', '-c', args];
};

// Workaround for "cannot retrieve path from cache" error.
export const exportCachedFile = async (
  container: Container,
  file: string,
  dest: string,
  tmpDir = '/opt'
): Promise<boolean> => {
  const fileName = file.split('/').pop() || '';
  const tmp = `${tmpDir}/${fileName}`;
  return await container
    .withExec(bash(`cp ${file} ${tmp}`))
    .file(tmp)
    .export(dest);
};
