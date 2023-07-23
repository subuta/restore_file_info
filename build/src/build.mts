import { bash } from './lib/bash.mjs';
import { parseCommonCliOptions } from './lib/cli.mjs';
import { DirCache } from './lib/dircache.mjs';
import { lsIgnoredFiles } from './lib/git.mjs';
import { getArch } from './lib/process.mjs';
import { installRfi } from './lib/restore_file_info.mjs';
import { CARGO_BUILD_TARGETS, installRustTools } from './lib/rust.mjs';
import Client, { connect } from '@dagger.io/dagger';
import { packageDirectorySync } from 'pkg-dir';
import { path } from 'zx';

const ROOT_DIR = path.resolve(packageDirectorySync() || '', '../');

const main = async () => {
  const options = parseCommonCliOptions(
    CARGO_BUILD_TARGETS,
    `${getArch()}-unknown-linux-musl`
  );

  if (!options) {
    return;
  }

  const target = options.target;
  console.log(`Try building target='${target}'`);

  // initialize Dagger client
  await connect(
    async (client: Client) => {
      const dirCache = new DirCache('cache', client, [
        { path: '/root/.cargo/registry' },
        { path: '/app/target', rfi: true },
      ]);

      await dirCache.init();

      const gitIgnoredFiles = await lsIgnoredFiles('../');
      const packages = await client
        .host()
        .directory(path.resolve(ROOT_DIR, './'), {
          exclude: [...gitIgnoredFiles],
        });

      let rust = client.container().from('rust:1.70.0-slim-bullseye');

      // Setup rust tools
      rust = await installRustTools(rust, { zigbuild: true });
      // Install restore_file_info.
      rust = await installRfi(rust);

      // Copy 'packages' dir
      let builder = rust.withDirectory('/app', packages);

      builder = await dirCache.restore(builder);

      let result = builder
        .withWorkdir('/app')
        .withExec(
          bash(
            `cargo zigbuild --release --target ${target} --target-dir /app/target`
          )
        );

      await result
        .file(`/app/target/${target}/release/restore_file_info`)
        .export(`${ROOT_DIR}/dist/${target}/restore_file_info`);

      await dirCache.dump(result);
    },
    { LogOutput: process.stderr }
  );
};

await main();
