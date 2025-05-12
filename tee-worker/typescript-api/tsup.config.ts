import { defineConfig } from "tsup";
import { execSync } from "node:child_process";

export default defineConfig([
    {
        entry: ["src/identity"],
        outDir: "dist/identity",
        format: ["esm", "cjs"],
        splitting: false,
        clean: true,
        onSuccess: async () => {
            console.log("Running tsc for identity...");
            execSync("pnpm tsc -p src/identity/tsconfig.json --emitDeclarationOnly", {
                stdio: "inherit",
            });
        },
    },
    {
        entry: ["src/omni"],
        outDir: "dist/omni",
        format: ["esm", "cjs"],
        splitting: false,
        clean: true,
        onSuccess: async () => {
            console.log("Running tsc for omni...");
            execSync("pnpm tsc -p src/omni/tsconfig.json --emitDeclarationOnly", {
                stdio: "inherit",
            });
        },
    },
    {
        entry: ["src/sidechain"],
        outDir: "dist/sidechain",
        format: ["esm", "cjs"],
        splitting: false,
        clean: true,
        onSuccess: async () => {
            console.log("Running tsc for sidechain...");
            execSync("pnpm tsc -p src/sidechain/tsconfig.json --emitDeclarationOnly", {
                stdio: "inherit",
            });
        },
    },
]);
