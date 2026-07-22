import { spawnSync } from "node:child_process";

const commands = [
  ["cargo", ["fmt", "--check"]],
  ["cargo", ["clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"]],
  ["cargo", ["test", "--workspace"]],
  ["pnpm", ["run", "wasm:build"]],
  ["pnpm", ["run", "lint"]],
  ["pnpm", ["run", "test:web"]],
  ["pnpm", ["run", "build"]],
];

for (const [command, args] of commands) {
  console.log(`\n> ${command} ${args.join(" ")}`);
  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: process.platform === "win32",
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
