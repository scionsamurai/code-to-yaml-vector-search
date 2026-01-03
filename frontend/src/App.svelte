<!-- frontend/src/App.svelte -->
<script lang="ts">
    import type { Component } from 'svelte';
    // 1. Eagerly import all Svelte files in the pages directory
    // This creates an object where keys are paths and values are the component modules
    const modules: Record<string, { default: Component<any> }> = 
        import.meta.glob('./pages/*.svelte', { eager: true });

    // 2. Transform the paths into a clean mapping (e.g., "./pages/Home.svelte" -> "Home")
    const components: Record<string, Component<any>> = {};
    for (const path in modules) {
        const name = path.split('/').pop()?.replace('.svelte', '');
        if (name) {
            components[name] = modules[path].default;
        }
    }

    // 3. Destructure props using Svelte 5 runes
    let { componentName, title, extraData } = $props<{
        componentName: string | undefined;
        title: string | undefined;
        extraData: any;
    }>();

    // 4. Reactive logic to pick the component
    // If componentName is "Home", it looks for components["Home"]
    const Selected = $derived(
        (componentName && components[componentName]) 
        ? components[componentName] 
        : components['Index']
    );
</script>

{#if Selected}
    <Selected {title} {extraData} />
{:else}
    <p>Loading or Component Not Found...</p>
{/if}