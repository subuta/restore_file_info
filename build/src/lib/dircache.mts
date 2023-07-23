import Client, { Container } from '@dagger.io/dagger';
import hasha from 'hasha';
import path from 'path';
import { fs } from 'zx';

type Dir = {
  path: string;
  cargoRegistry?: boolean;
  cargoTarget?: boolean;
  rfi?: boolean;
};

type DirCacheOptions = {
  rustPackageDir?: string;
};

export class DirCache {
  private _path: string;
  private _client: Client;
  private _dirs: Dir[];
  private _options: DirCacheOptions | undefined;
  constructor(
    path: string,
    client: Client,
    dirs: Dir[],
    options?: DirCacheOptions
  ) {
    this._path = path;
    this._client = client;
    this._dirs = dirs;
    this._options = options;
  }

  cacheKey(dir: string): string {
    return hasha(dir, { algorithm: 'md5' });
  }

  dirPath(dir: string): string {
    return `./${this._path}/${dir}`;
  }

  async init() {
    for (const dir of this._dirs) {
      await fs.mkdirp(this.dirPath(this.cacheKey(dir.path)));
    }
  }

  async restore(container: Container): Promise<Container> {
    const cacheDir = this._client.host().directory(this._path);

    // Get last workDir
    const workDir = await container.workdir();
    let mounted = container;
    for (const dir of this._dirs) {
      mounted = mounted.withMountedDirectory(
        dir.path,
        cacheDir.directory(this.cacheKey(dir.path))
      );
      // Try restore file info.
      if (dir.rfi) {
        mounted = mounted.withWorkdir(dir.path).withExec(['restore_file_info']);
      }
    }

    // Restore last work_dir.
    mounted = mounted.withWorkdir(workDir);

    mounted = await mounted.sync();

    return mounted;
  }

  safeRustPackageDir() {
    let packageDir = this._options?.rustPackageDir;
    if (!packageDir) {
      throw new Error(`"rustPackageDir" option not specified.`);
    }
    return packageDir;
  }

  async dump(container: Container): Promise<boolean> {
    for (const dir of this._dirs) {
      // Clean cargo target dir before dump.
      if (dir.cargoTarget) {
        container = container
          .withWorkdir(this.safeRustPackageDir())
          .withExec([
            'restore_file_info',
            'cargo_clean_target_dir',
            '-t',
            dir.path,
          ]);
      }

      // Clean cargo registry dir before dump.
      if (dir.cargoRegistry) {
        container = container
          .withWorkdir(this.safeRustPackageDir())
          .withExec([
            'restore_file_info',
            'cargo_clean_registry',
            '-r',
            dir.path,
          ]);
      }

      // Dump file_info.
      if (dir.rfi) {
        container = container
          .withWorkdir(dir.path)
          .withExec(['restore_file_info', 'dump']);
      }

      await container
        // and dump it to host with rfi csv.
        .directory(dir.path)
        .export(this.dirPath(this.cacheKey(dir.path)));
    }
    return true;
  }
}
