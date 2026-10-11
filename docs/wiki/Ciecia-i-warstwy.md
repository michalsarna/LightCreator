🌐 [English](Cuts-and-Layers) · **Polski**

# Cięcia i warstwy

![Warstwy](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/19-layers-lock.png)

## Warstwy
* 30 kolorowych warstw. Każda ma **tryb** (Line, Fill, Fill + Line, Offset fill), prędkość, moc, liczbę przejść, interwał, kąt skanowania, overscan, skanowanie dwukierunkowe oraz przełączniki wyjścia i widoczności.
* Nowe warstwy startują z mocą 100 % i prędkością 1000 mm/min. Ustawienia warstw należą do **projektu**: plik projektu zawiera je razem z profilem urządzenia i materiałem, z którego powstał. To, co chcesz używać ponownie, zapisujesz jako **materiał** w bibliotece urządzenia; sam profil urządzenia nie przechowuje już prędkości warstw (starsze profile nadal zawierają starą listę, która jest ignorowana). Prędkości są pokazywane na sekundę lub na minutę (opcja urządzenia *Speed units*), w mm lub calach.
* **Pompa powietrza:** każda warstwa ma przełącznik *Air pump on*. G-code włącza pompę (`M8`; w Marlinie wentylator), gdy zaczyna się warstwa, która jej chce, wyłącza, gdy zaczyna się warstwa bez niej, i na końcu.
* Pasek kolorów u dołu pokazuje nazwy warstw. Dwukliknij nazwę (tam lub w zakładce **Layers**), aby otworzyć okno opcji warstwy.
* Trzy przełączniki na końcu wiersza, z nagłówkami w postaci ikon, to wyjście (wypalaj), widoczność i pompa powietrza. *Image settings* (rastrowanie, moc minimalna) pokazują się tylko dla warstw z bitmapami.
* Zakładka **Layers** wymienia warstwy w **kolejności wypalania**; przesuwaj je w górę lub w dół, by ją zmienić. Zakładka Properties pokazuje kolor warstwy zaznaczenia.

## Tryby
| Tryb | Efekt |
|---|---|
| Line | idzie po konturach |
| Fill | linie skanowania co interwał warstwy |
| Fill + Line | wypełnienie, potem kontur |
| Offset fill | koncentryczne pierścienie do środka co interwał warstwy |

Wnętrza są wycinane najpierw; kolejność wg najbliższego sąsiada.

## Obrazy rastrowe
Obrazy są grawerowane linia po linii co interwał warstwy. Wybierz **próg**, **Floyd-Steinberg**, **Jarvis**, **Stucki**, **Atkinson**, dithering **uporządkowany** lub **moc w skali szarości** (moc min / maks). Każdy obraz ma opcję negatywu.
![Grawerowanie obrazu](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/09-image.png)

## Biblioteka materiałów
**Materiał** zawiera ustawienia wszystkich potrzebnych mu warstw: tryb, prędkość, moc, przejścia, interwał, kąt skanowania, overscan, rastrowanie, moc minimalną i pompę powietrza. Współpracują trzy poziomy:

* **Biblioteka wbudowana:** wartości startowe dla typowych laserów diodowych 10 W i CO2 40 W. Jest źródłem dla pozostałych bibliotek. Wartości są ogólne: **zawsze testuj na odpadzie**.
* **Materiały globalne:** okno *Material library…* (menu Laser) jest jedynym miejscem, które rozszerza bibliotekę wbudowaną. *Save the layers of the design as a material for every device* dodaje materiał dostępny dla każdego urządzenia.
* **Biblioteka urządzenia:** każde urządzenie ma własną bibliotekę w zakładce **Materials** panelu bocznego. Zostaje raz wypełniona materiałami wbudowanymi i globalnymi pasującymi do jego typu lasera. W zakładce edytujesz wszystkie ustawienia każdej warstwy, dodajesz i usuwasz warstwy, zmieniasz nazwy, usuwasz, **stosujesz materiał do warstw** projektu i **zapisujesz warstwy projektu jako materiał**; wszystko trafia tylko do biblioteki urządzenia.

Biblioteka urządzenia jest zapisana w profilu urządzenia (więc także w kopiach zapasowych) i wewnątrz każdego zapisanego projektu. Przy otwarciu projektu jego biblioteka jest scalana z biblioteką aktywnego urządzenia: nowe materiały są dodawane, a przy różnicach okno pozwala wybrać, wartość po wartości, czy zachować swoją, czy wziąć tę z pliku.

Materiał jest **jedną pozycją dla obu rodzajów laserów**: dioda i CO2 mają własne ustawienia pod tym samym identyfikatorem. Nazwy, np. *Sklejka*, są tłumaczone, grubość jest osobną wartością, a każdy materiał ma jednoliniowy komentarz i tagi (*Cut*, *Engrave*…). Wartości wbudowane można zmienić na swoim komputerze: **kliknij dwukrotnie** materiał w oknie Material library (albo *Edit material*), ustaw każdą warstwę i zapisz; kopia nadpisuje dane wbudowane, a *Restore built-in values* przywraca oryginał. Biblioteka urządzenia dostaje wariant dla swojego rodzaju lasera. Listy są pogrupowane po kategoriach, a każdą grupę można zwinąć klikając tytuł. Zakładka Materials ma przycisk **Delete all materials**: czyści bibliotekę urządzenia i pokazuje zamiast niej bibliotekę główną, a zmieniony tam materiał zostaje zapisany jako kopia dla urządzenia. Tagi są podpowiadane z już używanych, a najechanie na *Rastrowanie* wyjaśnia wybraną metodę. Zapisany projekt zawiera profil urządzenia, bibliotekę urządzenia i materiał, z którym powstał.

