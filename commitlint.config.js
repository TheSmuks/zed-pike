// Conventional Commits configuration for zed-pike.
// https://www.conventionalcommits.org/en/v1.0.0/
module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'type-enum': [
      2,
      'always',
      [
        'feat',
        'fix',
        'docs',
        'style',
        'refactor',
        'perf',
        'test',
        'build',
        'ci',
        'chore',
        'revert',
      ],
    ],
    'scope-enum': [
      2,
      'always',
      [
        'bridge',
        'lsp',
        'transport',
        'analysis',
        'daemon',
        'extension',
        'grammar',
        'queries',
        'fixtures',
        'docs',
        'ci',
        'release',
        'open',
        'repo',
      ],
    ],
    'scope-empty': [0],
    'subject-case': [2, 'always', 'lower-case'],
    'header-max-length': [2, 'always', 100],
    'body-leading-blank': [2, 'always'],
    'footer-leading-blank': [2, 'always'],
  },
};