import { defineConfig } from 'tsup';
import { execSync } from 'node:child_process';
import path from 'node:path';

export default defineConfig([
  {
    entry: ['src'],
    outDir: 'dist/packages/client-sdk',
    format: ['esm', 'cjs'],
    splitting: false,
    sourcemap: true,
    clean: true,
    treeshake: true,
    esbuildOptions(options) {
      options.alias = {
        '@lib': path.resolve(__dirname, 'src/lib'),
        '@requests': path.resolve(__dirname, 'src/lib/requests'),
        '@type-creators': path.resolve(__dirname, 'src/lib/type-creators'),
        '@utils': path.resolve(__dirname, 'src/lib/utils'),
        '@test-utils': path.resolve(__dirname, 'src/lib/test-utils'),
      };
    },
    onSuccess: async () => {
      console.log('ESM/CJS build successful!');
      execSync('pnpm tsc -p ./tsconfig.lib.json --emitDeclarationOnly', {
        stdio: 'inherit',
      });
    },
  },
]);
