export function loadConfig() {
    require('dotenv').config();
    switch (process.env.NODE_ENV) {
        case 'local':
            return require('../../config.local.json');
        case 'test':
        case 'ci':
            return require('../../config.ci.json');
        case 'staging':
            return require('../../config.staging.json');
        default:
            throw new Error(`Invalid NODE_ENV: ${process.env.NODE_ENV}`);
    }
}
