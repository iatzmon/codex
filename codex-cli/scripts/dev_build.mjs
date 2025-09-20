import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {promises as fs} from 'node:fs';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const cliRoot = path.resolve(scriptDir, '..');
const repoRoot = path.resolve(cliRoot, '..');
const distDir = path.join(cliRoot, 'dist');
const distBinDir = path.join(distDir, 'bin');

async function ensureDir(dir) {
  await fs.mkdir(dir, {recursive: true});
}

async function copyFileIfExists(source, target) {
  try {
    await fs.copyFile(source, target);
  } catch (error) {
    if (error.code === 'ENOENT') {
      return;
    }
    throw error;
  }
}

async function main() {
  await fs.rm(distDir, {recursive: true, force: true});
  await ensureDir(distBinDir);

  const pkgPath = path.join(cliRoot, 'package.json');
  const pkg = JSON.parse(await fs.readFile(pkgPath, 'utf8'));

  await copyFileIfExists(path.join(cliRoot, 'bin', 'codex.js'), path.join(distBinDir, 'codex.js'));
  await copyFileIfExists(path.join(repoRoot, 'README.md'), path.join(distDir, 'README.md'));
  await copyFileIfExists(path.join(cliRoot, 'README.md'), path.join(distDir, 'CLI_README.md'));

  const stagedPkg = {
    name: pkg.name,
    version: pkg.version,
    license: pkg.license,
    bin: pkg.bin,
    type: pkg.type,
    files: pkg.files,
    dependencies: pkg.dependencies ?? {},
  };

  await fs.writeFile(path.join(distDir, 'package.json'), `${JSON.stringify(stagedPkg, null, 2)}\n`, 'utf8');

  console.log(`Staged ${stagedPkg.name}@${stagedPkg.version} in ${path.relative(repoRoot, distDir)}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
