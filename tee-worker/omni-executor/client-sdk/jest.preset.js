const nxPreset = require('@nx/jest/preset').default;

process.env.HEIMA_NETWORK = 'heima-local'

module.exports = { ...nxPreset };