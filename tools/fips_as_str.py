#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "pandas>=2.0",
# ]
# ///
"""
Quick script to convert FIPS code column from int64 to string in the city ratings dataset.
"""

import csv

import pandas as pd

DATASET = "assets/city-ratings/city-ratings-all-historical-results-v25.09.csv"
FIPS_COLUMN = "census_fips_code"  # Replace with the actual column name

# Read the CSV file.
df = pd.read_csv(DATASET)

# Change the column type from int64 to string, and adds quotes around the FIPS codes.
df[FIPS_COLUMN] = '"' + df[FIPS_COLUMN].astype("string") + '"'

# Save the changes back.
df.to_csv(DATASET, index=False, quoting=csv.QUOTE_NONE, escapechar="\\")
