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
        poolOptions: {
            threads: {
                singleThread: true,
            },
        },
        reporters: ['verbose'],
        retry: 0,
    },
});
