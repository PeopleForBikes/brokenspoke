# Meta task running ALL the CI tasks at onces.
ci: lint

# Meta task running all the linters at once.
lint: lint-md lint-spellcheck

# Lint markown files.
lint-md:
    npx --yes markdownlint-cli2 "**/*.md" "#.venv" "#docs/themes" "#target/*" "#target"

# Lint sass files.
lint-sass:
  npx --yes prettier --check "**/*.scss"

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

# Format SASS files.
fmt-sass:
  npx --yes prettier --write "**/*.scss"

# Build the documentation
docs:
    cd docs && zola build
