# Anchor Design System — vendored copy

**Version `1.0.0`. Vendored 2026-08-08.** Source: the "Anchor Design System" project, which has no repository, package or URL — this tree **is** the durable artifact now, which is why `docs/product/features/visual-redesign.md` also carries the token *values* in prose. Either copy alone is enough to rebuild the design.

**Nothing imports this yet**, and nothing should until #20 rebuilds the component layer. It is here as a **reference spec**, not as code to ship: Anchor is Svelte and these are React, so the component layer is a hand-rebuild with the JSX as the specification (risk **R13**(a)).

## What is here

- `tokens/{colors,typography,spacing}.css` — the token layer, including the corrected values (`--warning`, `--success`, `--danger`, `--muted`, `--hairline-strong` were all darkened for AA before adoption).
- `styles.css` — base layer.
- `components/{core,forms,navigation,feedback}/` — 12 components, each with a `.d.ts` (prop types) and a `.prompt.md` (intent and usage).

## Three defects found on import

Recorded rather than patched, so this copy stays faithful to what was delivered and the fixes land visibly during #20.

### 1. Typography fetches fonts from the network — and the CSP now blocks it

`tokens/typography.css` opens with `@import url('https://fonts.googleapis.com/css2?…')`.

Two accepted constraints say no. `visual-redesign.md`: *"Fonts must be bundled, not fetched — a Tauri desktop app has no guaranteed network."* And the content-security policy shipped on 2026-08-08 is `default-src 'self'` with `font-src 'self'`, so that request is refused outright.

The failure mode is quiet: no error a user sees, just fallback fonts. **Fix during #20** by bundling the four weights the system actually uses — Familjen Grotesk 600, Hanken Grotesk 400 and 500, JetBrains Mono 500 — and dropping the `@import`.

### 2. Five components hard-code colour values

| Component | Value |
|---|---|
| `Button.jsx`, `Checkbox.jsx`, `Switch.jsx`, `Tooltip.jsx` | `#fff` |
| `Dialog.jsx` | `rgba(26,23,18,0.4)` — the modal scrim |

`visual-redesign.md`: *"Components may not reference raw colour values. Every colour resolves through a semantic token, or the second theme breaks silently. This is the one constraint whose violation is invisible until someone switches themes."*

`#fff` should be `var(--on-primary)`. The scrim is derived from ink, so in dark theme it is a dark wash over a dark surface — the exact silent breakage that constraint predicts.

### 3. Dark theme is unwired, not merely incomplete

Twelve dark values exist — `--canvas-dark`, `--surface-dark`, `--ink-on-dark`, `--accent-fg-dark`, `--success-dark` and the rest — but:

- there is **no theme-switching mechanism** anywhere: no `prefers-color-scheme`, no `[data-theme]`, no class hook; and
- **no component references a single dark token.** The system renders light-only, and the dark values are unused declarations.

This is a structural mismatch with accepted decision **C.2**, which chose *semantic tokens with two value sets* — one role whose value is rebound per theme, so components name roles and never themes. What shipped is two roles per concept with nothing to switch between them, which forces every component to branch on theme: precisely the drift C.2 rejected.

**Fix during #20** by restructuring to one role per concept (`--surface` bound to `#ffffff` in light and `#1f1c16` in dark) rather than renaming at the call site. The *values* are correct and verified; it is the shape that needs changing.

## Also worth knowing

- **`Tag` encodes category by hue alone** and ships no second channel. Anchor's accepted rule forbids colour-only state encoding, so the provenance marks, `Resumed`/`Skipped` and the cluster indicator all need the second channel designed separately — as they were, on 2026-08-08.
- **`Checkbox`'s visual box is 18×18**, under the 24×24 floor on its own. It sits inside a `<label>` that wraps box and text, so the *clickable* region clears the floor — which is what the rule is about.
- **Nothing here targets the 260×90 widget.** The smallest component is a ~40px Toast and `Dialog` has a 360px minimum width. The widget is styled directly from tokens, by design.
