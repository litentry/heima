const nxPreset = require('@nx/jest/preset').default;

process.env.HEIMA_NETWORK = process.env.OMNI_WORKER_ENDPOINT ?? 'heima-local';
process.env.PARACHAIN_NETWORK = process.env.PARACHAIN_ENDPOINT ?? 'ws://localhost:9944';

module.exports = { ...nxPreset };