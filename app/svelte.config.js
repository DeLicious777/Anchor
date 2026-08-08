// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: "index.html",
    }),
    // SvelteKit emits exactly one inline bootstrap script, and its contents
    // change every build (it carries a build-specific identifier). `hash` mode
    // makes SvelteKit compute that hash itself and write it into a
    // `<meta http-equiv="content-security-policy">` tag on each build.
    //
    // **This is why the hash is not hard-coded in `tauri.conf.json`.** A policy
    // there would have to be edited by hand after every build, and would fail
    // silently — as a white screen — the first time someone forgot. Tauri's own
    // policy therefore stays permissive on `script-src` and this one is strict;
    // a browser enforces every policy it is given, so the inline script must
    // satisfy *both*, and only the hashed one does.
    csp: {
      mode: "hash",
      directives: {
        "default-src": ["self"],
        "script-src": ["self"],
        // `app.html`'s wrapper carries a `style` attribute, and inline style
        // attributes fall under this directive. Style injection is not the
        // threat CSP is here for; script injection is.
        "style-src": ["self", "unsafe-inline"],
        "img-src": ["self", "data:"],
        "font-src": ["self"],
        // Tauri's IPC. Everything else this app talks to is itself.
        "connect-src": ["self", "ipc:", "http://ipc.localhost"],
        "object-src": ["none"],
        "base-uri": ["self"],
        "frame-ancestors": ["none"],
        "form-action": ["none"],
      },
    },
  },
};

export default config;
