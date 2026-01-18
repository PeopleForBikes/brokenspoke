#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.14"
# dependencies = [
#   "pandas>=2.0",
#   "typer>=0.12",
#   "us>=3.2",
#   "xdoctest>=1.1",
# ]
# ///
"""
Build an official fips_place_lookup.csv from U.S. Census Gazetteer "Places" files.

This is a `uv` PEP-723 script, dependencies are declared inline.

Run directly with:
    uv run fips.py \
        build \
        --year 2024 \
        --out fips_place_lookup.csv
    uv run fips.py \
        validate \
        --dataset-csv assets/city-ratings/city-ratings-all-historical-results-v25.09.csv
    uv run fips.py \
        fix \
        --dataset-csv assets/city-ratings/city-ratings-all-historical-results-v25.09.csv \
        --out assets/city-ratings/city-ratings-all-historical-results-v25.09-fixed.csv

Outputs:
- fips_place_lookup.csv               (fips, place_name, state_abbr, state_name)
- fips_place_lookup.manifest.json     (sha256 inputs + output for auditability)

Format with:
    uv tool run isort --profile black --fgw 2 fips.py
    uv tool run ruff format fips.py

Run all xdoctest-based docstring tests in this file with:
    uv run \
        --with pandas \
        --with us \
        --with typer \
        --with xdoctest \
        python -m xdoctest \
        fips.py \
        all
"""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
from typing import (
    Dict,
    List,
    Tuple,
)

# Ensure DC is considered a US state.
# https://github.com/unitedstates/python-us/issues/67
os.environ["DC_STATEHOOD"] = "1"

import pandas as pd
import typer
import us
from typer import Option

app = typer.Typer(add_completion=False)


def sha256_file(path: Path) -> str:
    """
    Compute the SHA256 hash of a file.

    >>> from pathlib import Path
    >>> p = Path("test.txt")
    >>> _ = p.write_text("hello")
    >>> sha256_file(p)
    '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'
    >>> p.unlink()
    """
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def build_state_code_table(include_pr: bool, include_territories: bool) -> pd.DataFrame:
    """
    Build a table of state/territory FIPS metadata using the `us` library.

    Returns columns:
      - state_fips (2-digit string)
      - state_abbr
      - state_name

    >>> df = build_state_code_table(include_pr=True, include_territories=False)
    >>> "TX" in df["state_abbr"].values
    True
    >>> df[df["state_abbr"] == "TX"]["state_fips"].iloc[0]
    '48'
    >>> "PR" in df["state_abbr"].values
    True
    """
    records: List[Tuple[str, str, str]] = []

    for s in us.STATES:
        if s.fips:
            records.append((str(s.fips).zfill(2), s.abbr, s.name))

    if include_pr or include_territories:
        for t in us.TERRITORIES:
            if not t.fips:
                continue
            fips = str(t.fips).zfill(2)
            if t.abbr == "PR" and include_pr:
                records.append((fips, t.abbr, t.name))
            elif t.abbr != "PR" and include_territories:
                records.append((fips, t.abbr, t.name))

    return (
        pd.DataFrame(records, columns=["state_fips", "state_abbr", "state_name"])
        .drop_duplicates()
        .sort_values("state_fips")
        .reset_index(drop=True)
    )


def expected_state_fips_prefix(state_abbr: str) -> str:
    """
    Return the expected 2-digit FIPS prefix for a USPS state/territory abbreviation.

    Uses `us.states.lookup`, which correctly supports states, DC, and PR.

    >>> expected_state_fips_prefix("TX")
    '48'
    >>> expected_state_fips_prefix("CA")
    '06'
    >>> expected_state_fips_prefix("DC")
    '11'
    >>> expected_state_fips_prefix("PR")
    '72'
    >>> expected_state_fips_prefix("XX")
    ''
    """
    st = us.states.lookup(state_abbr)
    if st and st.fips:
        return str(st.fips).zfill(2)
    return ""


def normalize_fips_7(x: str) -> str:
    """
    Normalize a FIPS-like value to a 7-digit string when safe.

    Safe normalization rules:
      - If x is all digits and length < 7: left-pad with zeros to 7 digits.
      - If x is all digits and length == 7: unchanged.
      - Otherwise: unchanged.

    >>> normalize_fips_7("101852")
    '0101852'
    >>> normalize_fips_7("4805000")
    '4805000'
    >>> normalize_fips_7("nan")
    'nan'
    >>> normalize_fips_7("48A5000")
    '48A5000'
    """
    s = str(x)
    if s.isdigit() and len(s) < 7:
        return s.zfill(7)
    return s


def parse_gazetteer_file(path: Path) -> pd.DataFrame:
    """
    Parse a Census Gazetteer place file (TSV).

    Only valid 7-digit GEOIDs are retained.

    >>> import pandas as pd
    >>> from pathlib import Path
    >>> p = Path("gaz_test.tsv")
    >>> _ = p.write_text("USPS\\tGEOID\\tNAME\\nTX\\t4805000\\tAustin\\nTX\\tBAD\\tX")
    >>> df = parse_gazetteer_file(p)
    >>> list(df["GEOID"])
    ['4805000']
    >>> p.unlink()
    """
    df = pd.read_csv(path, sep="\t", dtype=str, keep_default_na=False)
    df.columns = [c.strip().lstrip("\ufeff") for c in df.columns]

    required = {"USPS", "GEOID", "NAME"}
    missing = required - set(df.columns)
    if missing:
        raise typer.BadParameter(f"Missing required columns: {sorted(missing)}")

    df = df[["USPS", "GEOID", "NAME"]]
    return df[df["GEOID"].str.fullmatch(r"\d{7}")]


