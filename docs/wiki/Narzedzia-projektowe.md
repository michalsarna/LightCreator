🌐 [English](Design-Tools) · **Polski**

# Narzędzia projektowe

![Edycja węzłów](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/07-nodes.png)

## Rysowanie
* Narzędzia: prostokąt, elipsa, linia, linia łamana i tekst; zaznaczanie, przesuwanie i skalowanie uchwytami.
* **Kształty automatyczne:** trójkąt (Y), pięcioramienna gwiazda (S), wielokąt foremny (G, od 3 do 360 boków wybieranych w okienku) serce (K) i spirala (I). Przeciągnij ramkę (Shift utrzymuje kwadrat); wynik to zwykły edytowalny kształt Béziera. Małe okno w rogu obszaru roboczego ustawia liczbę boków lub zwojów (domyślnie 3); nie ma przycisku OK i zostaje, dopóki narzędzie jest aktywne; wielokąt ma suwak od 5 do 36 boków, a spirala od 1 do 24 zwojów, obok pola liczby (3 do 360 i 1 do 30), a zmiana zmienia też ostatnio narysowany kształt.
  ![Kształty](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/16-shapes.png)
* **Grupy na pasku narzędzi:** zaznaczanie / edycja węzłów, narzędzia wstawiania, operacje na kształtach (suma, część wspólna, odejmowanie, różnica symetryczna), nawigacja, rozdzielone liniami. Podwójna strzałka na brzegu paska oznacza, że dalsze ikony są poza widokiem: przewiń, by je zobaczyć.

## Edycja węzłów (N)
Przeciągaj węzły i uchwyty, przełączaj węzeł narożny / gładki, wstawiaj (dwuklik na segmencie) i usuwaj węzły, zamieniaj segmenty na linie lub krzywe, otwieraj i zamykaj ścieżki. Otworzysz ją drugim narzędziem, przez *Arrange → Edit nodes*, menu prawego przycisku lub dwukliknięciem kształtu. Cliparty edytuje się tak samo. Tekst nie jest zamieniany automatycznie: najpierw użyj *Convert to curves*.

## Tekst (T)
Tekst na żywo z czcionek systemowych (wyszukiwanie rodziny, pogrubienie, kursywa, rozmiar, odstępy liter i wierszy, wyrównanie), z możliwością zamiany na krzywe. Devanagari i arabski nie są kształtowane.
![Tekst](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/08-text.png)

## Operacje na kształtach
* **Logiczne:** suma, część wspólna, odejmowanie, różnica symetryczna.
* **Offset:** powiększanie lub zmniejszanie konturów z zachowaniem albo zastąpieniem oryginału.
* **Zaokrąglanie rogów:** *Arrange → Round corners…* zaokrągla ostre proste narożniki zadanym promieniem; przy dwóch zaznaczonych liniach łączy je łukiem. W narzędziu węzłów pasek zaokrągla tylko zaznaczone narożniki.
  ![Zaokrąglanie](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/23-rounded-shapes.png)
* Numeryczna pozycja i rozmiar, obrót, odbicie, wyrównanie, środek stołu, **wyśrodkowanie względem siebie** (środki zaznaczonych obiektów, grupa liczy się jako jeden, trafiają w jeden punkt), tablica siatki, kopiuj / wklej / powiel.
* **Pasek wyrównania:** wąski pasek ikon między obszarem roboczym a panelem bocznym z poleceniami wyrównania, środkowania, obrotu, odbicia, grupowania, blokowania i kolejności dla zaznaczenia (nazwy w podpowiedziach). Menu *Arrange* ma te same polecenia; edycja krzywych (węzły, konwersja, zaokrąglanie rogów) jest razem, a *Adjust image* / *Trace image* osobno.
* **Grupowanie** (Ctrl+G) i **blokowanie** (Ctrl+L). Zablokowanych obiektów nie da się przesunąć, edytować ani usunąć, ale można je zaznaczyć i skopiować (kopia jest odblokowana).

## Zaznaczanie
Kwadratowe uchwyty wokół zaznaczenia zmieniają rozmiar. **Kliknij uchwyt** (bez przeciągania), a zamienią się w okrągłe uchwyty narożne, które obracają zaznaczenie wokół jego środka (Shift: co 15°), i rombowe uchwyty krawędzi, które pochylają je wzdłuż tej krawędzi; ponowne kliknięcie uchwytu wraca do zmiany rozmiaru.