![Edycja materiału](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/36-material-edit.png)

Każda warstwa materiału pamięta też swój **kolor** i **etykietę** (przeznaczenie, np. *Cut*); wybór materiału (dwukrotne kliknięcie w zakładce Materials; zielony znaczek pokazuje używany) wstawia parametry i kolory jego warstw do warstw projektu, ale nie ich nazwy, i utrzymuje oba w zgodzie: zmiana w zakładce Layers jest od razu kopiowana do materiału i co 60 sekund zapisywana w bibliotece urządzenia; zmiana materiału w jego edytorze trafia do warstw. Lista pokazuje tylko nazwy. Materiały mają **kategorię** (Drewno, Metal, Tworzywa, Papier i skóra, Kamień i szkło, Inne albo własną), a listę w zakładce Materials można po niej filtrować. Listy pokazują nazwę i tagi materiału w wierszach o naprzemiennym tle, a strzałki na brzegu oznaczają, że listę można przewinąć. Etykietę warstwy wybiera się z listy; przycisk **Layer labels…** w oknie Material library dodaje własne.

**Export library… / Import library…** (w oknie Material library i w zakładce Materials) zapisują i wczytują jeden plik `.lcb` z materiałami globalnymi i urządzeniami wraz z ich bibliotekami. Okno pozwala zaznaczyć dokładnie, co trafia do pliku lub co jest z niego brane; materiały, które się różnią, otwierają okno różnic. Zakładki panelu bocznego można przeciągać, aby zmienić ich kolejność.

![Plik biblioteki](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/35-library-export.png)
![Zakładka Materials](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/33-materials-tab.png)
![Różnice](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/34-material-merge.png)
![Materiały](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/11-materials.png)

## Okno podglądu
Otwiera się przybliżone do obiektów do wypalenia; dwie ikony w linii tytułu dopasowują widok do obszaru roboczego lub do wszystkich obiektów. Przycisk *Preview…* na pasku sterowania jest błękitny, gdy okno jest otwarte, i zamyka je po ponownym kliknięciu. Całe zadanie, a każda operacja (warstwa, wypełnienie, linia, offset, obraz, przejście) w swoim kolorze lub kolorze warstwy, przełączniki operacji, opcjonalne ruchy jałowe, zero maszyny w prawdziwym rogu, odtwarzanie w czasie zadania (15 s, czas rzeczywisty, ×10 do ×200), suwak i sumy czasu, długości, przejść i ruchów.
![Podgląd](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/12-preview.png)

## Szczelina, perforacja i mostki (warstwy liniowe)
* **Kerf offset:** przesuwa zamknięte kontury na zewnątrz o podaną odległość (do wewnątrz, gdy ujemna), by uwzględnić szerokość cięcia; otwory przesuwają się w drugą stronę. 0 = wyłączone. Używa tego tylko G-code; eksport HPGL / DXF i podgląd ścieżek pokazują kontury tak, jak je narysowano.
* **Perforacja:** kontur jest cięty przez *Cut length*, pomijany przez *Skip length* i tak dalej. Pominięte odcinki to przejazdy z wyłączonym laserem.
* **Mostki:** krótkie odcinki konturu zostają nieprzecięte albo cięte z procentem mocy warstwy (*Tab cut power*), żeby element trzymał się na miejscu. *Tab size* to długość wzdłuż konturu. *Even spacing* daje mostek co *Spacing* milimetrów (co najmniej *Min tabs*), *Tabs per shape* stałą liczbę; *Limit max tabs* ogranicza liczbę, a *Skip inner shapes* zostawia bez mostków kształty leżące wewnątrz innych. Mostki są rozmieszczane automatycznie, równomiernie wzdłuż konturu; ręczne ustawianie nie jest jeszcze możliwe.
Opcje są domyślnie wyłączone i wtedy nic nie zmieniają w G-code. Zapisują się z warstwą, z materiałem i w projekcie.

## Obrazy i tryb Obraz
Warstwa, która zawiera wyłącznie obrazy bitmapowe, jest zawsze w trybie **Obraz**; trybu nie można zmienić, ale prędkość, moc, przejścia, odstęp, kąt skanowania, overscan, rastrowanie i resztę można. Kształty wektorowe położone na takiej warstwie nie są wypalane (warstwa z obrazami i konturami zachowuje wybrany tryb). **Kąt skanowania** obraca linie wypalania obrazu o ten kąt.

Niezaimplementowane z ustawień cięcia innych programów: podwarstwy (kilka ustawień na jednym kolorze) i ręczne ustawianie mostków.
