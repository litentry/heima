import { defineConfig } from 'tsup';
import { execSync } from 'child_process';
export default defineConfig([
  {
    entry: ['src'],
    outDir: './dist',
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
