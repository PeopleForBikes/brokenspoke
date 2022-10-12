+++
title = "Svggloo"
sort_by = "weight"
weight = 1
template = "docs/page.html"

[extra]
toc = true
+++

Svggloo is a tool to perform a data-merge operation between an SVG file and a
CSV file.

## Goal

The goal is to replace some specific placeholders in the template with specific
values coming from the city rating file.

### Specifics

#### Template

The template use the jinja2 syntax to perform replacements, therefore all
variables in the template must be surrounded by `{{}}`, for instance `{{name}}`.

#### Data file

The data file must be a CSV file.

#### SVG Export

The SVG export is done using [inkscape], [cairosvg], or [svg2pdf]. If the
exporter program is not found svggloo will abort the operation.

## Example

Usage:

```bash
svggloo --field country --field state --field city --export Inkscape \
  examples/brochures/brochure.svg
```

The tool expects the following inputs:

- an SVG file to use as a template (see template details in the dedicated
  section below)
- a data file with the same name as the template, but with a `.csv` extension.
  Each record in the data file will produce a new output.

[cairosvg]: https://cairosvg.org/
[inkscape]: https://inkscape.org/
[svg2pdf]: https://docs.rs/svg2pdf/latest/svg2pdf/
