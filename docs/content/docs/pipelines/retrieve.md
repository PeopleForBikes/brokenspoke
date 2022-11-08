+++
title = "Retrieve"
sort_by = "weight"
weight = 1
template = "docs/page.html"

[extra]
toc = true
+++

The retrieve pipeline downloads all the datasets for a list of given cities and
bundle them all in a zip file.

## How it works

The pipeline reads the city details from a City Rating CSV file and uses this
information to automatically find the matching datasets to download.

The pipeline will attempt to download all the available datasets for each city.
As a result, the amount of data to retrieve then bundle can be quite large.

As of 2021, there is about 11GB of datasets available, so depending on you
internet connection it may take a while. For reference, with a 200Mbps
connection, it took 11 min to complete.

<img src="../../../images/pipelines/retrieve/retrieve_etl.png"
alt="Retrieve Pipeline" width="100%">

## Run it locally

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)

### Run it

This pipeline was written in Rust and can be run locally with the following
commands:

```bash
cd pipelines/retrieve
cargo run
```

Output:

```bash
2022-11-08T02:55:01.944039Z  INFO retrieve: 📁 Creating the output directory...
2022-11-08T02:55:01.944299Z  INFO retrieve: 📡 Downloading datasets...
2022-11-08T03:02:17.860077Z  INFO retrieve: 📦 Bundling datasets...
2022-11-08T03:06:09.756437Z  INFO retrieve: ✅ Done
```
