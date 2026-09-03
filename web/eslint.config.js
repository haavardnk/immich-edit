import js from '@eslint/js';
import ts from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import prettier from 'eslint-config-prettier';
import globals from 'globals';

export default ts.config(
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs.recommended,
  prettier,
  ...svelte.configs.prettier,
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
      parserOptions: {
        projectService: { allowDefaultProject: ['playwright.config.ts'] },
        tsconfigRootDir: import.meta.dirname,
        extraFileExtensions: ['.svelte']
      }
    }
  },
  {
    files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
        projectService: { allowDefaultProject: ['playwright.config.ts'] },
        tsconfigRootDir: import.meta.dirname,
        extraFileExtensions: ['.svelte']
      }
    }
  },
  {
    rules: {
      'svelte/prefer-svelte-reactivity': 'off',
      'svelte/no-navigation-without-resolve': 'off',
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_', caughtErrorsIgnorePattern: '^_' }
      ]
    }
  },
  {
    files: ['**/*.js'],
    ...ts.configs.disableTypeChecked
  },
  {
    ignores: ['build/', '.svelte-kit/', 'package/', 'test-results/', 'playwright-report/']
  }
);