# ─────────────────────────────────────────────────────────────
# CLI commands (build / validate / fix)
# ─────────────────────────────────────────────────────────────


@app.command()
def build(
    year: int = Option(2024),
    out: Path = Option(Path("fips_place_lookup.csv")),
    workdir: Path = Option(Path(".cache/gazetteer_places")),
    include_pr: bool = Option(True),
    include_territories: bool = Option(False),
    commit_guard: bool = Option(False),
) -> None:
    """
    Build fips_place_lookup.csv from Census Gazetteer data.
    """
    base = f"https://www2.census.gov/geo/docs/maps-data/data/gazetteer/{year}_Gazetteer"
    state_codes = build_state_code_table(include_pr, include_territories)

    old_hash = sha256_file(out) if commit_guard and out.exists() else None
    rows: List[pd.DataFrame] = []
    inputs: List[Dict[str, str]] = []

    for _, r in state_codes.iterrows():
        state_fips = r["state_fips"]
        url = f"{base}/{year}_gaz_place_{state_fips}.txt"
        dest = workdir / f"{year}_gaz_place_{state_fips}.txt"

        dest.parent.mkdir(parents=True, exist_ok=True)
        df_raw = pd.read_csv(url, sep="\t", dtype=str, keep_default_na=False)
        df_raw.to_csv(dest, sep="\t", index=False)

        inputs.append({"url": url, "path": str(dest), "sha256": sha256_file(dest)})

        df = parse_gazetteer_file(dest)
        df = df.rename(
            columns={"GEOID": "fips", "NAME": "place_name", "USPS": "state_abbr"}
        )
        df = df.merge(
            state_codes[["state_abbr", "state_name"]],
            on="state_abbr",
            how="left",
            validate="m:1",
        )
        rows.append(df[["fips", "place_name", "state_abbr", "state_name"]])

    lookup = (
        pd.concat(rows, ignore_index=True)
        .drop_duplicates(subset=["fips"])
        .sort_values("fips")
        .reset_index(drop=True)
    )

    lookup.to_csv(out, index=False)

    manifest = {
        "source": {"type": "census_gazetteer_places", "year": year},
        "inputs": inputs,
        "output": {
            "path": str(out),
            "sha256": sha256_file(out),
            "row_count": len(lookup),
        },
    }

    out.with_suffix(".manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True)
    )

    typer.echo(f"Wrote {out} with {len(lookup):,} rows")

    if commit_guard and old_hash and sha256_file(out) != old_hash:
        raise typer.Exit(code=1)


@app.command()
def validate(
    dataset_csv: Path = Option(...),
    lookup_csv: Path = Option(Path("fips_place_lookup.csv")),
    country_value: str = Option("UNITED STATES"),
) -> None:
    """
    Validate US rows in a dataset against fips_place_lookup.csv.
    """
    df = pd.read_csv(
        dataset_csv, dtype={"census_fips_code": str}, keep_default_na=False
    )
    us_df = df[df["country"].astype(str).str.upper() == country_value.upper()].copy()

    us_df["valid_len"] = us_df["census_fips_code"].str.fullmatch(r"\d{7}").fillna(False)
    us_df["expected_prefix"] = us_df["state"].apply(expected_state_fips_prefix)
    us_df["state_prefix"] = us_df["census_fips_code"].str[:2]
    us_df["state_prefix_ok"] = us_df["expected_prefix"] == us_df["state_prefix"]

    lookup = pd.read_csv(lookup_csv, dtype={"fips": str})[["fips"]]
    us_df = us_df.merge(lookup, left_on="census_fips_code", right_on="fips", how="left")
    us_df["place_known"] = us_df["fips"].notna()

    errors = us_df[
        ~us_df["valid_len"] | ~us_df["state_prefix_ok"] | ~us_df["place_known"]
    ]

    typer.echo(f"Found {len(errors)} problematic US rows out of {len(us_df)} total.")
    if not errors.empty:
        typer.echo(
            errors[
                [
                    "city",
                    "state",
                    "census_fips_code",
                    "valid_len",
                    "state_prefix_ok",
                    "place_known",
                ]
            ].to_string(index=False)
        )
        raise typer.Exit(code=1)


@app.command()
def fix(
    dataset_csv: Path = Option(..., help="Input dataset CSV to fix."),
    out: Path = Option(
        Path("city-ratings-all-historical-results-fixed.csv"), help="Output CSV path."
    ),
    country_value: str = Option(
        "UNITED STATES", help="Country label to treat as US (case-insensitive)."
    ),
) -> None:
    """
    Fix (safe normalization only) of census_fips_code for US rows.

    What it changes:
      - For US rows, if census_fips_code is all digits and length < 7,
        left-pad with zeros to 7 digits.

    What it does NOT do:
      - It does NOT attempt to "guess" corrections for state prefix mismatches.
      - It does NOT rewrite non-numeric FIPS values.

    Use `validate` after `fix` to ensure everything is now correct.
    """
    df = pd.read_csv(
        dataset_csv, dtype={"census_fips_code": str}, keep_default_na=False
    )

    mask = df["country"].astype(str).str.upper() == country_value.upper()

    before = df.loc[mask, "census_fips_code"].copy()
    df.loc[mask, "census_fips_code"] = df.loc[mask, "census_fips_code"].apply(
        normalize_fips_7
    )
    after = df.loc[mask, "census_fips_code"]

    changed = (before != after).sum()

    out.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(out, index=False)

    typer.echo(f"Wrote fixed dataset to {out}")
    typer.echo(
        f"Updated {changed} US rows by left-padding census_fips_code to 7 digits (when safe)."
    )


if __name__ == "__main__":
    app()
