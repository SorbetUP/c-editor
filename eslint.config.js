import { createRequire } from 'module'

const requireFromElephantModules = createRequire(
  new URL('./Elephant/node_modules/.eslint-anchor.js', import.meta.url)
)
const eslintJs = requireFromElephantModules('@eslint/js')
const pluginVue = requireFromElephantModules('eslint-plugin-vue')
const pluginHtml = requireFromElephantModules('eslint-plugin-html')
const pluginI18nJson = requireFromElephantModules('eslint-plugin-i18n-json')
const pluginJsonc = requireFromElephantModules('eslint-plugin-jsonc')
const neostandard = requireFromElephantModules('neostandard')
const babelParser = requireFromElephantModules('@babel/eslint-parser')
const globals = requireFromElephantModules('globals')
const { configs: js } = eslintJs

const compatibilityStyleRules = Object.freeze({
  '@stylistic/arrow-spacing': 'off',
  '@stylistic/block-spacing': 'off',
  '@stylistic/brace-style': 'off',
  '@stylistic/comma-spacing': 'off',
  '@stylistic/eol-last': 'off',
  '@stylistic/indent': 'off',
  '@stylistic/key-spacing': 'off',
  '@stylistic/keyword-spacing': 'off',
  '@stylistic/lines-between-class-members': 'off',
  '@stylistic/multiline-ternary': 'off',
  '@stylistic/no-multiple-empty-lines': 'off',
  '@stylistic/object-curly-spacing': 'off',
  '@stylistic/quotes': 'off',
  '@stylistic/semi-spacing': 'off',
  '@stylistic/space-before-blocks': 'off',
  '@stylistic/space-before-function-paren': 'off',
  '@stylistic/space-infix-ops': 'off',
  curly: 'off',
  'no-new': 'off',
  'no-return-await': 'off',
  'no-template-curly-in-string': 'off',
  'no-undef': 'off',
  'no-unneeded-ternary': 'off',
  'no-useless-constructor': 'off',
  'no-useless-escape': 'off',
  'no-void': 'off',
  'one-var': 'off',
  'promise/param-names': 'off'
})

export default [
  {
    ignores: [
      '**/node_modules/**',
      '**/dist/**',
      'build/out/**',
      'build/coverage/**',
      'build/test-results/**',
      'build/android/**/build/**',
      'build/ios/.build/**',
      'Elephant/backend/tauri/target/**',
      '**/target/**',
      'Elephant/backend/tauri/gen/**',
      'Elephant/backend/tauri-plugin-elephant-android-vault/android/.tauri/**',
      'Elephant/backend/tauri-plugin-elephant-android-vault/android/build/**',
      '.cache/elephant-addons/**',
      'addons/**',
      'packs/**',
      '**/.build/**',
      '**/test-results/**',
      '**/playwright-report/**',
      '**/*.min.json',
      'package.json',
      'out/**',
      'blinko-offline/**',
      'Elephant/frontend/src/muya/lib/assets/libs/**',
      'Elephant/frontend/src/muya/lib/parser/marked/urlify.js',
      'Elephant/frontend/src/muya/lib/rust/generated/**',
      'Elephant/frontend/src/renderer/src/assets/symbolIcon/index.js'
    ]
  },

  js.recommended,
  ...neostandard(),
  ...pluginVue.configs['flat/recommended'],

  {
    files: ['**/*.js', '**/*.mjs', '**/*.cjs'],
    plugins: {
      html: pluginHtml
    },
    languageOptions: {
      parser: babelParser,
      parserOptions: {
        requireConfigFile: false,
        ecmaVersion: 'latest',
        sourceType: 'module'
      },
      globals: {
        ...globals.browser,
        MARKTEXT_VERSION_STRING: 'readonly',
        MARKTEXT_VERSION: 'readonly',
        __static: 'readonly'
      }
    },
    rules: {
      '@stylistic/semi': ['error', 'never'],
      '@stylistic/arrow-parens': 'off',
      '@stylistic/no-mixed-operators': 'off',
      'no-return-assign': 'error',
      'no-console': 'off',
      'no-debugger': process.env.NODE_ENV === 'production' ? 'error' : 'off',
      'require-atomic-updates': 'off',
      'prefer-const': 'off',
      'no-prototype-builtins': 'off',
      ...compatibilityStyleRules
    },
    ignores: [
      'node_modules',
      'Elephant/frontend/src/muya/dist/**/*',
      'Elephant/frontend/src/muya/webpack.config.js'
    ]
  },

  {
    files: ['**/*.vue'],
    languageOptions: {
      globals: { ...globals.browser }
    },
    rules: {
      'vue/multi-word-component-names': 'off',
      'vue/require-default-prop': 'off',
      ...compatibilityStyleRules
    }
  },

  {
    files: ['tests/**/*.js'],
    languageOptions: {
      globals: { ...globals.vitest }
    },
    rules: {
      'no-confusing-arrow': 'off',
      ...compatibilityStyleRules
    }
  },

  {
    files: ['Elephant/frontend/src/muya/lib/**/*.js'],
    rules: {
      'no-sequences': 'off',
      'no-unused-expressions': 'off',
      'no-return-assign': 'off',
      eqeqeq: 'warn',
      'no-var': 'warn',
      ...compatibilityStyleRules
    }
  },

  ...pluginJsonc.configs['flat/recommended-with-json'],

  {
    files: ['Elephant/frontend/src/shared/i18n/locales/*.json'],
    plugins: {
      'i18n-json': pluginI18nJson
    },
    rules: {
      'i18n-json/valid-json': 'error',
      'i18n-json/sorted-keys': 'warn',
      'i18n-json/identical-keys': [
        'error',
        {
          filePath: 'Elephant/frontend/src/shared/i18n/locales/en.json'
        }
      ]
    }
  }
]
