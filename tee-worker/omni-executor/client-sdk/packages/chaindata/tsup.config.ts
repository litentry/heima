import { defineConfig } from 'tsup';
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
