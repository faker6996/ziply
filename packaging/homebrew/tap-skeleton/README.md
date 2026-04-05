# Homebrew Tap Skeleton

This directory is a starter layout for a dedicated Homebrew tap repository such as:

```text
faker6996/homebrew-tap
```

Expected repository structure:

```text
homebrew-tap/
├── Casks/
│   └── ziply.rb
├── .github/
│   └── workflows/
│       └── validate-cask.yml
└── README.md
```

## Install flow after publish

```bash
brew tap faker6996/tap
brew install --cask faker6996/tap/ziply
```

## Bootstrapping from this repo

From the main project repository, run:

```bash
scripts/bootstrap-homebrew-tap.sh /path/to/homebrew-tap
```

If a rendered cask already exists at `packaging/homebrew/Casks/ziply.rb`, the bootstrap script will copy it into the target repo.
Otherwise it will copy the template cask and keep the placeholder values.

## Before publishing

Make sure the cask contains real values for:

- version
- sha256
- url

Then rename `validate-cask.yml.template` to `validate-cask.yml` if the bootstrap script did not already do that for you.
