Integrating **Svelte 5** with **Actix Web** (Rust) using a single-template approach provides a high-performance, lightweight architecture. By avoiding full template engines like Tera, we gain absolute control over the HTML delivery and keep the backend logic minimal.

This updated guide follows your requested Python-like `render_svelte` pattern and utilizes **TypeScript** with the latest **Svelte 5 Runes**.

---

### 1. The Actix Web Backend (Rust)

We'll implement the `render_svelte` helper. It performs a simple string replacement on our base HTML file.

**Cargo.toml:**

```toml
[dependencies]
actix-web = "4.0"
actix-files = "0.6"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

```

**Main.rs with `render_svelte` Helper:**

```rust
use actix_web::{get, HttpResponse, Responder};
use serde::Serialize;
use std::fs;

// The Python-like helper function
fn render_svelte<T: Serialize>(component: &str, title: Option<&str>, extra_data: Option<T>) -> impl Responder {
    let mut html = fs::read_to_string("static/index.html")
        .expect("Missing static/index.html - ensure you ran npm run build");

    let page_title = title.unwrap_or("Svelte 5 App");
    let json_data = serde_json::to_string(&extra_data).unwrap_or_else(|_| "null".into());

    // Inject values into our placeholders
    html = html.replace("{{title}}", page_title);
    html = html.replace("{{component}}", component);
    html = html.replace("{{extra_data}}", &json_data);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[derive(Serialize)]
struct UserData {
    username: String,
    role: String,
}

#[get("/dashboard")]
async fn dashboard() -> impl Responder {
    let data = UserData { 
        username: "Rustacean".into(), 
        role: "Admin".into() 
    };
    
    // Usage matches your requested pattern
    render_svelte("Dashboard", Some("Admin Panel"), Some(data))
}

```

---

### 2. The Single HTML Template (`static/index.html`)

This is the "master shell" that Vite populates with your CSS and JS links. We use double-curly braces for our custom replacements.

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{title}}</title>
    <link rel="stylesheet" href="/static/bundle.css">
</head>
<body>
    <div id="svelte-app"></div>

    <script defer src="/static/bundle.js" 
            id="svelte-entry" 
            data-component="{{component}}" 
            data-title="{{title}}"
            data-json='{{extra_data}}'>
    </script>
</body>
</html>

```

---

### 3. Svelte 5 Entry Point (`main.ts`)

Using TypeScript and the Svelte 5 `mount` function.

```typescript
import { mount } from 'svelte';
import Root from './App.svelte';

const entry = document.getElementById("svelte-entry") as HTMLElement;

// Extract values from the HTML attributes replaced by Rust
const componentName = entry.dataset.component || 'Index';
const title = entry.dataset.title || 'Home';
const extraData = entry.dataset.json ? JSON.parse(entry.dataset.json) : null;

mount(Root, {
    target: document.getElementById("svelte-app")!,
    props: {
        componentName,
        title,
        extraData
    }
});

```

---

### 4. Root Component (`App.svelte`)

In Svelte 5, we use **Runes**. We replace `<svelte:component>` with a dynamic capitalized variable.

```svelte
<script lang="ts">
    import Index from './pages/Index.svelte';
    import Dashboard from './pages/Dashboard.svelte';

    // Svelte 5: Define props via $props rune
    let { componentName, title, extraData } = $props<{
        componentName: string,
        title: string,
        extraData: any
    }>();

    // Mapping string names to components
    const components: Record<string, any> = {
        Index,
        Dashboard
    };

    // Capitalized variable allows us to use it as a tag
    const ActiveComponent = components[componentName] || Index;
</script>

<header>
    <h1>{title}</h1>
</header>

<main>
    <ActiveComponent {extraData} />
</main>

<style>
    main { padding: 1rem; }
</style>

```

---

### 5. Page Component (`Dashboard.svelte`)

Showcasing the reactive `$state` and `$derived` runes.

```svelte
<script lang="ts">
    let { extraData } = $props<{ extraData: any }>();

    // Svelte 5: Modern reactivity
    let clicks = $state(0);
    let doubleClicks = $derived(clicks * 2);
</script>

<div class="card">
    <p>Logged in as: <strong>{extraData?.username}</strong></p>
    <p>Role: {extraData?.role}</p>
    
    <button onclick={() => clicks++}>
        Clicks: {clicks} (Double: {doubleClicks})
    </button>
</div>

<style>
    .card { border: 1px solid #ccc; padding: 1rem; border-radius: 8px; }
</style>

```

---

### 6. Build Setup (`vite.config.ts`)

Configure Vite to output exactly what Actix expects in the `static/` folder.

```typescript
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  build: {
    outDir: '../static',
    emptyOutDir: false, // Prevents deleting the index.html template
    lib: {
      entry: './src/main.ts',
      name: 'app',
      formats: ['iife'],
      fileName: () => 'bundle.js'
    }
  }
});

```

### Workflow Summary:

1. **Frontend**: Write components with `$state` and `$props`. Run `npm run build`.
2. **Static**: Vite places `bundle.js` and `bundle.css` into your Rust project's `static/` folder.
3. **Rust**: The `render_svelte` function reads your `index.html`, swaps `{{extra_data}}` for your serialized struct, and sends it to the user.

Would you like me to show you how to set up **Hot Module Replacement (HMR)** so you don't have to rebuild the frontend every time you make a change?