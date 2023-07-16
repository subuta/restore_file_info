import { bash, exportCachedFile } from './lib/bash.mjs';
import { lsIgnoredFiles } from './lib/git.mjs';
import { getArch } from './lib/process.mjs';
import Client, { ClientContainerOpts, connect } from '@dagger.io/dagger';
import path from 'path';
import { packageDirectorySync } from 'pkg-dir';

const ROOT_DIR = path.resolve(packageDirectorySync() || '', '../');

// initialize Dagger client
await connect(
  async (client: Client) => {
    let opts: ClientContainerOpts = {} as ClientContainerOpts;
    const target = process.env.TARGET || `${getArch()}-unknown-linux-musl`;
    if (target === 'aarch64-unknown-linux-musl') {
      opts = { platform: 'linux/arm64' } as ClientContainerOpts;
    }

    const rust = client.container(opts).from('rust:1.71.0-slim-bullseye');

    const registryCache = client.cacheVolume('registry');
    const targetCache = client.cacheVolume('target');

    const gitIgnoredFiles = await lsIgnoredFiles('../');

    const restoreFileInfo = await client
      .host()
      .directory(path.resolve(ROOT_DIR), {
        exclude: [...gitIgnoredFiles, 'build/'],
      });

    const builder = await rust
      .withDirectory('/app', restoreFileInfo)
      .withWorkdir('/app')
      .withEnvVariable('CARGO_HOME', '/root/.cargo')
      .withMountedCache('/root/.cargo/registry', registryCache)
      .withMountedCache('/app/target', targetCache)
      .withExec(bash(`rustup target add ${target}`))
      .withExec(bash(`cargo build --release --target ${target}`))
      // Export "version" as well.
      .withExec(bash(`/app/target/${target}/release/version > version.txt`))
      .sync();

    // Workaround for "cannot retrieve path from cache" error.
    await exportCachedFile(
      builder,
      `/app/target/${target}/release/restore_file_info`,
      `./bin/${target}/restore_file_info`
    );

    await exportCachedFile(
      builder,
      `/app/version.txt`,
      `./bin/${target}/version.txt`
    );
  },
  { LogOutput: process.stderr }
);
