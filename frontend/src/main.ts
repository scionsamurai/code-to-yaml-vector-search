import { mount } from 'svelte';
import App from './App.svelte';

const entry = document.getElementById("svelte-entry") as HTMLElement;

// Extract variables from the data attributes we set in Rust
const componentName = entry.dataset.component;
const title = entry.dataset.title;
const rawJson = entry.dataset.json;
const extraData = rawJson ? JSON.parse(rawJson) : null;

mount(App, {
    target: document.getElementById("svelte-app")!,
    props: {
        componentName,
        title,
        extraData
    }
});