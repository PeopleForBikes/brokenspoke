#!/bin/bash
set +euo pipefail

# Get the paths.
TOP_DIR=$(git rev-parse --show-toplevel)
OUTPUT_DIR="${TOP_DIR}/pipelines/brochures/output"

# Create the output directory.
mkdir -p "${OUTPUT_DIR}"
pushd "${OUTPUT_DIR}"

# Copy the brochure template.
cp "${TOP_DIR}/assets/brochures/brochure.svg" .

# Convert the City Ratings file to a Shortcode file.
cargo run -p spokes --bin shortcodes "${TOP_DIR}/assets/city_ratings/city_ratings_2021_v15.csv"

# Generate SVG files.
cargo run -p svggloo -- --field co --field st --field ci "${OUTPUT_DIR}/brochures.svg"

# Generate the PDF files.
inkscape --export-area-drawing --batch-process --export-type=pdf *.svg
