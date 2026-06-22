import { defineConfig } from "tsup";
import { execSync } from "node:child_process";

const configs = [
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
];

export default defineConfig(configs);
