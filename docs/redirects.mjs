// The old MkDocs site served the API reference at api/rust/<module>/. It is
// now at reference/api/rust/<module>/. After the build, this writes a page at
// each old address that sends the reader to the new one. Each target is
// relative, so the pages work under any base path. fidius-guest is new in the
// API reference: it has no old address.
import { existsSync } from 'node:fs';
import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { redirectPage } from 'starlight-aurora/mdbook-redirects';

const API = 'reference/api/rust/';

// Modules that moved from fidius-core to fidius-guest.
const MOVED = Object.fromEntries(
	['descriptor', 'error', 'hash', 'status', 'wire'].map((m) => [
		`api/rust/fidius-core/${m}/`,
		`${API}fidius-guest/${m}/`,
	]),
);

export default function apiRedirects() {
	return {
		name: 'fidius-api-redirects',
		hooks: {
			'astro:build:done': async ({ dir, pages, logger }) => {
				const out = fileURLToPath(dir);
				const map = { ...MOVED };
				for (const { pathname } of pages) {
					if (pathname.startsWith(API) && !pathname.startsWith(`${API}fidius-guest/`)) map[pathname.slice('reference/'.length)] = pathname;
				}
				let written = 0;
				for (const [from, to] of Object.entries(map)) {
					const file = join(out, from, 'index.html');
					if (existsSync(file)) continue;
					const up = '../'.repeat(from.split('/').length - 1);
					await mkdir(join(out, from), { recursive: true });
					await writeFile(file, redirectPage(up + to));
					written++;
				}
				logger.info(`${written} redirects from the old API addresses`);
			},
		},
	};
}
