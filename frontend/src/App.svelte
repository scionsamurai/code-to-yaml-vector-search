<!-- frontend/src/App.svelte -->
<script lang="ts">
    import Index from './pages/Index.svelte';
    import Profile from './pages/Profile.svelte';
    import Home from './pages/Home.svelte'; // Import the Home component
    import AnalyzeQuery from './pages/AnalyzeQuery.svelte'; // Import AnalyzeQuery

    // Svelte 5: Use $props rune
    let { componentName, title, extraData } = $props<{
        componentName: string | undefined;
        title: string | undefined;
        extraData: any;
    }>();

    const components: Record<string, any> = { Index, Profile, Home, AnalyzeQuery }; // Add AnalyzeQuery to the components list
    const Selected = components[componentName] || Index;
</script>

{#if title}
    <h1>{title}</h1>
{/if}

<Selected {extraData} />