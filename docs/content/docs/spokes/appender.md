+++
title = "Appender"
sort_by = "weight"
weight = 1
template = "docs/page.html"

[extra]
toc = true
+++

A tool to append a PDF file to other PDF files.

## Goal

The goal is to append a PDF files to a series of PDF files.

## Example

Our main use case is to add general guidelines to all the city rating brochures.

```bash
appender guidelines.pdf brochure-austin-tx.pdf brochure-boulder-co.pdf
```

This command would append to content of `guidelines.pdf` to
`brochure-austin-tx.pdf` and `brochure-boulder-co.pdf`, i.e. modifying the files
in place.
