import { bash } from './bash.mjs';
import { Container } from '@dagger.io/dagger';

export const CARGO_BUILD_TARGETS = [
  'x86_64-unknown-linux-gnu',
  'x86_64-unknown-linux-musl',
  'aarch64-unknown-linux-gnu',
  'aarch64-unknown-linux-musl',
  'x86_64-apple-darwin',
  'aarch64-apple-darwin',
  // 'x86_64-pc-windows-gnu'
];

export const installRustTools = async (
  container: Container,
  opts?: { zigbuild?: boolean }
): Promise<Container> => {
  const PATH = await container.envVariable('PATH');
  container = await container
    .withExec(
      bash(
        'apt-get -qq update && apt-get install -y -q build-essential curl jq'
      )
    )
    // Install "rust" runtime for build language binding.
    .withExec(bash('curl https://sh.rustup.rs -sSf | sh -s -- -y'))
    // Add .cargo/bin to PATH
    .withEnvVariable('PATH', `/root/.cargo/bin:${PATH}`)
    .withEnvVariable('CARGO_HOME', '/root/.cargo');

  // SEE: [cargo-zigbuild/Dockerfile at main · rust-cross/cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild/blob/main/Dockerfile)
  if (opts?.zigbuild) {
    const ZIG_VERSION = '0.10.1';
    const zigUrl = `https://ziglang.org/download/${ZIG_VERSION}/zig-linux-$(uname -m)-${ZIG_VERSION}.tar.xz`;
    container = container.withExec(
      bash(
        `curl -L "${zigUrl}" | tar -J -x -C /usr/local && ln -s "/usr/local/zig-linux-$(uname -m)-${ZIG_VERSION}/zig" /usr/local/bin/zig`
      )
    );

    // Install macOS SDKs
    container = container
      .withExec(
        bash(
          'curl -L "https://github.com/phracker/MacOSX-SDKs/releases/download/11.3/MacOSX10.9.sdk.tar.xz" | tar -J -x -C /opt'
        )
      )
      .withExec(
        bash(
          'curl -L "https://github.com/phracker/MacOSX-SDKs/releases/download/11.3/MacOSX11.3.sdk.tar.xz" | tar -J -x -C /opt'
        )
      )
      .withEnvVariable('SDKROOT', '/opt/MacOSX11.3.sdk')
      .withExec(bash(`rustup target add ${CARGO_BUILD_TARGETS.join(' ')}`))
      .withExec(bash('cargo install cargo-zigbuild'));
  }

  container = await container.sync();

  return container;
};
