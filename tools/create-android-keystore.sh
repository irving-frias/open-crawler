#!/usr/bin/env bash
set -euo pipefail

# Generates the Android signing keystore and prints the exact GitHub Actions
# secret values to paste into: repo Settings -> Secrets and variables -> Actions.
#
# Usage:
#   tools/create-android-keystore.sh [--alias <alias>] [--password <password>] [--out <path>]
#
# Defaults match ANDROID_SETUP.md: alias "opencrawler", password "android".

ALIAS="opencrawler"
PASSWORD="android"
OUT="android/opencrawler.keystore"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --alias) ALIAS="$2"; shift 2 ;;
    --password) PASSWORD="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

mkdir -p "$(dirname "$OUT")"

run_keytool() {
  if command -v keytool >/dev/null 2>&1; then
    keytool "$@"
  elif command -v docker >/dev/null 2>&1; then
    docker run --rm -v "$PWD:/work" -w /work eclipse-temurin:17-jdk keytool "$@"
  else
    echo "keytool not found. Install a JDK or Docker, or run this on a machine with Java." >&2
    exit 1
  fi
}

echo "Generating keystore at $OUT (alias=$ALIAS, validity=10000 days)..."
run_keytool -genkeypair -v \
  -keystore "$OUT" \
  -alias "$ALIAS" \
  -keyalg RSA -keysize 2048 -validity 10000 \
  -storepass "$PASSWORD" -keypass "$PASSWORD" \
  -dname "CN=Open Crawler, OU=Development, O=Open Crawler, L=City, ST=State, C=US" 2>&1

echo
echo "Verifying..."
run_keytool -list -keystore "$OUT" -storepass "$PASSWORD"

echo
echo "Copy these values into GitHub Actions secrets:"
echo "  ANDROID_KEYSTORE_BASE64      = $(base64 -w 0 "$OUT")"
echo "  ANDROID_KEY_ALIAS            = $ALIAS"
echo "  ANDROID_KEYSTORE_PASSWORD    = $PASSWORD"
echo "  ANDROID_KEY_PASSWORD         = $PASSWORD"
echo
echo "NOTE: store the keystore file ($OUT) somewhere safe; it is gitignored."
