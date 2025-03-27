import { defineConfig } from 'tsup';

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
