#!/usr/bin/env bash
set -euo pipefail

TARGET_VERSION="${1:?target version required}"
DIST_DIR="${2:?dist dir required}"
GH_PAGES_DIR="${3:?gh-pages dir required}"
ROOT_DIST_DIR="${4:-}"
ROOT_BASE="${ROOT_BASE:-}"
CUSTOM_DOMAIN="${CUSTOM_DOMAIN:-immich-edit.iridiumlab.dev}"

mkdir -p "$GH_PAGES_DIR"
touch "$GH_PAGES_DIR/.nojekyll"
if [ -n "$CUSTOM_DOMAIN" ]; then
  printf '%s\n' "$CUSTOM_DOMAIN" > "$GH_PAGES_DIR/CNAME"
fi

rm -rf "$GH_PAGES_DIR/$TARGET_VERSION"
mkdir -p "$GH_PAGES_DIR/$TARGET_VERSION"
cp -R "$DIST_DIR"/. "$GH_PAGES_DIR/$TARGET_VERSION"/

LATEST="$(node -e '
const fs = require("node:fs");
const dir = process.argv[1];
const parse = (v) => {
  const m = /^v(\d+)\.(\d+)\.(\d+)(?:-(.+))?$/.exec(v);
  if (!m) return null;
  return { major: +m[1], minor: +m[2], patch: +m[3], pre: m[4] || null };
};
const cmp = (a, b) => {
  for (const k of ["major", "minor", "patch"]) {
    if (a[k] !== b[k]) return b[k] - a[k];
  }
  if (!a.pre && b.pre) return -1;
  if (a.pre && !b.pre) return 1;
  return (b.pre || "").localeCompare(a.pre || "");
};
const stables = fs.readdirSync(dir, { withFileTypes: true })
  .filter((d) => d.isDirectory())
  .map((d) => ({ id: d.name, p: parse(d.name) }))
  .filter((x) => x.p && !x.p.pre)
  .sort((a, b) => cmp(a.p, b.p));
process.stdout.write(stables[0]?.id ?? "");
' "$GH_PAGES_DIR")"

if [ -n "$ROOT_DIST_DIR" ] && { [ "$TARGET_VERSION" = "$LATEST" ] || { [ -z "$LATEST" ] && [ "$TARGET_VERSION" = "main" ]; }; }; then
  for entry in "$GH_PAGES_DIR"/* "$GH_PAGES_DIR"/.[!.]* "$GH_PAGES_DIR"/..?*; do
    [ -e "$entry" ] || continue
    name="$(basename "$entry")"
    case "$name" in
      .git|.nojekyll|CNAME|versions.json|main|v[0-9]*)
        continue
        ;;
      *)
        rm -rf "$entry"
        ;;
    esac
  done
  cp -R "$ROOT_DIST_DIR"/. "$GH_PAGES_DIR"/
fi

node -e '
const fs = require("node:fs");
const path = require("node:path");
const dir = process.argv[1];
const rootBase = process.argv[2].replace(/\/+$/, "");
const latest = process.argv[3] || null;
const parse = (v) => {
  const m = /^v(\d+)\.(\d+)\.(\d+)(?:-(.+))?$/.exec(v);
  if (!m) return null;
  return { major: +m[1], minor: +m[2], patch: +m[3], pre: m[4] || null };
};
const cmp = (a, b) => {
  for (const k of ["major", "minor", "patch"]) {
    if (a[k] !== b[k]) return b[k] - a[k];
  }
  if (!a.pre && b.pre) return -1;
  if (a.pre && !b.pre) return 1;
  return (b.pre || "").localeCompare(a.pre || "");
};
const entries = fs.readdirSync(dir, { withFileTypes: true }).filter((d) => d.isDirectory());
const tags = entries
  .map((d) => ({ id: d.name, p: parse(d.name) }))
  .filter((x) => x.p)
  .sort((a, b) => cmp(a.p, b.p))
  .map((x) => ({
    id: x.id,
    label: x.id === latest ? `${x.id} (latest)` : x.id,
    base: x.id === latest ? (rootBase || "/") : `${rootBase}/${x.id}`
  }));
const versions = [...tags];
if (entries.some((d) => d.name === "main")) {
  versions.push({ id: "main", label: "main (dev)", base: `${rootBase}/main` });
}
fs.writeFileSync(
  path.join(dir, "versions.json"),
  JSON.stringify({ latest, versions }, null, 2) + "\n"
);
' "$GH_PAGES_DIR" "$ROOT_BASE" "$LATEST"
