import { defineConfig } from 'vitest/config';

export default defineConfig({
    test: {
        environment: 'node',
        include: ['integration-tests/**/*.test.ts'],
        testTimeout: 600000,
        hookTimeout: 60000,
        sequence: {
            concurrent: false,
        },
        globals: false,
        pool: 'threads',
        // Vitest 4.0: poolOptions removed, use top-level options
        maxWorkers: 1,
        isolate: false,
        reporters: ['verbose'],
        retry: 0,
    },
});
