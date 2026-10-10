🌐 [English](Design-Tools) · **Polski**

# Narzędzia projektowe

![Edycja węzłów](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/07-nodes.png)

## Rysowanie
* Narzędzia: prostokąt, elipsa, linia, linia łamana i tekst; zaznaczanie, przesuwanie i skalowanie uchwytami.
* **Kształty automatyczne:** trójkąt (Y), pięcioramienna gwiazda (S), wielokąt foremny (G, od 3 do 360 boków wybieranych w okienku) i serce (K). Przeciągnij ramkę (Shift utrzymuje kwadrat); wynik to zwykły edytowalny kształt Béziera.
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
Ramka z lewej do prawej: tylko obiekty w całości w środku. Z prawej do lewej: wszystko, czego dotyka. Shift+klik dodaje. Dwukrotne Escape wraca do narzędzia zaznaczania. Menu prawego przycisku zawiera najczęstsze polecenia.

## Widok i nawigacja
Przesuwanie (H), powiększanie (Z), pomniejszanie (X), *Fit work area* (Ctrl+0), *Fit all objects* (Ctrl+9). Przytrzymaj **spację**, by przesuwać widok dowolnym narzędziem. Obszar roboczy sam dopasowuje się do okna, dopóki nie zmienisz zbliżenia ręcznie. *Settings → View options*: siatka główna i dodatkowa, kolor tła, grubość linii.
![Siatka](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/15-grid-layers.png)

## Import
* **SVG:** krzywe pozostają krzywymi Béziera; kolory trafiają na warstwy.
* **Bitmapy:** PNG, JPEG, BMP, GIF, WebP, z obrotem, odbiciem, jasnością, kontrastem, gammą, automatycznymi poziomami i negatywem.
  ![Import obrazu](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/13-image-import.png)
* **Illustrator / PDF:** pierwsza strona, ścieżki wektorowe (pliki Illustratora muszą mieć zgodność z PDF).
* **Trasowanie obrazu:** zamienia ciemne lub jasne obszary na krzywe Béziera z progiem, filtrem plamek, uproszczeniem i wygładzaniem.
  ![Trasowanie](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/14-trace.png)

## Zapis i eksport
Własne projekty `.lcr` (z osadzonymi obrazami), SVG, G-code, HPGL (`.plt`) i DXF.
