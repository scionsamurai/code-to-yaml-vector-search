// frontend/src/main.ts
import { mount } from 'svelte';
import App from './App.svelte';

const entry = document.getElementById("svelte-entry") as HTMLElement;

// Extract variables from the data attributes we set
const componentName = entry.dataset.component;
const title = entry.dataset.title;
const rawJson = entry.dataset.json;
console.log("rawJson", rawJson);
const extraData = rawJson ? JSON.parse(rawJson) : null;
console.log("title", title);
console.log("extraData", extraData);

mount(App, {
    target: document.getElementById("svelte-app")!,
    props: {
        componentName,
        title,
        extraData
    }
});