**Przyciąganie do obiektów** (ikona celu obok magnesu na pasku sterowania oraz Widok > Przyciągaj do obiektów; zapisywane w ustawieniach): podczas przesuwania zaznaczenia jego lewa, środkowa i prawa linia (górna, środkowa, dolna) przeskakują na tę samą linię pobliskiego widocznego obiektu, gdy znajdą się w odległości około 8 pikseli ekranu, a magentowa linia pomocnicza pokazuje, gdzie się złączyły. Działa niezależnie od *Przyciągaj do siatki*; obiekty na ukrytych warstwach nie są celami przyciągania.

Ramka z lewej do prawej: tylko obiekty w całości w środku. Z prawej do lewej: wszystko, czego dotyka. Shift+klik dodaje. Dwukrotne Escape wraca do narzędzia zaznaczania. Menu prawego przycisku zawiera najczęstsze polecenia.

## Widok i nawigacja
* **Podzielony widok:** *View → Split view* dzieli obszar roboczy na najwyżej trzy panele, obok siebie albo jeden nad drugim (*Single view* przywraca jeden). Panel pokazuje **aktywny rysunek** (ten, który edytujesz), **inny otwarty rysunek** (żeby zobaczyć, jak coś zostało zrobione; ma własne przybliżenie, kółko i środkowy przycisk, a kliknięcie robi z niego aktywny rysunek, podczas gdy aktywny zajmuje jego miejsce), **podgląd**, **analizę G-code** albo **kamerę**. Zawartość wybierasz na belce panelu; przeciągnij podział, żeby zmienić rozmiar, dwuklik wyrównuje panele. *Show beside* w menu karty (prawy przycisk) otwiera ten rysunek w panelu. Przycisk **⧉** panelu wyjmuje go do osobnego okna, np. na drugi monitor; okno pamięta miejsce, a **Back to the main window** przywraca panel. Układ jest zapamiętywany. Przy jednym panelu nic się nie zmienia.
Przesuwanie (H), powiększanie (Z), pomniejszanie (X), *Fit work area* (Ctrl+Shift+0), *Fit all objects* (Ctrl+Shift+9). Przytrzymaj **spację**, by przesuwać widok dowolnym narzędziem. Obszar roboczy sam dopasowuje się do okna, dopóki nie zmienisz zbliżenia ręcznie. *Settings → View options*: siatka główna i dodatkowa, kolor tła, grubość linii.
![Siatka](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/15-grid-layers.png)

## Import
* **SVG:** krzywe pozostają krzywymi Béziera; kolory trafiają na warstwy.
* **Bitmapy:** PNG, JPEG, BMP, GIF, WebP, z obrotem, odbiciem, jasnością, kontrastem, gammą, automatycznymi poziomami i negatywem.
  ![Import obrazu](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/13-image-import.png)
* **Illustrator / PDF:** pierwsza strona, ścieżki wektorowe (pliki Illustratora muszą mieć zgodność z PDF).
* **DXF (`.dxf`):** *File → Import → Import DXF…* (także *Open…* i przeciągnięcie pliku) wczytuje ASCII DXF (R12 i nowsze): linie, polilinie z łukami, łuki, okręgi, elipsy, splajny i bloki (ze skalą i obrotem) jako kontury, w milimetrach (jednostka pliku jest odczytywana), łuki i okręgi jako krzywe. Linie, które się stykają (z dokładnością 0,01 mm), stają się jedną ścieżką. Warstwy DXF trafiają na warstwy programu po kolei (pierwsza na warstwę 1 i tak dalej; reszta na ostatnią). To, co nie jest linią (tekst, kreskowanie, wymiary), jest pomijane i wymienione w pasku stanu. *Arrange → Auto-join paths…* / *Close path with tolerance…* łączą i zamykają ścieżki z Twoją tolerancją, a *File → Export → Export selection as SVG… / DXF…* zapisują zaznaczone kształty.
* **Projekty LightBurn (`.lbrn2`, `.lbrn`):** *File → Import → Import LightBurn project…* (albo przeciągnięcie pliku) wczytuje prostokąty, elipsy, wielokąty, gwiazdy, ścieżki z krzywymi, grupy, tekst (jako kontury, więc wygląda tak samo bez czcionki) i bitmapy z obrotem i odbiciem. Warstwy przychodzą z rodzajem (linia, wypełnienie, wypełnienie offsetem, obraz), prędkością, mocą, przejściami i odstępem, a rysunek jest odwracany do osi Y tego programu według wysokości pola roboczego wybranego urządzenia. Kształty są dodawane do otwartego rysunku (jedno cofnięcie je usuwa); warstwy z pliku są używane dla warstw, na których nie ma jeszcze kształtów. To, co LightBurn ma, a nie jest jeszcze wczytywane (np. warstwy narzędziowe T1/T2, tekst zmienny, kody QR), jest pomijane i wymienione w komunikacie. Sprawdzone na projektach zapisanych przez LightBurn 2.1.04.
* **Pliki UGS Designer (`.ugsd`):** *File → Import → Import UGS design…* (albo przeciągnięcie pliku) wczytuje ścieżki z krzywymi, prostokąty (także z zaokrąglonymi rogami), elipsy, punkty, tekst, obrazy i grupy; ukryte obiekty też są rysowane. Każdy sposób cięcia staje się warstwą: kieszenie i wypełnienia jako *Fill*, reszta jako *Line*. UGS to program do frezowania, więc jego posuwy i głębokości nie są brane dla lasera. Rzeźbienie V, planowanie powierzchni, mapy wysokości, zakładki i głębokości początkowe nie są obsługiwane i są wymienione w komunikacie.
* **Trasowanie obrazu:** zamienia ciemne lub jasne obszary na krzywe Béziera z progiem, filtrem plamek, uproszczeniem i wygładzaniem.
  ![Trasowanie](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/14-trace.png)

