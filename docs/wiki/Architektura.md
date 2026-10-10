🌐 [English](Architecture) · **Polski**

# Architektura

Repozytorium to workspace Cargo z dwoma crate'ami.

| Crate | Przeznaczenie |
|---|---|
| `crates/lc-core` | biblioteka niezależna od interfejsu, pokryta testami jednostkowymi |
| `crates/lightcreator` | aplikacja desktopowa na [egui](https://github.com/emilk/egui) |

## lc-core
`geom`, `bezier`, `fillet` (geometria, kontury, zaokrąglanie) · `ops` (operacje logiczne, offsety) · `text` (kontury tekstu) · `raster`, `image`, `trace` (grawerowanie, dithering, trasowanie) · `gcode`, `export` (G-code, PLT / DXF) · `svg`, `pdf` (import) · `doc` (model dokumentu, `.lcr`) · `materials`, `clipart` · `controller`, `ser2net`.

## lightcreator
`app` (główny stan) · `canvas`, `nodes`, `shape_ops`, `overlay` (edycja) · `panels`, `menu`, `native_menu`, `theme`, `icons`, `fonts`, `i18n`, `guide` / `help_ui` (interfejs) · `laser` (wątek portu szeregowego / TCP dla GRBL i Marlin) · `device_ui`, `ser2net_ui`, `camera`, `camera_stream`, `stream_ui` (urządzenia, kamera) · `clipart_ui`, `clip_net`, `materials_ui`, `image_ui`, `preview_ui`, `units_ui` · `prefs` (`app.ron`) · `screenshots` (renderowanie bez okna).

`build.rs` zbiera ikony interfejsu z `assets/ui-icons` i osadza cliparty; test sprawdza, czy każda żądana ikona istnieje.

## Konwencje
* Współrzędne w **milimetrach**, początek w **lewym górnym rogu** pola roboczego. Na wyjściu oś Y jest odwracana dla maszyn z zerem z przodu po lewej (*Laser → Device settings*).
* Geometria jest zawsze przechowywana w mm; jednostki wyświetlania są ustawieniem urządzenia.

## Testy i CI
* `cargo test --workspace` obejmuje geometrię, operacje logiczne, offsety, raster, G-code, eksport i tłumaczenia.
* Zrzuty ekranu są renderowane bez okna z prawdziwego interfejsu: `cargo test -p lightcreator --release render_screenshots -- --ignored`.
* Strumieniowanie GRBL jest testowane od początku do końca w [Symulatorze GRBL](Symulator-GRBL).
* Workflowy GitHub: `tests.yml`, `release.yml` (tag `v` + wersja z `Cargo.toml` buduje paczki Windows / macOS / Linux), `codeql.yml`.

## Współtworzenie
Zgłoszenia i pull requesty są mile widziane. Pomagają: raporty błędów, testy na prawdziwych maszynach, tłumaczenia, cliparty i materiały. Zobacz [Wsparcie](Wsparcie).
