🌐 [English](Menus) · **Polski**

# Menu

Każda pozycja każdego menu, w kolejności, w jakiej pokazuje je program, z klawiszem i krótką notką o tym, co robi. Odnośniki prowadzą do strony, która opisuje ją dokładnie. LightCreator, MillCreator i PenCreator mają ten sam układ menu:

**Plik (File) · Edycja (Edit) · Wstawianie (Insert) · Rozmieść (Arrange) · Widok (View) · Maszyna (Machine) · Laser · Ustawienia (Settings) · Pomoc (Help)**

Na macOS **Ctrl** czytaj jako **Cmd**. Wszystkie klawisze są zebrane w jednym miejscu na stronie [Skróty klawiszowe](Skroty-klawiszowe).

**Różnice między systemami**

* **Windows i Linux:** pasek menu jest rysowany w oknie programu. **Zakończ** (Quit) jest ostatnią pozycją menu **Plik**, a **O programie** (About) ostatnią pozycją menu **Pomoc**.
* **macOS:** menu są w systemowym pasku menu. Na lewo od **Plik** jest menu **LightCreator** z pozycjami **O programie LightCreator**, Services, Hide, Hide Others, Show All i **Zakończ** (Cmd+Q); **Plik** nie ma pozycji Zakończ, a **Pomoc** nie ma O programie. Po **Pomoc** jest jeszcze menu **Window** z pozycjami Minimize i Maximize. Systemowe menu pokazuje klawisz tylko przy **Nowy**, **Otwórz…**, **Zapisz**, **Zapisz jako…** i **Importuj SVG…**, ale każdy klawisz z tabel poniżej działa także na macOS (z Cmd).
* **Tylko Linux:** **Maszyna → Udostępnij w sieci (ser2net)…**.
* **W oknie startowym** działają tylko te pozycje: **Nowy**, **Otwórz…**, **Otwórz ostatnie**, **Pokaż plik G-code…**, **Zakończ**, **O programie**, **Język**, **Schemat kolorów**, **Przywróć z kopii zapasowej…** i **Sprawdź aktualizacje…**. Reszta czeka, aż otworzysz projekt.

Pozycja, której nie można teraz użyć, jest wyszarzona (na przykład **Cofnij**, gdy nie ma czego cofać, **Obrys** bez połączenia). Przełączniki (**Siatka**, **Podgląd ścieżek**…) mają znaczek, gdy są włączone.

## Plik (File)

