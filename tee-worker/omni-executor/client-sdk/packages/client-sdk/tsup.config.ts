import { defineConfig } from 'tsup';
import { execSync } from 'node:child_process';
import path from 'node:path';

export default defineConfig([
  {
    entry: ['src'],
    outDir: '../../dist/packages/client-sdk',
    format: ['esm', 'cjs'],
    splitting: false,
    sourcemap: true,
    clean: true,
    treeshake: true,
    dts: {
      resolve: true,
    },
  },
]);
