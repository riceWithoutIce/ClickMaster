import { mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
export function projectEnvironment() {
  const locations = {
    TEMP: 'build/tmp', TMP: 'build/tmp', TMPDIR: 'build/tmp',
    npm_config_cache: 'build/cache/npm', CARGO_HOME: 'build/cache/cargo-home',
    CARGO_TARGET_DIR: 'build/cargo', NODE_COMPILE_CACHE: 'build/cache/node',
  };
  const env = { ...process.env };
  for (const [key, value] of Object.entries(locations)) {
    env[key] = resolve(root, value);
    mkdirSync(env[key], { recursive: true });
  }
  // Keep the installed toolchain; remap project and dependency paths in Rust metadata.
  const flags = [`--remap-path-prefix=${root}=<repository-root>`];
  env.CARGO_ENCODED_RUSTFLAGS = flags.join('\x1f');
  delete env.RUSTFLAGS;
  env.CLICKMASTER_DEV_ROOT = resolve(root, 'build/runtime/dev');
  env.CARGO_INCREMENTAL = '0';
  env.CARGO_BUILD_DEP_INFO_BASEDIR = root;
  env.npm_config_update_notifier = 'false';
  env.npm_config_fund = 'false';
  env.npm_config_audit = 'false';
  env.npm_config_logs_max = '0';
  return env;
}
