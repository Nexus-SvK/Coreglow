#!/bin/sh
# CI check of apt-repo.sh: build a repository with a throw-away key into a scratch remote,
# then read it with apt (as a normal user, in scratch directories) like a client would.
# usage: packaging/test-apt-repo.sh target/debian/coreglow_*.deb
set -eu
deb=$(realpath "$1")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# throw-away signing key
export GNUPGHOME="$work/gpg"
mkdir -m700 "$GNUPGHOME"
gpg --batch --quiet --passphrase '' --quick-gen-key "apt test <test@example.invalid>" ed25519 sign 1d
APT_SIGNING_KEY=$(gpg --armor --export-secret-keys)
export APT_SIGNING_KEY
gpg --export > "$work/key.gpg"
unset GNUPGHOME

# publish into a scratch remote
git init --quiet --bare "$work/remote.git"
git clone --quiet "$(pwd)" "$work/src"
git -C "$work/src" remote set-url origin "$work/remote.git"
(cd "$work/src" && packaging/apt-repo.sh "$deb")
git clone --quiet -b gh-pages "$work/remote.git" "$work/site"

# read it back with apt
a="$work/apt"
mkdir -p "$a/lists/partial" "$a/cache/archives/partial" "$a/etc/apt.conf.d" "$a/etc/preferences.d" "$a/etc/sources.list.d"
echo "deb [signed-by=$work/key.gpg] file://$work/site stable main" > "$a/etc/sources.list"
set -- -o Dir::Etc="$a/etc" -o Dir::Etc::sourcelist="$a/etc/sources.list" -o Dir::Etc::sourceparts="$a/etc/sources.list.d" \
    -o Dir::State::Lists="$a/lists" -o Dir::State::status=/dev/null -o Dir::Cache="$a/cache" -o Debug::NoLocking=1 \
    -o APT::Sandbox::User="$(id -un)"
apt-get "$@" update
version=$(dpkg-deb -f "$deb" Version)
apt-cache "$@" policy coreglow | tee "$work/policy"
grep -q "Candidate: $version" "$work/policy"
echo "apt repository check passed (coreglow $version, signed)"
