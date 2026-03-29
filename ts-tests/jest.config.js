module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  setupFiles: ['./setup.js'],
  testMatch: ['**/*.test.ts'],
  transform: {
    '^.+\\.ts$': 'ts-jest',
  },
  moduleFileExtensions: ['ts', 'js', 'json', 'node'],
};
