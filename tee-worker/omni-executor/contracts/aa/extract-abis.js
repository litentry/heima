#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Function to find all deployment files
function findDeploymentFiles(baseDir) {
    const files = [];
    
    if (!fs.existsSync(baseDir)) {
        return files;
    }
    
    const entries = fs.readdirSync(baseDir);
    
    for (const entry of entries) {
        const fullPath = path.join(baseDir, entry);
        const stat = fs.statSync(fullPath);
        
        if (stat.isDirectory()) {
            // Check subdirectory for JSON files
            const subFiles = fs.readdirSync(fullPath)
                .filter(f => f.endsWith('.json'))
                .map(f => path.join(fullPath, f));
            files.push(...subFiles);
        } else if (entry.endsWith('.json')) {
            // Support flat structure (backward compatibility)
            files.push(fullPath);
        }
    }
    
    return files;
}

// Find all deployment files
const deploymentDir = path.join(__dirname, 'deployments');
const deploymentFiles = findDeploymentFiles(deploymentDir);

if (deploymentFiles.length === 0) {
    console.log('No deployment files found in:', deploymentDir);
    process.exit(1);
}

console.log(`Found ${deploymentFiles.length} deployment file(s) to process\n`);

// Process each deployment file
for (const deploymentFile of deploymentFiles) {
    console.log(`Processing: ${path.relative(__dirname, deploymentFile)}`);
    
    try {
        const deployment = JSON.parse(fs.readFileSync(deploymentFile, 'utf8'));

        // Update each contract with its ABI
        for (const [contractName, contractDataOrAddress] of Object.entries(deployment.contracts)) {
            try {
                // Handle both old format (address string) and new format (object)
                let contractData;
                if (typeof contractDataOrAddress === 'string') {
                    // Convert old format to new format
                    contractData = { address: contractDataOrAddress };
                    deployment.contracts[contractName] = contractData;
                } else {
                    contractData = contractDataOrAddress;
                }
                // Determine the artifact path based on contract name
                let artifactPath;
                if (contractName === 'TestToken') {
                    // TestToken might have multiple instances, use the generic path
                    artifactPath = path.join(__dirname, 'out', 'TestToken.sol', 'TestToken.json');
                } else {
                    // Map contract names to their file locations
                    const contractMappings = {
                        'EntryPoint': 'EntryPointV1',
                        'EntryPointV1': 'EntryPointV1',
                        'OmniAccountFactory': 'OmniAccountFactoryV1', 
                        'OmniAccountFactoryV1': 'OmniAccountFactoryV1',
                        'OmniAccount': 'OmniAccountV1',
                        'OmniAccountV1': 'OmniAccountV1'
                    };
                    
                    const mappedName = contractMappings[contractName] || contractName;
                    
                    // Try different possible locations
                    const possiblePaths = [
                        path.join(__dirname, 'out', `${mappedName}.sol`, `${mappedName}.json`),
                        path.join(__dirname, 'out', 'src', 'core', `${mappedName}.sol`, `${mappedName}.json`),
                        path.join(__dirname, 'out', 'src', 'accounts', `${mappedName}.sol`, `${mappedName}.json`),
                    ];
                    
                    artifactPath = possiblePaths.find(p => fs.existsSync(p));
                }
                
                if (artifactPath && fs.existsSync(artifactPath)) {
                    const artifact = JSON.parse(fs.readFileSync(artifactPath, 'utf8'));
                    contractData.abi = artifact.abi || [];
                    console.log(`  ✓ Updated ABI for ${contractName}`);
                } else {
                    console.log(`  ⚠ Could not find artifact for ${contractName}`);
                }
            } catch (error) {
                console.error(`  ✗ Error processing ${contractName}:`, error.message);
            }
        }

        // Write updated deployment file
        fs.writeFileSync(deploymentFile, JSON.stringify(deployment, null, 2));
        console.log('  ✅ File updated with ABIs\n');
    } catch (error) {
        console.error(`  ✗ Error processing file:`, error.message);
    }
}

console.log('✨ All deployment artifacts processed');