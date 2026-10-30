# Releasing

## Making a release

1. Update the version in `[workspace.package]` of the root `Cargo.toml` (the only place it is
   defined) and move the `Unreleased` entries of `CHANGELOG.md` under the new version.
2. Commit, then tag and push: `git tag -s v1.2.3 -m "NanoPrint 1.2.3" && git push origin v1.2.3`.
   Tags with a hyphen (`v1.2.3-1`) become pre-releases.
3. The [release workflow](../.github/workflows/release.yml) builds Windows, macOS (universal) and
   Linux installers from scratch, generates the third-party license notices, attests the build
   provenance of every file and attaches an SBOM, all to a **draft** release.
4. A maintainer checks the draft (install it on at least one system) and publishes it.
5. While Windows installers are unsigned, submit the published `NanoPrint_x64-setup.exe` to
   [Microsoft](https://www.microsoft.com/en-us/wdsi/filesubmission) as a software developer.
   Without a signature, SmartScreen judges each file by its own hash, so every release starts with
   no reputation and needs its own submission. It may clear the warning within a day or two; it is
   not guaranteed. Fill in:
   - Product: **Microsoft Defender Smartscreen**; the file is **Incorrectly detected as
     malware/malicious**.
   - Detection name: `SmartScreen: Unrecognized app ("Windows protected your PC")`. Leave the
     definition version empty.
   - Additional information: what NanoPrint is, the repository and release URLs, the installer's
     SHA-256 (`gh release view v1.2.3 --json assets` shows each asset's `digest`), the
     `gh attestation verify` command below, and that the installer is not code-signed yet.

Assets keep version-less names (`NanoPrint_<arch><-setup><ext>`, for example
`NanoPrint_x64-setup.exe`), so `https://github.com/nanofactura/nanoprint/releases/latest/download/<name>`
always serves the newest release and the landing page never needs a redeploy. Check the exact names
in the first draft and use them in the landing page.

Anyone can verify that an installer was built by this repository's workflow:

```sh
gh attestation verify NanoPrint_x64-setup.exe --repo nanofactura/nanoprint
```

## Code signing

| System  | Today                                                                                                                                                                   | Plan                                                                                      |
| ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| Windows | Unsigned: SmartScreen shows "Windows protected your PC"; users choose **More info → Run anyway**.                                                                       | Apply to the [SignPath Foundation](https://signpath.org/), free for open-source projects. |
| macOS   | Ad-hoc signature (`signingIdentity: "-"`), not notarized. Gatekeeper blocks the first launch; users allow it in **System Settings → Privacy & Security → Open Anyway**. | Developer ID and notarization need a paid Apple Developer account.                        |
| Linux   | Packages are not signed; the provenance attestation covers them.                                                                                                        | Signed repository when there is one.                                                      |

### Windows: SignPath Foundation

SignPath Foundation signs open-source releases for free with its own certificate (Windows shows
"SignPath Foundation" as the publisher). It requires a project that is already released and has
some verifiable reputation, so the order is:

1. Publish the first releases unsigned and let them gather users (downloads, stars, issues).
2. Turn on multi-factor authentication for every maintainer, on GitHub and later on SignPath.
3. Apply at [signpath.org](https://signpath.org/) with the repository, the license (GPL-3.0-only is
   OSI-approved) and this release workflow, which builds every installer from the tagged source.
4. Once accepted, add a "Code signing policy" section to the README and the landing page: "Free code
   signing provided by SignPath.io, certificate by SignPath Foundation", plus the team roles
   (authors, reviewers, approvers). The README already states the privacy policy and how to
   uninstall, which SignPath also requires.
5. Add SignPath's GitHub Action to the release workflow: CI submits each installer, SignPath checks
   that it came from this repository's build, and a maintainer approves every signing by hand.
   The release then waits until someone with the approver role confirms that version in SignPath
   (it notifies by email); a compromised workflow or account alone cannot get an installer signed.

### macOS without an Apple Developer account

Gatekeeper only opens notarized apps without asking, and notarization needs the paid program (Apple
waives the fee for some nonprofits, schools and governments). Until then:

- The README and the landing page show the "Open Anyway" steps, ideally with screenshots.
- IT staff installing many Macs can clear the download's quarantine flag instead:
  `xattr -dr com.apple.quarantine /Applications/NanoPrint.app`.
- Releases are signed with a stable self-signed identity, "NanoPrint Code Signing". An ad-hoc
  signature changes with every build, and macOS ties Keychain access to the signature: after each
  update it would ask the user again (password included) before NanoPrint may read its signing key.
  With the same identity on every version, the first "Always Allow" lasts. It does not avoid
  Gatekeeper.

Setting up the identity, once:

1. A maintainer runs `scripts/macos-signing-certificate.sh` on their own computer (Git Bash works on
   Windows). It creates a ten-year certificate for code signing only.
2. Add `certificate.p12.base64` and `password.txt` as the `MACOS_CERTIFICATE` and
   `MACOS_CERTIFICATE_PASSWORD` secrets of the `release` environment.
3. Keep `certificate.p12` and its password in the password manager, then delete the directory.

The release workflow imports it into a temporary keychain and trusts it for code signing on the
throwaway runner only. Without the secret, the app keeps its ad-hoc signature (local builds and
forks too).

Its private key deserves care: a program signed with it, running as the user, would pass the
Keychain's check for NanoPrint's signing key without a prompt. If it leaks, create a new one and
replace the secrets; users get one Keychain prompt after the next update. When it expires, in ten
years, the same applies.

## Update signing key

The updater only installs updates signed with NanoPrint's update key; the matching public key is
embedded in the app. The private key is the most sensitive secret of the project:

- **If it leaks**, whoever also controls the release downloads can push code to every user.
- **If it is lost**, installed copies cannot verify any new update, and every user has to reinstall
  by hand.

Custody:

1. A maintainer generates it on their own computer, never in CI or a shared session:
   `npx tauri signer generate -w ~/.tauri/nanoprint.key`, with a strong password.
2. The key file and its password go into the maintainers' password manager, plus one offline copy
   (an encrypted USB drive kept by a second maintainer).
3. They are added as secrets of the protected `release` environment only:
   `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
4. The public key goes into `tauri.conf.json` (`plugins.updater.pubkey`); it is not secret.
   NanoPrint's key was generated on 2026-10-16; its public key is the one in `tauri.conf.json`.
5. `npm run tauri build` on a computer needs the same two variables, because every build signs
   its updater artifacts.
6. If the key leaks: publish at once an update, signed with the old key, that embeds a new public
   key; revoke the old secrets; announce it.

## Repository settings (outside the repository)

- **Private vulnerability reporting** on (Settings → Code security), which SECURITY.md relies on.
- **Branch protection for `main`**: pull requests only, required checks (CI, CodeQL), up-to-date
  branches, no force pushes.
- **Tag protection** for `v*`, so only maintainers can trigger a release.
- **`release` environment**: required reviewers, deployments limited to `v*` tags.
- **Secret scanning and push protection**, Dependabot alerts and security updates.
