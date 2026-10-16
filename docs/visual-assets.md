# Visual assets

What NanoPrint needs from design, with sizes and formats. Vector masters (SVG) live in
`assets/brand/`; `npm run icons` renders every raster icon from them.

## Current assets

| File                                           | Kind                                                    | Source                         |
| ---------------------------------------------- | ------------------------------------------------------- | ------------------------------ |
| `assets/brand/logo-nanoprint.svg`              | Wordmark for light backgrounds                          | Design                         |
| `assets/brand/logo-nanoprint-dark.svg`         | Wordmark for dark backgrounds                           | Design                         |
| `assets/brand/isologo-nanoprint.svg`           | Isologo (brand blue `#4b5ff7`, light outline)           | Design                         |
| `assets/brand/isologo-nanoprint-mono.svg`      | Single-color isologo, "P" cut out                       | Derived from the isologo       |
| `assets/brand/isologo-nanoprint-1024.png`      | Square app icon master, 6% padding                      | `npm run icons`                |
| `src-tauri/icons/tray/tray-{16…48}.png`        | Tray, Windows and Linux (16, 20, 22, 24, 32, 40, 48 px) | `npm run icons`                |
| `src-tauri/icons/tray/tray-template{,@2x}.png` | Tray, macOS template (18 and 36 px)                     | `npm run icons`                |
| `src-tauri/icons/{icon.ico,icon.icns,*.png}`   | App icons for installers, the taskbar and the Dock      | `npm run icons` (`tauri icon`) |

The app picks the tray image that matches the display scale at startup. Still missing: tray
state badges, installer images, the macOS app icon plate and the web assets (sections 3–6).

## 1. Brand basics

| Asset                    | Format      | Notes                                                                                                                                |
| ------------------------ | ----------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| Symbol (isotype)         | SVG, square | The mark without text. It is the base of every icon below. Must read at 16 px.                                                       |
| Wordmark                 | SVG         | Already available: `NanoPrint-logo.svg` (black fill).                                                                                |
| Wordmark, light and dark | SVG         | One for light backgrounds, one for dark (GitHub, landing page, installers).                                                          |
| Monochrome symbol        | SVG         | Single color, no gradients, for the macOS menu bar and Linux panels.                                                                 |
| Color palette            | Hex + OKLCH | Primary, its foreground, and status colors (success, warning, error). The primary feeds the UI theme (`--primary` in `src/app.css`). |

Design rules for the symbol: simple silhouette, no text, no thin lines (minimum stroke ≈ 1.5 px at
16 px), recognizable in one color, keep ~10% padding inside the square.

## 2. App icon

- Master: **1024×1024 PNG with transparency** (and the SVG source). `npm run tauri icon <file>`
  generates `icon.ico`, `icon.icns`, the PNG sizes and the Windows Store tiles in `src-tauri/icons/`.
- macOS variant (optional but recommended): the symbol on a rounded-square plate following the
  macOS app icon grid (824×824 plate centered in the 1024×1024 canvas), so it matches other Mac
  apps.
- Windows and Linux use the symbol without a plate.

## 3. Tray icon

The tray icon is the main UI of NanoPrint and has the strictest constraints.

| Platform             | Style                                                                 | Sizes (px)             | Notes                                                                                                                                    |
| -------------------- | --------------------------------------------------------------------- | ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Windows 10/11        | Full color, or monochrome with light and dark variants                | 16, 20, 24, 32, 40, 48 | One size per display scale (100–300%). The taskbar can be light or dark; a color glyph with enough contrast on both avoids two variants. |
| macOS                | **Template image:** black glyph on transparent background, alpha only | 18 (@1x), 36 (@2x)     | macOS tints it for light/dark menu bars and the selected state. Glyph ≈ 14–16 px tall inside the 18 px canvas.                           |
| Linux (AppIndicator) | Full color or symbolic                                                | 22, 24, 32, 48         | Panels are usually dark; check contrast on light panels too.                                                                             |

### States

Same glyph, with a small badge in the bottom-right corner (≈ 40% of the icon) so the state is
visible without opening the menu. No animations.

| State                                        | Badge                         |
| -------------------------------------------- | ----------------------------- |
| Ready                                        | none                          |
| Paused (printing paused by the user)         | grey/dimmed glyph             |
| Attention (a site is waiting for permission) | orange dot                    |
| Error (printer offline, last job failed)     | red dot                       |
| LAN access on                                | small network mark (optional) |

The macOS template versions need the badge drawn as a cut-out or solid shape, since templates are
single-color.

## 4. Windows that show branding

- Settings window and permission prompt: the monochrome symbol as SVG (rendered with the UI text
  color) at 24–32 px.

## 5. Installers

| Installer         | Asset                                           | Size                       | Format       |
| ----------------- | ----------------------------------------------- | -------------------------- | ------------ |
| Windows NSIS      | Header image                                    | 150×57                     | BMP (24-bit) |
| Windows NSIS      | Sidebar image (welcome/finish pages)            | 164×314                    | BMP (24-bit) |
| Windows MSI (WiX) | Banner                                          | 493×58                     | BMP          |
| Windows MSI (WiX) | Dialog background                               | 493×312                    | BMP          |
| macOS DMG         | Window background with an arrow to Applications | 660×400 and 1320×800 (@2x) | PNG          |

## 6. Web presence

| Asset                    | Size             | Format                                                          |
| ------------------------ | ---------------- | --------------------------------------------------------------- |
| Favicon                  | scalable + 32×32 | SVG (with a dark-mode variant via `prefers-color-scheme`) + ICO |
| Apple touch icon         | 180×180          | PNG                                                             |
| Open Graph / social card | 1200×630         | PNG/JPG                                                         |
| GitHub social preview    | 1280×640         | PNG, under 1 MB                                                 |
| README logo              | wordmark         | SVG light + dark, used with `<picture>`                         |
