# odocgen

An unofficial static documentation generator for Odoo addons' Python code

**Example instance (Odoo Community's master branch):**
[https://sovi-odoo.github.io/odocgen/master](https://sovi-odoo.github.io/odocgen/master)

## Installation

After installing the Rust programming language's toolchain:

```sh
cargo install --git https://github.com/sovi-odoo/odocgen
```

## Usage

```sh
git clone --depth=1 https://github.com/odoo/odoo
# NOTE: This will delete whatever is at 'path/to/output'
odocgen odoo/addons odoo/odoo/addons -o path/to/output -l my-documentation

# Open 'path/to/output/index.html' in your browser
xdg-open path/to/output/index.html
```

Generated documentation will be around 10 mb, be a self-contained static website and may be moved anywhere

## Searching

The algorithm used for the search bar can be summarized to:

```py
matches = []
for symbol in ALL_SYMBOLS:
    found = True

    # Split query by spaces and check all the resulting strings are present in
    # the symbol
    for term in search_bar.get_text().split():
        if term not in symbol:
            found = False

    if found:
        matches.append(symbol)
```

## Performance

Generating documentation for the whole Odoo codebase should take about 3 seconds depending on your computer.
It's mainly bound by disk read/write speed

## Note

This is alpha quality software.
The inheritence detection mechanism will sometimes tag things as "inherited" even if they're not.
Other than that, everything should work.
