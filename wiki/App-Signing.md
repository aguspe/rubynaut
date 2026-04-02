# App Signing & Distribution

## Do I need to sign the app?

| Platform | Required? | Without signing | Cost |
|----------|-----------|----------------|------|
| **macOS** | Practically yes | Gatekeeper blocks the app. Users must `xattr -cr` or allow in System Settings. | $99/year (Apple Developer Program) |
| **Windows** | Recommended | SmartScreen shows "Windows protected your PC" warning. Users can click "Run anyway". | $200-400/year for a code signing cert, or free via [SignPath](https://signpath.io/) for OSS |
| **Linux** | No | No signing required. Distribute `.deb`, `.rpm`, or `.AppImage` freely. | Free |

## macOS signing and notarization

### Requirements
1. Apple Developer Program membership ($99/year) at [developer.apple.com](https://developer.apple.com)
2. A "Developer ID Application" certificate
3. An app-specific password for notarization

### Setup for CI

Add these secrets to your GitHub repository:

| Secret | Value |
|--------|-------|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12` |
| `APPLE_SIGNING_IDENTITY` | e.g., `Developer ID Application: Your Name (TEAMID)` |
| `APPLE_ID` | Your Apple ID email |
| `APPLE_PASSWORD` | App-specific password (not your Apple ID password) |
| `APPLE_TEAM_ID` | Your 10-character team ID |

### Tauri configuration

Add to `tauri.conf.json` under `bundle > macOS`:
```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: ...",
      "providerShortName": "TEAMID"
    }
  }
}
```

## Windows signing

### Options
1. **OV (Organization Validation) certificate**: ~$200-400/year from DigiCert, Sectigo, etc.
2. **EV (Extended Validation) certificate**: ~$400-600/year, removes SmartScreen warnings immediately
3. **SignPath (free for OSS)**: [signpath.io](https://signpath.io/) offers free code signing for open source projects
4. **Azure Trusted Signing**: Microsoft's new signing service, available via Azure

### Setup for CI

Add to GitHub secrets:
| Secret | Value |
|--------|-------|
| `WINDOWS_CERTIFICATE` | Base64-encoded `.pfx` |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for the `.pfx` |

## Starting without signing

For an open-source project in early development, it's fine to distribute unsigned:

1. **macOS users**: Add to README that users should run `xattr -cr /Applications/Rubynaut.app`
2. **Windows users**: Note that SmartScreen warning is expected; click "More info" > "Run anyway"
3. **Linux users**: No issues

Add signing when the project gains traction and you want a polished distribution experience.
