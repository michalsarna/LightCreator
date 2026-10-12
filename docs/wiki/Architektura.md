🌐 [English](Architecture) · **Polski**

# Architektura

Repozytorium to workspace Cargo z dwoma crate'ami.

| Crate | Przeznaczenie |
|---|---|
| `crates/lc-core` | biblioteka niezależna od interfejsu, pokryta testami jednostkowymi |
| `crates/lightcreator` | aplikacja desktopowa na [egui](https://github.com/emilk/egui) |

## baseCreator (wspólne z MillCreatorem)
Kod, który nie potrzebuje lasera, jest w [baseCreator](https://github.com/michalsarna/baseCreator) i przypięty do tagu w `crates/lc-core/Cargo.toml` oraz `crates/lightcreator/Cargo.toml`.
* **base-core**: `geom`, `bezier`, `arcs`, `fillet`, `snap` (geometria, kontury, zaokrąglanie) · `ops` (operacje logiczne, offsety) · `text` (kontury) · `image`, `trace`, `camimg` · `scene` (kształty, grupy, identyfikatory) · `svg`, `pdf` (czytniki) · `clipart` (biblioteka) · `units` · `grbl_settings`, `ser2net`.
* **base-ui**: `theme`, `fonts`, `icons`, `i18n` / `texts` · `menu`, `native_menu` (wpisy, pasek w oknie i macOS, standardowe polecenia edycji) · `window` (ramka, okna, przyciski), `widgets` · `dialogs` (O programie, krótki przewodnik, sprawdzanie aktualizacji) · `clipart_ui` · `editor` (płótno, narzędzia, zaznaczanie, edycja węzłów, polecenia edycji, okna obrazów, opcje widoku, pasek narzędzi, pasek układania, sekcje panelu Właściwości, elementy paska sterowania i statusu za traitem `EditorHost`) · `start` (nagłówek ekranu startowego, wiersz języka i schematu kolorów, ikona okna) · `markdown_ui`, `units_ui`, `camera`, `camera_stream`.

`lc-core` re-eksportuje moduły base-core, więc `lc_core::geom` itp. nadal działają.

## lc-core
`doc` (laserowa część dokumentu: warstwy, urządzenie, `.lcr`; scena pochodzi z base-core) · `gcode`, `export` (G-code, PLT / DXF) · `raster`, `cutopts` · `svg`, `pdf`, `clipart` (układanie importów na warstwach) · `materials`, `profiles`, `zones`, `check`, `hooks` · `controller`.

## lightcreator
`app` (główny stan, implementuje `EditorHost` i `MenuHost`) · `canvas` (oznaczenia stołu, strefy, pozycja na żywo, obraz z kamery i podgląd jako haki edytora), `shape_ops`, `overlay` · `menu` (lista akcji), `icons` (ikona każdej akcji), `native_menu`, `panels`, `i18n` (teksty), `guide`, `help_ui`, `update_ui`, `prefs`, `window_ui` · `laser` (wątek portu szeregowego / TCP dla GRBL i Marlin) · `device_ui`, `ser2net_ui`, `stream_ui` (urządzenia, kamera) · `clipart_ui` (obrazy dostarczane z programem), `materials_ui`, `preview_ui`, `print_ui`, `check_ui` · `screenshots` (renderowanie bez okna).

`build.rs` osadza cliparty. Ikony interfejsu osadza baseCreator (`assets/ui-icons`); test sprawdza, czy każda ikona żądana przez akcję istnieje.

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
