# Zed Docs

Zed documentation publishes to https://zed.dev/docs automatically after every push to the main branch.

To preview changes locally, install mdBook version 0.4.40. Generate the action metadata before serving the book:

```sh
script/generate-action-metadata
mdbook serve docs
```

The generation script creates an action manifest at `crates/docs_preprocessor/actions.json`. The preprocessor uses this file to validate keybindings and actions. Run this script whenever actions change.

Use version 0.4.40 of mdBook. Later versions like 0.4.48 cause broken URL behavior.

Format files with Prettier before committing:

```sh
cd docs && pnpm dlx prettier@3.5.0 . --write && cd ..
```

## Preprocessor

The custom mdBook preprocessor in `crates/docs_preprocessor` handles integration with Zed crates. To bypass it, comment out `[preprocessor.zed_docs_preprocessor]` in `book.toml`.

## Images and videos

Link to external assets instead of storing binary files in the repository. Host images and videos on zed.dev or GitHub to keep the Git history small.

## Internal notes

The `docs-proxy` Cloudflare router forwards requests from `zed.dev/docs` to Cloudflare Pages. GitHub Actions manage the deployment via `.github/workflows/deploy_docs.yml` on every push to main.

Static assets in `theme/page-toc.js` and `theme/page-doc.css` handle the table of contents.

## Referencing Keybindings and Actions

Reference keybindings with `{#kb scope::Action}`. This generates a code element for a client-side plugin to process based on the user's platform. Referencing actions directly ensures the documentation remains accurate even if defaults change.

### Keymap Overlays

Use `{#kb:keymap_name scope::Action}` for specific overlays. For example, `{#kb:jetbrains editor::GoToDefinition}` checks the JetBrains keymap first and falls back to the default map if necessary.

### Actions

Render human-readable action names with `{#action scope::Action}`.

## Creating New Templates

Templates modify doc source through regex matching. Find implementation details in `crates/docs_preprocessor/src/main.rs`.

## Consent Banner

The documentation uses a pre-bundled `c15t` package because the pipeline lacks a JS bundler.

Rebuild the bundle manually when updating `c15t`:

```fish
mkdir c15t-bundle && cd c15t-bundle
npm init -y
npm install c15t@<version> esbuild
echo "import { getOrCreateConsentRuntime } from 'c15t'; window.c15t = { getOrCreateConsentRuntime };" > entry.js
npx esbuild entry.js --bundle --format=iife --minify --outfile=c15t@<version>.js
cp c15t@<version>.js ../theme/c15t@<version>.js
cd .. && rm -rf c15t-bundle
```

Update the filename in `book.toml` after generating the new bundle.

## Postprocessor

The postprocessor adds support for page-specific titles and meta descriptions using front matter.
It wraps the HTML renderer and modifies the head tags.

Define metadata at the top of the markdown file:

```md
---
title: Detailed page title
description: Page-specific description
---

# Editor
```

The postprocessor replaces the default title and a `#description#` marker in the HTML.

Front matter parsing follows these rules:

* Place the block at the top of the file.
* Keep keys and values on a single line.
* Do not use double quotes or multi-line values.
* Use simple ASCII text without unicode characters.

If front matter is missing, the system uses the `default-title` and `default-description`
from `book.toml`.
