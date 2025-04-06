# mllt (My Little Link Tree)

**mllt** is a fast, low-complexity static site generator designed specifically for creating sleek, modern websites with low page counts. Built in Rust with Handlebars templating, it combines raw performance with flexible customization, perfect for personal bio pages, social media hubs, and curated link collections. No markdown conversion, no metadata headers, no complex content management rules - just a simple, no-frills content generator.

[![Example Site Screenshot](/sample.png)](/sample.png)  
*Example mllt-generated site (click to enlarge)*

## Features

- ⚡ **Sub-millisecond template rendering** with Rust-powered engine
- 🎨 **Handlebars templating** with quick, easy-to-learn transclusion system
- ⚙️ **TOML configuration** with CLI overrides
- 📦 **Automatic asset pipeline** with smart incremental copying
- 🔍 **Strict mode** for debugging template variables

**Planned Features:**

- 🔄 **Live reload development server** (via `mllt serve`)
- 🪚**Optimized image manipulation automation** (via `imagemagick`). 
- 📜**Page content transclusion**

## Requirements

- Rust 1.80+ and Cargo (install via [rustup](https://rustup.rs/))

## Installation

```bash
cargo install --git https://github.com/montessquio/mllt
```

## Getting Started

### Basic Usage

1. Create new project:
```bash
mllt new my-site && cd my-site
```

2. Build site:
```bash
mllt build
```

3. Serve locally:
```bash
mllt serve
```

### Project Structure

```txt
my-site/
├── mllt.toml          # Configuration
├── content/           # Main templates (.hbs)
│   └── index.hbs
├── theme/             # Theme partials (.hbs)
│   └── main.hbs
├── assets/            # Static files (css, images)
└── output/            # Generated site (auto-created)
```

## Configuration

### File (`mllt.toml`) 

```toml
[site]
baseURL = "https://yourdomain.com"
publishdir = "./output"    # Output directory
content = "./content"      # Template location
theme = "./theme"          # Partial templates
assets = "./assets"        # Static files
strict = false             # Enable strict variable checks

[params]                   # Custom template variables
title = "My Awesome Links"
social_handles = [
    { platform = "GitHub", url = "https://github.com/you" },
    { platform = "BlueSky", url = "https://bsky.app/you" }
]
```

### CLI Precedence

Command-line flags will always override config file settings:

```bash
mllt build \
    --output ./public \    # Overrides publishdir
    --content ./src   \    # Overrides content directory
    --strict               # Overrides strict mode
```

Full CLI reference:
```bash
mllt build --help
mllt serve --help
```

## Templating Guide

Mllt uses *handlebars* templating; a good overview
of most of Handlebars' feature set can be found at
the [official handlebars.js documentation](https://handlebarsjs.com/guide/#language-features).

### Basic Template (`content/index.hbs`)

```handlebars
{{! Uses theme's main.hbs template }}
{{#theme "main"}}
    {{! Content block, inserted into the corresponding 
        "content" tag in `theme/main.hbs` }}
    <section class="links">
        {{#each params.social_handles}}
            <a href="{{url}}" class="link-button">
                {{platform}}
            </a>
        {{/each}}
    </section>
{{/theme}}
```

### Theme Template (`theme/main.hbs`)

```handlebars
<!DOCTYPE html>
<html>
<head>
    <title>{{params.title}} | {{site.baseURL}}</title>
    <style>{{{_bundled_normalize}}}</style>
</head>
<body>
    <main>
        {{! Content from individual pages }}
        {{{content}}}
    </main>
</body>
</html>
```

### Automatic Variables

There are four variables automatically made available
to all templates:

```text
{{site}}                   # From config.site.baseurl
{{page}}                   # The name of the current page,
                           # e.g. "index" for index.html.
{{params}}                 # From config.params
{{_bundled_normalize}}     # Built-in normalize.css
```

`site` and `params` contain the values you set in your `mllt.toml`
project file (or overrode on the command line, for relevant options).
`page` contains a string corresponding to the path of the currently
rendering template file relative to the content folder root - this
doesn't change even when rendering in theme files. Finally,
`_bundled_normalize` is a string containing a pre-bundled [normalize](https://necolas.github.io/normalize.css/)
distribution, for convenience (put it in a set of \<style\> tags).

## Static Assets

Place files in the `assets/` directory:

```txt
   assets/
   ├── css/
   │   └── custom.css
   └── images/
       └── logo.png
```

Reference in templates:
```handlebars
   <link rel="stylesheet" href="/css/custom.css">
   <img src="/images/logo.png" alt="Logo">
```

Files are automatically copied to output directory on build.

### Strict Mode

By default, the template engine replaces missing variables with
empty strings. If this behavior is not preferable, you can enable
*strict mode* in order to raise an error upon missing variables.

```toml
[site]
strict = true
```

or

```bash
mllt build --strict
```

## Development

### Contributing

1. Fork the repository
2. Create a feature branch:
   ```bash
   git checkout -b feature/amazing-thing
   ```
3. Submit a Pull Request

Please follow existing code style and add tests for new features.

## License

GNU AGPL v3 License - see [LICENSE](LICENSE) for details.
