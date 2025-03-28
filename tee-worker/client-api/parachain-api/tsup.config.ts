import { defineConfig } from "tsup";
import { execSync } from "node:child_process";

export default defineConfig([
    {
        entry: ["src"],
        outDir: "dist/src",
        format: ["esm", "cjs"],
        splitting: false,
        clean: true,
        onSuccess: async () => {
            console.log("Running tsc for parachain-api...");
            execSync("pnpm tsc -p ./tsconfig.json --emitDeclarationOnly", {
                stdio: "inherit",
            });
            console.log("Running tsc for parachain-api... done");
        },
    },
]);
