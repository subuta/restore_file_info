import { getArch } from './process.mjs';
import { CARGO_BUILD_TARGETS } from './rust.mjs';
import { cac } from 'cac';

type CommonCliOptions = {
  target: string;
};

export const parseCommonCliOptions = (
  targets: string[],
  defaultTarget: string
): CommonCliOptions | null => {
  const cli = cac();

  cli.option(
    '-t, --target [target]',
    `Target for cargo build allowed targets = [${targets}]`,
    {
      default: defaultTarget,
    }
  );

  // Allow help.
  cli.help();

  const parsed = cli.parse();
  const options = parsed.options;

  if (options['help']) {
    return null;
  }

  if (options['target'] && !CARGO_BUILD_TARGETS.includes(options['target'])) {
    cli.outputHelp();
    process.exit(1);
    return null;
  }

  return options as CommonCliOptions;
};
