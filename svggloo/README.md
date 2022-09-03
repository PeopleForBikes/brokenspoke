# Svggloo

[![ci](https://github.com/PeopleForBikes/svggloo/actions/workflows/ci.yaml/badge.svg)](https://github.com/PeopleForBikes/svggloo/actions/workflows/ci.yaml)
[![Latest Version](https://img.shields.io/github/v/tag/PeopleForBikes/svggloo?sort=semver&label=version)](https://github.com/PeopleForBikes/svggloo/)
[![License](https://img.shields.io/badge/license-mit-blue.svg)](https://github.com/PeopleForBikes/svggloo/blob/main/LICENSE)
[![Code of Conduct](https://img.shields.io/badge/code_of_conduct-🌐-ff69b4.svg?logoColor=white)](https://github.com/PeopleForBikes/svggloo/blob/main/code-of-conduct.md)

Svggloo is a tool to perform a data-merge operation in an SVG file.

## Quickstart

The tool expects the following inputs:

- an SVG file to use as a template (see template details in the dedicated
  section below)
- a data file with the same name as the template, but with a `.csv` extension.
  Each record in the data file will produce a new output.

Usage:

```bash
svggloo --field country --field state --field city --export \
  examples/quantifier/bike_lane_categories.svg
```

## Specifics

### Template

The template use the jinja2 syntax to perform replacements, therefore all
variables in the template must be surrounded by `{{}}`, for instance `{{name}}`.

### Data file

The data file must be a CSV file.

### SVG Export

The SVG export is done using [inkscape]. If the program is not found ssvggloo
will abort the operation.

[inkscape]: https://inkscape.org/
