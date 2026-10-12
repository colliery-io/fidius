import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightAurora from 'starlight-aurora';
import apiSidebar from './src/content/docs/reference/api/_sidebar.mjs';
import apiRedirects from './redirects.mjs';

// The Pages workflow sets DOCS_SITE and DOCS_BASE. On a pull request and
// locally they are empty, and the site is at /.
const site = process.env.DOCS_SITE || undefined;
const base = process.env.DOCS_BASE || '/';

const PROJECT = 'fidius';
const REPO = 'https://github.com/colliery-io/fidius';

export default defineConfig({
	site,
	base,
	integrations: [
		starlight({
			title: PROJECT,
			description: 'A Rust plugin framework for trait-to-dylib plugin systems',
			favicon: '/assets/logo.png',
			social: [{ icon: 'github', label: 'GitHub', href: REPO }],
			plugins: [starlightAurora({ hyper: true })],
			sidebar: [
				{ label: 'Home', link: '/' },
				{
					label: 'Tutorials',
					items: [
						{ label: 'Your First Plugin', slug: 'tutorials/your-first-plugin' },
						{ label: 'Your First Python Plugin', slug: 'tutorials/python-plugin' },
						{ label: 'Your First WASM Plugin', slug: 'tutorials/your-first-wasm-plugin' },
						{ label: 'Optional Methods', slug: 'tutorials/optional-methods' },
						{ label: 'Signing Plugins', slug: 'tutorials/signing-plugins' },
						{ label: 'Source Packages', slug: 'tutorials/source-packages' },
					],
				},
				{
					label: 'How-to guides',
					items: [
						{ label: 'Write a Host Application', slug: 'how-to/host-application' },
						{ label: 'Build a Production Connector', slug: 'how-to/production-connector' },
						{ label: 'Scaffold a Project', slug: 'how-to/scaffold-project' },
						{ label: 'Test Plugins', slug: 'how-to/test-plugins' },
						{ label: 'Method Metadata', slug: 'how-to/method-metadata' },
						{ label: 'Async Methods', slug: 'how-to/async-methods' },
						{ label: 'Multiple Plugins per Dylib', slug: 'how-to/multiple-plugins-per-dylib' },
						{ label: 'Inspect a Plugin', slug: 'how-to/inspect-plugin' },
						{ label: 'Development Workflow', slug: 'how-to/development-workflow' },
						{ label: 'Package Metadata Schema', slug: 'how-to/create-package-schema' },
						{ label: 'White-Label Interface', slug: 'how-to/white-label-interface' },
						{ label: 'Set Up the WASM Toolchain', slug: 'how-to/wasm-component-toolchain' },
						{ label: 'A WASM Plugin in Python', slug: 'how-to/wasm-python-plugin' },
						{ label: 'A WASM Plugin in JavaScript', slug: 'how-to/wasm-javascript-plugin' },
						{ label: 'A WASM Plugin in Go', slug: 'how-to/wasm-go-plugin' },
						{ label: 'A WASM Plugin in C', slug: 'how-to/wasm-c-plugin' },
					],
				},
				{
					label: 'Reference',
					items: [
						{ label: 'CLI', slug: 'reference/cli' },
						{ label: 'ABI Specification', slug: 'reference/abi-specification' },
						{ label: 'Errors', slug: 'reference/errors' },
						{ label: 'Package Manifest', slug: 'reference/package-manifest' },
						apiSidebar,
					],
				},
				{
					label: 'Explanation',
					items: [
						{ label: 'Architecture', slug: 'explanation/architecture' },
						{ label: 'Interface Evolution', slug: 'explanation/interface-evolution' },
						{ label: 'Buffer Strategies', slug: 'explanation/buffer-strategies' },
						{ label: 'Streaming (server, client, bidirectional)', slug: 'explanation/streaming' },
						{ label: 'Host Functions (plugin → host callbacks)', slug: 'explanation/host-functions' },
						{ label: 'Configured Instances', slug: 'explanation/configured-instances' },
						{ label: 'Wire Format', slug: 'explanation/wire-format' },
						{ label: 'WASM Component ABI', slug: 'explanation/wasm-component-abi' },
						{ label: 'Capabilities & the WASM Sandbox', slug: 'explanation/wasm-capabilities' },
						{ label: 'Performance (plugins vs microservice)', slug: 'explanation/performance' },
					],
				},
			],
		}),
		apiRedirects(),
	],
});
