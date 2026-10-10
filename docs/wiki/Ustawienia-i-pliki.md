🌐 [English](Settings-and-Files) · **Polski**

# Ustawienia i pliki

Język, schemat kolorów, profile urządzeń, własne ustawienia materiałów, ostatnio używany profil i rozmiar okna są zapisane w jednym pliku, `app.ron`:

| System | Położenie |
|---|---|
| macOS | `~/Library/Application Support/lightcreator/app.ron` |
| Windows | `%APPDATA%\lightcreator\data\app.ron` |
| Linux | `$XDG_DATA_HOME/lightcreator/app.ron`, domyślnie `~/.local/share/lightcreator/app.ron` |

* Twoje cliparty leżą w folderze `clipart` obok, z plikiem `index.json` zawierającym nazwę, kategorię, źródło, licencję i autora.
* Plik jest zapisywany przy zamknięciu programu. **Usuń go, aby przywrócić domyślne ustawienia**; okno startowe znów poprosi o profil urządzenia.
* Projekty (`.lcr`), SVG i G-code zapisujesz tam, gdzie wskażesz. **File → Open recent** pokazuje ostatnie 10 otwartych lub zapisanych projektów i plików SVG (też w `app.ron`; *Clear list* czyści listę).
* Po ręcznym zapisie przez 5 sekund widać komunikat **Zapisano plik**. **Settings → Program options…** ustawia **automatyczny zapis** (domyślnie co 120 sekund): praca bez nazwy pliku trafia do katalogu tymczasowego, plik z nazwą jest zapisywany w miejscu. Zamknięcie programu z niezapisaną pracą pyta: **Save**, **Discard** (kasuje plik tymczasowy i kończy) lub **Cancel** (powrót do programu).
* **Help → Download backup…** pyta, gdzie zapisać plik zip z Twoimi ustawieniami (`app.ron`, w obecnym stanie) i całym folderem `clipart`, z README. **Help → Restore from backup…** wczytuje taki plik z powrotem (ustawienia i cliparty są zastępowane po potwierdzeniu); gdy przy starcie nie znaleziono plików profili, okno startowe proponuje to zielonym przyciskiem.
* **Aktualizacje:** przy uruchomieniu program sprawdza, czy na GitHubie jest nowsze wydanie (**Settings → Program options…** to wyłącza); **Help → Check for updates…** robi to ręcznie, a nowsza wersja otwiera stronę z paczkami.
* Okno ma **własną, płaską, prostokątną ramkę**: pasek tytułu z ikoną, tytułem i otwartym plikiem oraz przyciski minimalizacji / maksymalizacji / zamknięcia. Przeciągnij pasek, by przesunąć okno, dwuklik maksymalizuje, rozmiar zmieniasz za krawędzie (na macOS za prawą i dolną krawędź).
* Menu **Settings**: język (domyślnie angielski) i schemat kolorów (Dark, Light Dark, Medium Light, Light; domyślnie Light Dark) oraz opcje widoku (siatki, tło, grubość linii).
* Języki interfejsu (30): English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी oraz 23 kolejne języki europejskie: Български, Català, Hrvatski, Čeština, Dansk, Nederlands, Eesti, Français, Ελληνικά, Magyar, Íslenska, Gaeilge, Latviešu, Lietuvių, Malti, Norsk bokmål, Português, Română, Slovenčina, Slovenščina, Español, Svenska, Українська. Teksty języków europejskich są w `assets/lang/` (po jednym pliku na język) i powstały z pomocą tłumaczenia maszynowego, więc poprawki od native speakerów są mile widziane. Ikony to pliki SVG, a czcionki (Noto Sans, Mono, SC, Devanagari) są osadzone z `assets/`, więc wygląd jest taki sam na każdym systemie.
