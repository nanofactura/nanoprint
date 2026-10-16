#!/usr/bin/env bash
# Creates NanoPrint's self-signed macOS code-signing certificate. A maintainer runs it once, on
# their own computer (Git Bash works on Windows), and stores the output as described in
# docs/releasing.md. The release workflow signs the macOS app with it, so every version carries the
# same identity and macOS does not ask again for NanoPrint's Keychain item after an update.
#
# Usage: scripts/macos-signing-certificate.sh [output-directory]
set -euo pipefail

name="NanoPrint Code Signing"
out="${1:-nanoprint-macos-signing}"

if [ -e "$out" ]; then
  echo "$out already exists; choose another directory." >&2
  exit 1
fi
# Files readable by the current user only (where the file system supports it).
umask 077
mkdir "$out"

# Ten years, RSA 3072, for code signing only. A config of our own: the system's default one would
# add its CA extensions next to these.
cat > "$out/openssl.cnf" <<EOF
[req]
prompt = no
distinguished_name = subject
x509_extensions = code_signing
[subject]
CN = $name
O = Nanofactura
[code_signing]
basicConstraints = critical, CA:false
keyUsage = critical, digitalSignature
extendedKeyUsage = critical, codeSigning
subjectKeyIdentifier = hash
EOF
openssl req -x509 -config "$out/openssl.cnf" -newkey rsa:3072 -sha256 -days 3650 -nodes \
  -keyout "$out/key.pem" -out "$out/certificate.pem"
rm "$out/openssl.cnf"

password="$(openssl rand -hex 24)"
# PBE-SHA1-3DES: the PKCS#12 encryption every macOS version's `security import` reads.
openssl pkcs12 -export -name "$name" \
  -inkey "$out/key.pem" -in "$out/certificate.pem" \
  -keypbe PBE-SHA1-3DES -certpbe PBE-SHA1-3DES -macalg sha1 \
  -passout "pass:$password" -out "$out/certificate.p12"
openssl base64 -A -in "$out/certificate.p12" -out "$out/certificate.p12.base64"
printf '%s' "$password" > "$out/password.txt"
rm "$out/key.pem"

cat <<EOF

Created in $out:
  certificate.p12          the certificate and its private key, encrypted
  certificate.p12.base64   the same, as the MACOS_CERTIFICATE secret
  password.txt             its password, the MACOS_CERTIFICATE_PASSWORD secret
  certificate.pem          the public certificate only

Next:
  1. Add both secrets to the "release" environment in GitHub
     (Settings > Environments > release > Environment secrets).
  2. Keep certificate.p12 and password.txt in the maintainers' password manager.
  3. Delete this directory.
EOF