| Pozycja | Klawisz | Co robi |
|---|---|---|
| **Nowy** (New) | Ctrl+N | Nowy, pusty rysunek w nowej karcie (do 10 kart). W oknie startowym zaczyna projekt z wybranym urządzeniem (albo z urządzeniem Dummy, gdy nie ma żadnego). Zobacz [Ustawienia i pliki](Ustawienia-i-pliki). |
| **Otwórz…** (Open…) | Ctrl+O | Otwiera projekt (`.cre`) we własnej karcie (pusta karta jest używana tak, jak jest; projekt, który jest już otwarty, zostaje tylko pokazany). Wybrany tu plik SVG lub DXF jest dodawany do otwartego rysunku. |
| **Otwórz ostatnie** (Open recent) ▸ | | Ostatnie 10 projektów i plików, które otwierałeś lub zapisywałeś; **Wyczyść listę** ją opróżnia. Plik, którego już nie ma, znika z listy. |
| **Zapisz** (Save) | Ctrl+S | Zapisuje projekt; projekt bez nazwy pliku pyta o nią. |
| **Zapisz jako…** (Save as…) | Ctrl+Shift+S | Zapisuje projekt pod nową nazwą. |
| **Importuj** ▸ **Importuj obraz…** (Import image…) | | Kładzie bitmapę (PNG, JPEG, BMP, GIF, WebP) na stronie, z oknem dostosowania. Zobacz [Narzędzia projektowe](Narzedzia-projektowe#import). |
| **Importuj** ▸ **Importuj SVG…** | Ctrl+I | Dodaje kształty pliku SVG do rysunku; kolory trafiają na warstwy. |
| **Importuj** ▸ **Importuj Adobe Illustrator / PDF…** | | Ścieżki wektorowe pierwszej strony pliku PDF lub pliku `.ai` zgodnego z PDF. |
| **Importuj** ▸ **Importuj DXF…** | | Linie, łuki, okręgi, polilinie, splajny i bloki z DXF w postaci ASCII. |
| **Importuj** ▸ **Importuj projekt LightBurn…** | | Kształty i ustawienia warstw z pliku `.lbrn2` / `.lbrn`. |
| **Importuj** ▸ **Importuj projekt UGS…** | | Kształty z pliku UGS Designer (`.ugsd`). |
| **Importuj** ▸ **Importuj wtyczką…** (Import with a plugin…) | | Wczytuje plik importerem wtyczki. Zobacz [Wtyczki](Wtyczki). |
| **Importuj** ▸ **Generuj kształt…** (Generate shape…) | | Formularz generatora kształtów z wtyczki (ta sama pozycja jest w menu **Wstawianie**). |
| **Eksportuj** ▸ **Eksportuj G-code…** (Export G-code…) | | Sprawdza zadanie i zapisuje G-code do pliku, przez włączone postprocesory. To samo co przycisk **Zapisz G-code** i **Laser → Eksportuj G-code…**. |
| **Eksportuj** ▸ **Eksportuj SVG…** | | Zapisuje rysunek jako SVG. |
| **Eksportuj** ▸ **Eksportuj PLT / DXF…** | | Sprawdza zadanie i zapisuje HPGL (`.plt`) lub DXF dla programów sterowników Ruida i Trocen. Zobacz [Urządzenia i sterowniki](Urzadzenia-i-sterowniki#sterowniki). |
| **Eksportuj** ▸ **Eksportuj zaznaczenie jako SVG…** / **Eksportuj zaznaczenie jako DXF…** | | Zapisuje zaznaczone kształty (wszystkie, gdy nic nie jest zaznaczone). |
| **Pokaż plik G-code…** (View G-code file…) | | Otwiera plik G-code w analizie G-code (karta z jego ścieżkami). |
| **Drukuj…** (Print…) | Ctrl+P | Drukuje projekt na drukarce papierowej albo zapisuje go jako PDF. Zobacz [Ustawienia i pliki](Ustawienia-i-pliki#drukowanie). |
| **Zakończ** (Quit) | Ctrl+Q | Zamyka program; o niezapisaną pracę pyta karta po karcie. Tylko Windows i Linux (na macOS jest w menu LightCreator). |

## Edycja (Edit)

| Pozycja | Klawisz | Co robi |
|---|---|---|
| **Cofnij** (Undo) | Ctrl+Z | Cofa ostatnią zmianę. |
| **Ponów** (Redo) | Ctrl+Y (także Ctrl+Shift+Z) | Wykonuje ponownie to, co cofnięto. |
| **Kopiuj** (Copy) | Ctrl+C | Kopiuje zaznaczenie. |
| **Wklej** (Paste) | Ctrl+V | Wkleja to, co skopiowano. |
| **Powiel** (Duplicate) | Ctrl+D | Kopia zaznaczenia od razu. |
| **Usuń** (Delete) | Delete lub Backspace | Usuwa zaznaczenie (w edycji węzłów: zaznaczone węzły). Zablokowane obiekty nie są usuwane. |
| **Zaznacz wszystko** (Select all) | Ctrl+A | Zaznacza wszystkie obiekty. |

## Wstawianie (Insert)

Narzędzia lewej palety, które kładą nowe obiekty na stronie; używane narzędzie ma znaczek. Wybranie jednego działa tak samo jak kliknięcie jego przycisku albo jego klawisz. Zobacz [Narzędzia projektowe](Narzedzia-projektowe#rysowanie).

| Pozycja | Klawisz | Co robi |
|---|---|---|
| **Prostokąt (R)** | R | Przeciągnij prostokąt (Shift: kwadrat). |
| **Elipsa (E)** | E | Przeciągnij elipsę (Shift: koło). |
| **Trójkąt (Y)** | Y | Przeciągnij trójkąt. |
| **Gwiazda (S)** | S | Przeciągnij pięcioramienną gwiazdę. |
| **Wielokąt (G)** | G | Przeciągnij wielokąt foremny; małe okienko ustawia liczbę boków. |
| **Serce (K)** | K | Przeciągnij serce. |
| **Spirala (I)** | I | Przeciągnij spiralę; małe okienko ustawia liczbę zwojów. |
| **Linia (L)** | L | Przeciągnij prostą linię. |
| **Linia łamana / pióro (P)** | P | Klikaj punkt po punkcie; prawy przycisk, **Enter** albo dwuklik kończy linię. |
| **Tekst (T)** | T | Kliknij, aby postawić tekst. |
| **Generuj kształt…** (Generate shape…) | | Formularz generatora kształtów z wtyczki. Zobacz [Wtyczki](Wtyczki). |
| **Cliparty z internetu…** (Online clipart…) | | Otwiera zakładkę **Clipart** z wyszukiwaniem w internetowych zbiorach ikon. Zobacz [Galeria clipartów](Galeria-clipartow). |

## Rozmieść (Arrange)

Polecenia działają na zaznaczeniu. Większość z nich jest też w menu prawego przycisku na obszarze roboczym i na pasku rozmieszczenia obok niego. Zobacz [Narzędzia projektowe](Narzedzia-projektowe).

| Pozycja | Klawisz | Co robi |
|---|---|---|
| **Wyrównaj do lewej**, **Wyśrodkuj w poziomie**, **Wyrównaj do prawej**, **Wyrównaj do góry**, **Wyśrodkuj w pionie**, **Wyrównaj do dołu** | | Ustawia zaznaczone obiekty równo przy jednej krawędzi albo na ich środku. |
| **Wyśrodkuj na stole** (Centre on bed) | | Przenosi zaznaczenie na środek obszaru roboczego. |
| **Wyśrodkuj względem siebie** (Centre on each other) | | Kładzie środki zaznaczonych obiektów w jednym punkcie (grupa liczy się jako jeden obiekt); wymaga co najmniej dwóch. |
| **Odbij w poziomie** / **Odbij w pionie** | | Odbija zaznaczenie lustrzanie. |
| **Obróć 90° w prawo** / **Obróć 90° w lewo** | | Obraca zaznaczenie o ćwierć obrotu. |
| **Grupuj** (Group) | Ctrl+G | Łączy zaznaczenie w grupę. |
| **Rozgrupuj** (Ungroup) | Ctrl+Shift+G | Rozbija grupę. |
| **Zablokuj** (Lock) | Ctrl+L | Zablokowanych obiektów nie można przesuwać, edytować ani usuwać, ale można je zaznaczać i kopiować. |
| **Odblokuj** (Unlock) | Ctrl+Shift+L | Odblokowuje zaznaczenie. |
| **Przesuń na wierzch** / **Przesuń na spód** | | Zmienia kolejność obiektów. |
| **Edytuj węzły** (Edit nodes) | N | Przełącza na narzędzie węzłów. Zobacz [Narzędzia projektowe](Narzedzia-projektowe). |
| **Zamień na ścieżkę** (Convert to path) | | Zamienia zaznaczone kształty w zwykłe ścieżki z prostych odcinków (ich obrót i rozmiar stają się częścią obrysu). |
| **Suma**, **Część wspólna**, **Odejmij**, **Różnica symetryczna** | | Operacje logiczne na zaznaczonych kształtach. |
| **Przesuń obrys…** (Offset shape…) | | Powiększa lub zmniejsza obrysy. |
| **Zaokrąglij rogi…** (Round corners…) | | Zaokrągla ostre narożniki promieniem. |
| **Zamień na krzywe** (Convert to curves) | | Zamienia zaznaczone kształty (także tekst, nie obrazy) w edytowalne krzywe Béziera. |
| **Połącz ścieżki automatycznie…** (Auto-join paths…) | | Łączy otwarte ścieżki, których końce są bliżej niż tolerancja. |
| **Zamknij ścieżkę z tolerancją…** (Close path with tolerance…) | | Zamyka ścieżki, których dwa końce są tak blisko. |
| **Narożniki dogbone / T-bone…** | | Wcięcie w każdym ostrym narożniku (do frezowania). |
| **Tablica siatki…** (Grid array…) | | Kopie zaznaczenia w wierszach i kolumnach. |
| **Tablica kołowa…** (Circular array…) | | Kopie wokół środka zaznaczenia. |
| **Kopiuj wzdłuż ścieżki…** (Copy along path…) | | Kopie wzdłuż ścieżki (ścieżkę zaznacz na końcu). |
| **Tekst na ścieżce…** (Text on path…) | | Wygina obrysy tekstu wzdłuż ścieżki (ścieżkę zaznacz na końcu). |
| **Przytnij kształtem…** (Trim by shape…) | | Zostawia części linii wewnątrz (lub na zewnątrz) zamkniętego kształtu zaznaczonego na końcu. |
| **Usuń duplikaty** (Remove duplicates) | | Usuwa kształty o tym samym obrysie na tej samej warstwie. |
| **Rozdziel** (Break apart) | | Osobny kształt z każdego obrysu. |
| **Uprość…** (Simplify…) | | Usuwa punkty leżące w granicach tolerancji od linii. |
| **Tekst seryjny…** (Serial text…) | | Kopie tekstu z liczbą zwiększaną o jeden albo jedna kopia na każdy wiersz pliku CSV. |
| **Kod kreskowy / QR…** (Barcode / QR code…) | | Kod QR albo kod kreskowy Code 39 jako obrysy. |
| **Rozmieść kształty (nesting)…** (Nest shapes…) | | Układa zaznaczone kształty ciasno na zadanym polu. |
| **Pomiar…** (Measure…) | | Rozmiar, położenie, pole i długość zaznaczenia oraz odległości między dwoma kształtami. |
| **Dostosuj obraz…** (Adjust image…) | | Jasność, kontrast, gamma i reszta ustawień zaznaczonego obrazu. |
| **Wektoryzuj obraz…** (Trace image…) | | Zamienia zaznaczony obraz w obrysy. |

## Widok (View)

| Pozycja | Klawisz | Co robi |
|---|---|---|
| **Siatka** (Grid) | | Pokazuje lub ukrywa siatkę. |
| **Przyciągaj do siatki** (Snap to grid) | | Przesuwane i rysowane punkty przeskakują do siatki. |
| **Przyciągaj do obiektów** (Snap to objects) | | Przesuwane zaznaczenie wyrównuje się do krawędzi i środków pobliskich obiektów. |
| **Powiększ** / **Pomniejsz** (Zoom in / Zoom out) | Ctrl++ / Ctrl+- | Zbliża lub oddala wokół środka widoku. |
| **Dopasuj stół do okna** (Fit bed to window) | Ctrl+Shift+0 | Dopasowuje obszar roboczy do okna i trzyma go dopasowanym, dopóki nie zmienisz powiększenia ręcznie. |
| **Dopasuj do wszystkich obiektów** (Fit all objects) | Ctrl+Shift+9 | Pokazuje wszystkie obiekty. |
| **Automatycznie dopasuj obszar roboczy do okna** (Auto-fit work area to window) | | Przełącznik: obszar roboczy jest dopasowany do okna także wtedy, gdy okno zmienia rozmiar. |
| **Podziel widok: obok siebie** (Split view: side by side) | | Dzieli obszar roboczy na panele obok siebie (do trzech). Drugi panel powstaje, gdy jest tylko jeden i ma co najmniej dwie karty. Zobacz [Narzędzia projektowe](Narzedzia-projektowe#widok-i-nawigacja). |
| **Podziel widok: jeden nad drugim** (Split view: one above the other) | | To samo, z panelami jeden nad drugim. |
| **Jeden widok** (Single view) | | Zbiera wszystkie karty z powrotem w jednym panelu. |
| **Podgląd ścieżek** (Toolpath preview) | | Rysuje ścieżki na rysunku; to samo co przycisk **Ścieżki**. |
| **Widok z kamery** (Camera view) | | Pokazuje obraz na żywo z kamery urządzenia; wymaga adresu kamery w ustawieniach urządzenia. Zobacz [Urządzenia i sterowniki](Urzadzenia-i-sterowniki#kamera). |
| **Pokaż nakładkę z kamery** (Show camera overlay) | | Pokazuje lub ukrywa obraz nakładki pod projektem; wyszarzona, dopóki nie ma nakładki. |
| **Nakładka z…** (Overlay from…) | | Otwiera okno **Nakładka z kamery**: wczytaj obraz albo włącz kamerę, a potem go wyrównaj. |
| **Pokaż strefy zakazane** (Show no-go zones) | | Rysuje na czerwono strefy zakazane urządzenia; wyszarzona, gdy urządzenie ich nie ma. |

## Maszyna (Machine)

| Pozycja | Klawisz | Co robi |
|---|---|---|
| **Zmień urządzenie…** (Switch device…) | | Wraca do okna startowego, aby wybrać inne urządzenie. |
| **Ustawienia urządzenia…** (Device settings…) | | Okno urządzenia ze wszystkimi stronami. Zobacz [Urządzenia i sterowniki](Urzadzenia-i-sterowniki). |
| **Kalibracja…** (Calibration…) | | Wylicza liczbę kroków na milimetr osi ze zmierzonego ruchu oraz prostopadłość osi z dwóch zmierzonych przekątnych; może zapisać nową wartość w sterowniku. Wymaga połączenia. Zobacz [Urządzenia i sterowniki](Urzadzenia-i-sterowniki#kalibracja). |
| **Udostępnij w sieci (ser2net)…** (Share over network (ser2net)…) | | Tylko Linux: udostępnia lokalny port szeregowy przez TCP za pomocą ser2net. Zobacz [Urządzenia i sterowniki](Urzadzenia-i-sterowniki#tcp-i-ser2net). |
| **Obrys** (Frame) | | Laser objeżdża prostokąt wokół zaznaczonych kształtów (wszystkich, gdy nic nie jest zaznaczone) z mocą obrysu; to samo co **Obrys ramka** (Outline frame) w zakładce **Sterowanie**, a nie przycisk **Obrys** (Outline) obok niego, który objeżdża kształty jak gumka. Wymaga połączenia i braku trwającego zadania. |
| **Rozpocznij zadanie** (Start job) | | To samo co **START**: sprawdza zadanie i wysyła je do lasera (dla Ruida i Trocen zamiast tego eksportuje plik PLT / DXF). |
| **Start od wiersza…** (Start from line…) | | Zaczyna zadanie od wybranego wiersza programu; maszyna najpierw dostaje stan, jaki program ma w tym wierszu. Wymaga połączenia. |

## Laser

| Pozycja | Klawisz | Co robi |
|---|---|---|
| **Materiały…** (Materials…) | | Okno materiałów urządzenia: materiały, ustawienia ich warstw, eksport i import. Zobacz [Cięcia i warstwy](Ciecia-i-warstwy#materiały). |
| **Okno podglądu…** (Preview window…) | Ctrl+Shift+P | Otwiera podgląd zadania jako kartę obszaru roboczego. Zobacz [Cięcia i warstwy](Ciecia-i-warstwy#podgląd). |
| **Eksportuj G-code…** (Export G-code…) | Ctrl+E | To samo co **Plik → Eksportuj → Eksportuj G-code…** i przycisk **Zapisz G-code**. |

## Ustawienia (Settings)

| Pozycja | Klawisz | Co robi |
|---|---|---|
| **Opcje widoku…** (View options…) | | Siatki, kolory obszaru roboczego i wokół niego, grubość linii, nazwy warstw, nazwy zmiennych, podgląd G-code po zapisaniu. Zobacz [Narzędzia projektowe](Narzedzia-projektowe#widok-i-nawigacja). |
| **Opcje programu…** (Program options…) | | Automatyczny zapis, szukanie aktualizacji przy starcie, serwer WWW i API. Zobacz [Ustawienia i pliki](Ustawienia-i-pliki) oraz [Serwer WWW i API](Serwer-WWW-i-API). |
| **Wtyczki…** (Plugins…) | | Instalowanie, włączanie i usuwanie wtyczek; oficjalna lista. Zobacz [Wtyczki](Wtyczki). |
| **Język** (Language) ▸ | | Jeden z 30 języków interfejsu. |
| **Schemat kolorów** (Colour scheme) ▸ | | Jeden ze schematów kolorów. |

## Pomoc (Help)

| Pozycja | Klawisz | Co robi |
|---|---|---|
| **Krótki przewodnik** (Quick guide) | F1 | Przewodnik w programie, z wyszukiwarką. |
| **Strona projektu na GitHubie** (Project page on GitHub) | | Otwiera stronę projektu w przeglądarce. |
| **Pobierz kopię zapasową…** (Download backup…) | | Zip z twoimi ustawieniami i clipartami. Zobacz [Ustawienia i pliki](Ustawienia-i-pliki). |
| **Przywróć z kopii zapasowej…** (Restore from backup…) | | Wczytuje taki zip z powrotem. |
| **Eksportuj diagnostykę (zip)…** (Export diagnostics (zip)…) | | Zip do dołączenia do zgłoszenia błędu. |
| **Sprawdź aktualizacje…** (Check for updates…) | | Szuka nowszej wersji od razu. |
| **O programie** (About) | | Wersja programu i baseCreator, odnośniki i licencja. Tylko Windows i Linux (na macOS: **LightCreator → O programie LightCreator**). |
