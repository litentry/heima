import { defineConfig } from 'tsup';
import { execSync } from 'node:child_process';
import path from 'node:path';

export default defineConfig([
  {
    entry: ['src/index.ts'],
    outDir: '../../dist/packages/client-sdk',
    format: ['esm', 'cjs'],
    splitting: false,
    sourcemap: true,
    clean: true,
    dts: false,
    treeshake: true,
    esbuildOptions(options) {
      options.alias = {
        '@lib': path.resolve(__dirname, 'src/lib'),
        '@requests': path.resolve(__dirname, 'src/lib/requests'),
        '@type-creators': path.resolve(__dirname, 'src/lib/type-creators'),
        '@utils': path.resolve(__dirname, 'src/lib/utils'),
        '@test-utils': path.resolve(__dirname, 'src/lib/test-utils'),
      };
      options.mainFields = ['module', 'main'];
      options.resolveExtensions = ['.ts', '.js', '.json'];
    },
    onSuccess: async () => {
      console.log('ESM/CJS build successful, generating type declarations...');
      try {
        // Clean any existing nested directories
        execSync('rm -rf ../../dist/packages/client-sdk/src ../../dist/packages/client-sdk/packages', {
          stdio: 'inherit',
        });
        // Generate type declarations with explicit output directory
        execSync('pnpm tsc -p ./tsconfig.lib.json --emitDeclarationOnly --outDir ../../dist/packages/client-sdk', {
          stdio: 'inherit',
        });
        console.log('Type declarations generated successfully!');
      } catch (error) {
        console.error('Type declaration generation failed:', error);
      }
    },
  },
]);
