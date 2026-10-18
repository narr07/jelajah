import { spawn } from "node:child_process";
import { createServer } from "node:net";

const DEFAULT_PORT = 3000;

function isPortAvailable(port: number): Promise<boolean> {
	return new Promise((resolve) => {
		const server = createServer();
		server.once("error", () => resolve(false));
		server.once("listening", () => {
			server.close(() => resolve(true));
		});
		server.listen(port, "0.0.0.0");
	});
}

async function findAvailablePort(startPort: number): Promise<number> {
	for (let port = startPort; port < startPort + 100; port++) {
		if (await isPortAvailable(port)) {
			return port;
		}
	}
	throw new Error(`No available port found in range ${startPort}-${startPort + 99}`);
}

async function main(): Promise<void> {
	const port = await findAvailablePort(DEFAULT_PORT);

	const pm = "bun";
	const args = ["run", "tauri", "dev"];

	if (port !== DEFAULT_PORT) {
		console.log(`Port ${DEFAULT_PORT} is busy, using port ${port}`);
		const tauriConfig = JSON.stringify({
			build: {
				devUrl: `http://localhost:${port}`,
				beforeDevCommand: `${pm} run dev --port ${port}`
			}
		});
		const escapedConfig = process.platform === "win32"
			? `"${tauriConfig.replace(/"/g, '\\"')}"`
			: tauriConfig;

		args.push("--config", escapedConfig);
	}

	const tauri = spawn(pm, args, {
		stdio: "inherit",
		shell: process.platform === "win32",
		cwd: process.cwd()
	});

	tauri.on("close", (code) => {
		process.exit(code ?? 0);
	});
}

main();
