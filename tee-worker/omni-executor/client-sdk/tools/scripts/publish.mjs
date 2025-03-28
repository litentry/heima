/**
 * This is a minimal script to publish your package to "npm".
 * This is meant to be used as-is or customize as you see fit.
 *
 * This script is executed on "dist/path/to/library" as "cwd" by default.
 *
 * You might need to authenticate with NPM before running this script.
 */

import { execSync } from 'child_process';

import devkit from '@nx/devkit';
const { readCachedProjectGraph } = devkit;

function invariant(condition, message) {
  if (!condition) {
    console.error(message);
    process.exit(1);
  }
}

// Executing publish script: node path/to/publish.mjs {name} --tag {tag}
// Default "tag" to "next" so we won't publish the "latest" tag by accident.
const [, , name, tag = 'next'] = process.argv;
console.log(name)
const graph = readCachedProjectGraph();
const project = graph.nodes[name];

invariant(project, `Could not find project "${name}" in the workspace. Is the project.json configured correctly?`);

const outputPath = project.data?.targets?.build?.options?.cwd;

invariant(
  outputPath,
  `Could not find "build.options.outDir" of project "${name}". Is project.json configured  correctly?`,
);

process.chdir(outputPath);

// Execute "npm publish" to publish
execSync(`npm publish --access public --tag ${tag}`);
