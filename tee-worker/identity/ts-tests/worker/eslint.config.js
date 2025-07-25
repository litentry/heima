import js from '@eslint/js';
import tseslint from '@typescript-eslint/eslint-plugin';
import tsparser from '@typescript-eslint/parser';
import globals from 'globals';

export default [
    js.configs.recommended,
    {
        files: ['**/*.ts', '**/*.tsx'],
        languageOptions: {
            parser: tsparser,
            parserOptions: {
                ecmaVersion: 'latest',
                sourceType: 'module',
            },
            globals: {
                ...globals.node,
                ...globals.mocha,
            },
        },
        plugins: {
            '@typescript-eslint': tseslint,
        },
        rules: {
            ...tseslint.configs.recommended.rules,
            /**
        It's a temporary solution, folks. We had no choice but to shut it off,
        because there's just a liiittle bit too much "any" lurking around in the code.
        But fear not, my friends, for this is not the end of the story.
        We shall return, armed with determination and resolve,
        to tackle those "any" types head-on in the near future.
        **/
            '@typescript-eslint/no-explicit-any': 'off',
            '@typescript-eslint/no-non-null-assertion': 'off',
            '@typescript-eslint/no-var-requires': 'off',
            '@typescript-eslint/no-require-imports': 'off',
            '@typescript-eslint/no-unused-vars': 'warn',

            // explanation: https://typescript-eslint.io/rules/naming-convention/
            '@typescript-eslint/naming-convention': [
                'error',
                {
                    selector: 'typeLike',
                    format: ['StrictPascalCase'],
                },
                {
                    selector: 'variable',
                    modifiers: ['const'],
                    format: ['strictCamelCase', 'UPPER_CASE'],
                },
                {
                    selector: 'function',
                    format: ['strictCamelCase', 'StrictPascalCase'],
                },
                {
                    selector: 'parameter',
                    format: ['strictCamelCase'],
                },
            ],
        },
    },
    {
        files: ['**/*.js'],
        languageOptions: {
            globals: {
                ...globals.node,
            },
        },
        rules: {
            '@typescript-eslint/no-var-requires': 'off',
            '@typescript-eslint/no-require-imports': 'off',
        },
    },
];
