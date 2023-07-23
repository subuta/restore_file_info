import { Container } from '@dagger.io/dagger';

export const bash = (args: string): string[] => {
  return ['bash', '-c', args];
};

export const ash = (args: string): string[] => {
  return ['ash', '-c', args];
};
