#!/bin/sh
# Add .debs to the apt repository on the gh-pages branch, re-index and sign it, and push.
# Run by the release workflow: packaging/apt-repo.sh dist/*.deb
# Needs APT_SIGNING_KEY (armored private key) in the environment, apt-ftparchive and gpg.
#
# Layout (served by GitHub Pages at https://nexus-svk.github.io/Coreglow):
#   dists/stable/{Release,InRelease,Release.gpg}, dists/stable/main/binary-amd64/Packages(.gz),
#   pool/main/c/coreglow/*.deb (the newest KEEP versions), coreglow-archive-keyring.gpg
set -eu
KEEP=3

: "${APT_SIGNING_KEY:?APT_SIGNING_KEY isn't set (run packaging/apt-key-setup.sh once)}"
src=$(pwd)
site=$(mktemp -d)
GNUPGHOME=$(mktemp -d)
export GNUPGHOME
trap 'git -C "$src" worktree remove --force "$site" 2>/dev/null || true; rm -rf "$GNUPGHOME"' EXIT

if git fetch --quiet origin gh-pages 2>/dev/null; then
    git worktree add --quiet -B gh-pages "$site" FETCH_HEAD
else
    git worktree add --quiet --orphan -b gh-pages "$site"
fi

pool="$site/pool/main/c/coreglow"
mkdir -p "$pool" "$site/dists/stable/main/binary-amd64"
for deb in "$@"; do
    cp "$deb" "$pool/"
done
# keep the newest $KEEP versions
ls "$pool"/*.deb | sort -V | head -n -"$KEEP" | xargs -r rm -f

cd "$site"
apt-ftparchive packages pool > dists/stable/main/binary-amd64/Packages
gzip -9nkf dists/stable/main/binary-amd64/Packages
apt-ftparchive \
    -o APT::FTPArchive::Release::Origin=Coreglow \
    -o APT::FTPArchive::Release::Label=Coreglow \
    -o APT::FTPArchive::Release::Suite=stable \
    -o APT::FTPArchive::Release::Codename=stable \
    -o APT::FTPArchive::Release::Architectures=amd64 \
    -o APT::FTPArchive::Release::Components=main \
    -o APT::FTPArchive::Release::Description="Coreglow for Corsair Commander controllers" \
    release dists/stable > "$GNUPGHOME/Release"
mv "$GNUPGHOME/Release" dists/stable/Release

printf '%s\n' "$APT_SIGNING_KEY" | gpg --batch --quiet --import
gpg --batch --yes --clearsign -o dists/stable/InRelease dists/stable/Release
gpg --batch --yes --armor --detach-sign -o dists/stable/Release.gpg dists/stable/Release
gpg --export > coreglow-archive-keyring.gpg

touch .nojekyll
cat > index.html <<'EOF'
<!doctype html>
<meta charset="utf-8">
<title>Coreglow apt repository</title>
<h1>Coreglow apt repository</h1>
<p>Lighting effects, profiles and fan control for Corsair Commander controllers on Linux.
See <a href="https://github.com/Nexus-SvK/Coreglow">github.com/Nexus-SvK/Coreglow</a>.</p>
<pre>
sudo curl -fsSLo /usr/share/keyrings/coreglow-archive-keyring.gpg https://nexus-svk.github.io/Coreglow/coreglow-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/coreglow-archive-keyring.gpg] https://nexus-svk.github.io/Coreglow stable main" | sudo tee /etc/apt/sources.list.d/coreglow.list
sudo apt update &amp;&amp; sudo apt install coreglow
</pre>
EOF

git add -A
git -c user.name="github-actions[bot]" -c user.email="41898282+github-actions[bot]@users.noreply.github.com" \
    commit --quiet -m "apt repository: $(ls pool/main/c/coreglow | tr '\n' ' ')"
git push --quiet origin gh-pages
echo "apt repository updated: $(ls pool/main/c/coreglow | tr '\n' ' ')"
