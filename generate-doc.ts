const presets = {
	svelte: {
		title: 'Svelte',
		owner: 'sveltejs',
		repo: 'svelte',
		glob: ['**/documentation/docs/**/*.md'],
		prompt: 'Always use Svelte 5 runes. Runes do not need to be imported, they are globals.',
		minimize: {
			removeCodeBlocks: false,
			removeSquareBrackets: false,
			removeParentheses: false,
			normalizeWhitespace: true,
			trim: true
		}
	},
	sveltekit: {
			title: 'SvelteKit',
			owner: 'sveltejs',
			repo: 'kit',
			glob: ['**/documentation/docs/**/*.md'],
			minimize: {
				removeCodeBlocks: false,
				removeSquareBrackets: false,
				removeParentheses: false,
				normalizeWhitespace: true,
				trim: true
			}
		},
	'supabase-js': {
		title: 'Supabase',
		owner: 'supabase',
		repo: 'supabase',
		glob: ['**/*.md']
	},
	effect: {
		title: 'effect',
		owner: 'Effect-TS',
		repo: 'website',
		glob: ['**/content/docs/**/*.md', '**/content/docs/**/*.mdx']
	},
	effect_schema: {
		title: '@effect/schema',
		owner: 'Effect-TS',
		repo: 'effect',
		glob: ['packages/schema/README.md'],
		prompt: 'All Schema functions are now denoted with uppercase (Struct, String, Number etc.)'
	},
	triplit: {
		title: 'Triplit.dev',
		owner: 'aspen-cloud',
		repo: 'triplit',
		glob: ['**/packages/docs/src/pages/**/*.mdx']
	},
	instantdb: {
		title: 'InstantDB',
		owner: 'instantdb',
		repo: 'instant',
		glob: ['**/client/www/pages/docs/**/*.md']
	},
	'clerk-sveltkit': {
		title: 'Clerk adapter for SvelteKit',
		owner: 'markjaquith',
		repo: 'clerk-sveltekit',
		glob: ['**/README.md']
	},
	'shadcn-svelte': {
		title: 'Shadcn Svelte',
		owner: 'huntabyte',
		repo: 'shadcn-svelte',
		glob: ['**/sites/docs/src/content/**/*.md']
	}
};

async function fetchMarkdownFiles({ owner, repo, glob }: any): Promise<string[]> {
	const contents: string[] = [];
	const branch = (owner === 'sveltejs' && (repo === 'svelte' || repo === 'kit')) || (owner === 'huntabyte' && repo === 'shadcn-svelte') || (owner === 'Effect-TS' && (repo === 'website' || repo === 'effect')) || (owner === 'instantdb' && repo === 'instant') || (owner === 'markjaquith' && repo === 'clerk-sveltekit') ? 'main' : 'master';
	const apiUrl = `https://api.github.com/repos/${owner}/${repo}/git/trees/${branch}?recursive=1`;

	try {
		const response = await fetch(apiUrl);
		if (!response.ok) {
			throw new Error(`Failed to fetch file list from GitHub API: ${response.status}`);
		}
		const data = await response.json();
        const { minimatch } = await import('https://esm.sh/minimatch@9.0.3') as any;
		const files = data.tree
			.filter((item: any) => item.type === 'blob' && glob.some((pattern: string) => {
        return minimatch(item.path, pattern);
      }))
			.map((item: any) => item.path);

		for (const filePath of files) {
			const rawUrl = `https://raw.githubusercontent.com/${owner}/${repo}/${branch}/${filePath}`;
			try {
				const fileResponse = await fetch(rawUrl);
				if (!fileResponse.ok) {
					throw new Error(`Failed to fetch ${rawUrl}: ${fileResponse.statusText}`);
				}
				const content = await fileResponse.text();
				contents.push(content);
			} catch (error: any) {
				console.error(`Error fetching ${rawUrl}: ${error.message}`);
			}
		}
	} catch (error: any) {
		console.error(`Error fetching file list: ${error.message}`);
	}

	return contents;
}

async function fetchAndProcessMarkdown(preset: any): Promise<string> {
	const files = await fetchMarkdownFiles(preset);
	return files.join('\n\n---\n\n');
}

function minimizeContent(content: string, options: any): string {
	const defaultOptions = {
		normalizeWhitespace: true,
		removeCodeBlocks: true,
		removeSquareBrackets: true,
		removeParentheses: true,
		trim: true
	};
	const settings = options ? { ...defaultOptions, ...options } : defaultOptions;

	let minimized = content;

	if (settings.normalizeWhitespace) {
		minimized = minimized.replace(/\s+/g, ' ');
	}

	if (settings.removeCodeBlocks) {
		minimized = minimized.replace(/```[\s\S]*?```/g, '');
	}

	if (settings.removeSquareBrackets) {
		minimized = minimized.replace(/\[.*?\]/g, '');
	}

	if (settings.removeParentheses) {
		minimized = minimized.replace(/\(.*?\)/g, '');
	}

	if (settings.trim) {
		minimized = minimized.trim();
	}

	return minimized;
}

async function getCachedOrFetchMarkdown(preset: any): Promise<string> {
	const content = await fetchAndProcessMarkdown(preset);
    if (preset.minimize) {
        return minimizeContent(content, preset.minimize);
    }
	return content;
}

async function main() {
	const presetName = Deno.args[0];
    console.log('Preset name:', presetName);
	if (!presetName) {
		console.error('Please provide a preset name as a command-line argument.');
		Deno.exit(1);
	}

	if (!(presetName in presets)) {
		console.error(`Invalid preset: ${presetName}`);
		Deno.exit(1);
	}

	const preset = presets[presetName];
	try {
        await Deno.mkdir('./docs', { recursive: true });
		const content = await getCachedOrFetchMarkdown(preset);
		await Deno.writeFile(`./docs/${presetName}.txt`, new TextEncoder().encode(content));
		console.log(`Successfully wrote content to ./docs/${presetName}.txt`);
	} catch (e: any) {
		console.error(`Error processing preset ${presetName}: ${e.message}`);
		Deno.exit(1);
	}
}

main();