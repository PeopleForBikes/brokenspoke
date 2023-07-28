+++
title = "Brochures"
sort_by = "weight"
weight = 1
template = "docs/page.html"

[extra]
toc = true
+++

The brochure pipelines generates City Ratings brochures based off the City
Ratings Results.

## How it works

The pipeline starts by converting the original city ratings file, to the
shortcode version. Basically, that just means converting the headers to a 2-5
letter code, and performing some minor conversions and validation of the data.
For instance "opportunity" becomes "op" and the value gets converted from
"45.11" to "45".

The next step is to perform a data-merge operation between the shortcode file
which was just created and the brochure template. This step will generate one
SVG file per city. At this point the SVG itself is completely distributable if
only an image is needed.

To make it more portable, the following step will convert all the SVG files into
PDF files.

Then we add extra pages to the brochure. Typically the extra pages contain
instructions to help cities to implement better policies and safer street
designs.

Finally we bundle the brochures by country or state to simplify their
distribution.

<img src="../../../images/pipelines/brochures/brochure_etl.png"
alt="Brochure Pipeline" width="100%">

## Rendering

Here is an example of the SVG file being generated at the beginning of the
process.

<img src="../../../images/pipelines/brochures/united_states-tx-austin.svg"
alt="Brochure Rendering For Austin, TX" width="60%"
style="display: block;margin: 0 auto;">

## Run it locally

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [Inkscape](https://inkscape.org/release/)
- [Montserrat Font](https://fonts.google.com/specimen/Montserrat)
- Dharma Gothic Extended Font

### Run it

This pipeline was written in Rust and can be run locally with the following
commands:

```bash
cd pipelines/brochures
cargo run
```

This will produce the following output:

```bash
2022-10-05T01:53:20.508083Z  INFO brochures: 📁 Creating the output directory...
2022-10-05T01:53:20.508248Z  INFO brochures: ⚙️  Copying the brochure template...
2022-10-05T01:53:20.509215Z  INFO brochures: 🔄 Converting the City Ratings file to a Shortcode file...
2022-10-05T01:53:20.925506Z  INFO brochures: 📄 Generating SVG files...
2022-10-05T01:53:21.346107Z DEBUG brochures: 🗄️  Collecting the generated SVG files...
2022-10-05T01:53:21.348323Z  INFO brochures: 📃 Generating PDF files...
2022-10-05T01:54:14.165587Z  INFO brochures: 📦 Bundling the brochures...
2022-10-05T01:54:26.702501Z  INFO brochures: ✅ Done
```

### Remark

By default, the pipeline is configured to process the latest City Ratings file
with the latest version fo the template.

If another data set needs to be processed, the parameters may need to be
adjusted. This can be done at the top of the script, in the `parameters`
section:

```rs
// Parameters
let format = "v23";
let city_rating_version = "v23.2";
```
