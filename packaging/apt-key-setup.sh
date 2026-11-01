#!/bin/sh
# One-time setup of the apt repository's signing key (run by the repository owner).
#
# Creates an ed25519 signing key in a throw-away GnuPG home, stores the private key as
# the GitHub Actions secret APT_SIGNING_KEY (the release workflow signs with it), writes
# the public key to packaging/coreglow-archive-keyring.asc (commit that), and deletes
# the throw-away copy. Needs gpg and an authenticated gh. Running it again replaces the
# key, and everyone using the repository then has to fetch the new public key.
set -eu
cd "$(dirname "$0")/.."

home=$(mktemp -d)
trap 'rm -rf "$home"' EXIT
export GNUPGHOME="$home"

gpg --batch --quiet --passphrase '' --quick-gen-key \
    "Coreglow apt repository <96940759+Nexus-SvK@users.noreply.github.com>" ed25519 sign 5y
gpg --armor --export > packaging/coreglow-archive-keyring.asc
gpg --armor --export-secret-keys | gh secret set APT_SIGNING_KEY

echo "Signing key stored as the APT_SIGNING_KEY secret; public key in packaging/coreglow-archive-keyring.asc:"
gpg --list-keys --keyid-format long | sed -n '/^pub/,/^uid/p'
