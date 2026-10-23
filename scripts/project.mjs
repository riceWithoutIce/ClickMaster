import { spawn, spawnSync } from 'node:child_process';
import { root, projectEnvironment } from './environment.mjs';
import { resolve, relative, isAbsolute } from 'node:path';

const env = projectEnvironment();
const command = process.argv[2];
if (['desktop','build','test','typecheck'].includes(command)) {
 const check=spawnSync(process.execPath,[resolve(root,'scripts/version.mjs'),'--check'],{cwd:root,env,stdio:'inherit',windowsHide:true});
 if(check.status!==0)process.exit(check.status??1);
}
if (command === 'test') {
  const testRoot = resolve(process.env.CLICKMASTER_TEST_ROOT ?? resolve(root, 'build/verification/TASK-002/coder/tests'));
  const local = relative(root, testRoot);
  if (!local || local.startsWith('..') || isAbsolute(local)) throw new Error('Test output must remain in this project');
  env.CLICKMASTER_TEST_ROOT = testRoot;
}
const programs = {
  typecheck: [process.execPath, [resolve(root, 'node_modules/vue-tsc/bin/vue-tsc.js'), '--noEmit']],
  build: [process.execPath, [resolve(root, 'node_modules/vite/bin/vite.js'), 'build']],
  frontend: [process.execPath, [resolve(root, 'node_modules/vite/bin/vite.js')]],
  dev: [process.execPath, [resolve(root, 'node_modules/@tauri-apps/cli/tauri.js'), 'dev', '--no-watch']],
  desktop: [process.execPath, [resolve(root, 'node_modules/@tauri-apps/cli/tauri.js'), 'build', '--no-bundle', '--', '--locked']],
  test: ['cargo', ['test', '--release', '--locked', '--manifest-path', 'src-tauri/Cargo.toml']],
  'test-targets': [process.execPath, [resolve(root, 'scripts/test-target-filter.mjs')]],
};
if (!programs[command]) throw new Error('Unknown project command');
const [program, args] = programs[command];
const child = spawn(program, args, { cwd: root, env, stdio: 'inherit', windowsHide: true });
child.on('error', () => { console.error('Project tool could not start. Check the existing toolchain.'); process.exitCode = 1; });
child.on('exit', code => {
  if (code !== 0 || command !== 'desktop') { process.exitCode = code ?? 1; return; }
  // Verify compiled Windows resources, not merely the PNG embedded by Rust/frontend.
  const check = spawn(process.execPath, [
    resolve(root, 'scripts/verify-pe-icon.mjs'),
    resolve(env.CARGO_TARGET_DIR, 'release/clickmaster.exe'),
    resolve(root, 'src-tauri/icons/icon.ico'),
  ], { cwd: root, env, stdio: 'inherit', windowsHide: true });
  check.on('error', () => { console.error('PE icon verification could not start.'); process.exitCode = 1; });
  check.on('exit', result => { process.exitCode = result ?? 1; });
});
