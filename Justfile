assets_dir := "assets"
tools_dir := "tools"
fips_year := "2024"
fips_asset_file := assets_dir / "city-ratings/city-ratings-all-historical-results-v25.09.csv"
fips_place_lookup := tools_dir / "fips_place_lookup.csv"
tool_fips := tools_dir / "fips.py"

# Meta task running ALL the CI tasks at onces.
ci: lint

# Meta task running all the linters at once.
lint: lint-md lint-spellcheck

# Lint markown files.
lint-md:
    npx --yes markdownlint-cli2 "**/*.md" "#.venv" "#docs/themes" "#target/*" "#target"

# Check spelling.
lint-spellcheck:
    npx --yes cspell --no-progress --show-suggestions --show-context "**/*.md"

# Meta tasks running all formatters at once.
fmt: fmt-md fmt-just

# Format the justfile.
fmt-just:
    just --fmt --unstable

# Format markdown files.
fmt-md:
    npx --yes prettier --write --prose-wrap always "**/*.md"

# Build the documentation
docs:
    cd docs && zola build

# Build FIPS lookup table.
tool-fips-build:
    uv run {{ tool_fips }} build --year {{ fips_year }} --out {{ fips_place_lookup }}

# Validate FIPS dataset.
tools-fips-validate:
    uv run {{ tool_fips }} validate --dataset-csv {{ fips_asset_file }} --lookup-csv {{ fips_place_lookup }}

# Fix FIPS dataset.
tool-fips-fix:
    uv run {{ tool_fips }} fix --dataset-csv {{ fips_asset_file }} --out {{ fips_asset_file }}