## Zapis i eksport
Własne projekty `.cre` (z osadzonymi obrazami), SVG, G-code, HPGL (`.plt`) i DXF.

## Układanie, kopiowanie i generowanie

Menu **Insert** (Wstawianie) zawiera narzędzia lewej palety, które wstawiają obiekty (prostokąt, elipsa, trójkąt, gwiazda, wielokąt, serce, spirala, linia, linia łamana / pióro i tekst; używane narzędzie jest zaznaczone), a potem generatory wtyczek i clipart z internetu.

Polecenia menu *Arrange* po *Dogbone / T-bone corners…* (takie same w LightCreatorze, MillCreatorze i PenCreatorze):

* **Auto-join paths…** łączy otwarte ścieżki, których końce są bliżej niż tolerancja; **Close path with tolerance…** zamyka te, których oba końce są tak blisko. **Dogbone / T-bone corners…** dodaje wcięcie w każdym ostrym narożniku, żeby okrągły frez w niego sięgnął (frezowanie). **Export selection as SVG / DXF…** zapisuje zaznaczone kształty (wszystkie, gdy nic nie jest zaznaczone).
* **Circular array…** układa kopie wokół środka zaznaczenia na *Total angle* (360° to pełny okrąg), obrócone razem z miejscem albo tylko przesunięte; tablica siatkowa to ta z menu Arrange. Powstaje najwyżej 2000 kopii.
* **Copy along path…**: zaznacz kształty, a **na końcu** ścieżkę; kopie stoją co *Distance* wzdłuż niej i mogą się z nią obracać. **Text on path…**: zaznacz tekst, a na końcu ścieżkę; kontury tekstu są wyginane wzdłuż ścieżki od *Start* (potem to kontury, nie tekst).
* **Trim by shape…**: zaznacz linie, a **na końcu** kształt zamknięty; zostają części wewnątrz (albo na zewnątrz), przecięte tam, gdzie linie przecinają jego kontur. **Remove duplicates** usuwa kształty o tym samym konturze na tej samej warstwie (zaznaczone albo cały rysunek). **Break apart** robi z każdego konturu osobny kształt. **Simplify…** usuwa punkty leżące w granicach *Tolerance* od linii.
* **Serial text…**: zaznacz tekst; zostaje skopiowany w siatce z odliczanym `{n}` albo, z **Read a CSV file…**, raz na każdy wiersz, a `{1}`, `{2}`… to kolumny. **Barcode / QR code…** kładzie na aktywnej warstwie kod QR albo kod kreskowy Code 39 jako kontury (zrób je dość duże do zadania).
* **Nest shapes…** układa zaznaczone kształty w rzędach od rogu strony na podanym obszarze, z odstępem (według prostokątów kształtów, nie ich konturów; można pozwolić na obrót o ćwierć obrotu). **Measure…** pokazuje rozmiar, położenie, pole i długość zaznaczenia, a dla dwóch kształtów odległości między nimi.

W oknie **Trace image** obraz można przybliżać kółkiem myszy albo przyciskami **zoom in**, **zoom out** i **reset view**, a zielony przycisk **Trace** tworzy kontury. (Obraz pod rysunkiem, z pliku albo z kamery, to *Camera overlay*.)
