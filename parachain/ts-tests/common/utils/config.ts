import { config as dotenvConfig } from 'dotenv';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export function loadConfig() {
    dotenvConfig();

    let configPath: string;
    switch (process.env.NODE_ENV) {
        case 'local':
            configPath = join(__dirname, '../../config.local.json');
            break;
        case 'test':
        case 'ci':
            configPath = join(__dirname, '../../config.ci.json');
            break;
        case 'prod':
            configPath = join(__dirname, '../../config.prod.json');
            break;
        default:
            throw new Error(`Invalid NODE_ENV: ${process.env.NODE_ENV}`);
    }

    return JSON.parse(readFileSync(configPath, 'utf-8'));
}
