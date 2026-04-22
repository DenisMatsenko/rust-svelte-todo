import { defineConfig } from 'orval';

export default defineConfig({
    myApi: {
        input: {
            target: '../openapi.json',
        },
        output: {
            mode: 'tags-split',
            target: 'src/lib/api/generated',
            client: 'svelte-query',
            httpClient: 'fetch',
            override: {
                mutator: {
                    path: 'src/lib/api/mutator.ts',
                    name: 'customFetch',
                },
            },
            clean: true,
        },
    },
});