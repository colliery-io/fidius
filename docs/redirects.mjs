// The old MkDocs site served the API reference at api/rust/<module>/. It is
// now at reference/api/rust/<module>/. After the build, pageRedirects writes
// a page at each old address that sends the reader to the new one.
// fidius-guest is new in the API reference: it has no old address.
import pageRedirects from 'starlight-aurora/redirects';

const API = 'reference/api/';

// The old API pages that keep their module path.
const SAME = [
	'fidius',
	'fidius-cli/commands',
	'fidius-cli/python_stub',
	'fidius-core',
	'fidius-core/async_runtime',
	'fidius-core/host_registry',
	'fidius-core/package',
	'fidius-core/registry',
	'fidius-host',
	'fidius-host/arch',
	'fidius-host/arena',
	'fidius-host/client_stream',
	'fidius-host/error',
	'fidius-host/executor',
	'fidius-host/executor/body_tee',
	'fidius-host/executor/cdylib',
	'fidius-host/executor/name_lookup',
	'fidius-host/executor/python',
	'fidius-host/executor/wasm',
	'fidius-host/handle',
	'fidius-host/host',
	'fidius-host/host_import',
	'fidius-host/loader',
	'fidius-host/package',
	'fidius-host/signing',
	'fidius-host/stream',
	'fidius-host/types',
	'fidius-macro',
	'fidius-macro/host_interface',
	'fidius-macro/impl_macro',
	'fidius-macro/interface',
	'fidius-macro/ir',
	'fidius-macro/wit',
	'fidius-test',
	'fidius-test/dylib',
	'fidius-test/signing',
	'fidius-test/stream',
];

// Modules that moved from fidius-core to fidius-guest.
const MOVED = ['descriptor', 'error', 'hash', 'status', 'wire'];

export default function apiRedirects() {
	return pageRedirects({
		...Object.fromEntries(SAME.map((m) => [`api/rust/${m}/`, `${API}rust/${m}/`])),
		...Object.fromEntries(MOVED.map((m) => [`api/rust/fidius-core/${m}/`, `${API}rust/fidius-guest/${m}/`])),
		// The fidius-cli crate page held only private items. plissken 0.2.0
		// leaves them out, so the page is gone: send it to the API overview.
		'api/rust/fidius-cli/': API,
		[`${API}rust/fidius-cli/`]: API,
	});
}
