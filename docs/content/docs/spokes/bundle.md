+++
title = "Bundle"
sort_by = "weight"
weight = 1
template = "docs/page.html"

[extra]
toc = true
+++

A tool to bundle brochures together in order to simplify their distribution.

## Goal

The goal is to bundle brochures of the same state or country into in single zip
file to simplify distributing them.

The process also generates a special file named `all.zip` which contains all the
generated brochures in one zip file.

In order to work, the tool expects the brochure names to respect the BNA
convention: `<country>-<state>-<city>.pdf`.

## Example

If the following brochures were generated and put in the same folder:

```bash
.
├── australia-nt-alice_springs.pdf
├── england-eng-london.pdf
├── france-idf-paris.pdf
├── united_states-ca-arcata.pdf
├── united_states-fl-altamonte_springs.pdf
├── united_states-id-meridian.pdf
├── united_states-ks-topeka.pdf
├── united_states-nc-durham.pdf
├── united_states-pa-pittsburgh.pdf
└── united_states-ut-park_city.pdf
```

Running the bundler on this folder would produce the following bundles:

```bash
.
├── all.zip
├── australia.zip
├── england.zip
├── france.zip
└── united_states.zip

```
