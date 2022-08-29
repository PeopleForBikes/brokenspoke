# Meta task running ALL the CI tasks at onces.
ci: lint docs

# Meta task running all the linters at once.
lint: lint-md

# Lint markown files.
lint-md:
    npx --yes markdownlint-cli2 "**/*.md" "#.venv"

# Meta tasks running all formatters at once.
fmt: fmt-md fmt-just

# Format the justfile.
fmt-just:
    just --fmt --unstable

# Format markdown files.
fmt-md:
    npx --yes prettier --write --prose-wrap always **/*.md

# Build the documentation
docs:
    echo "TBD"


