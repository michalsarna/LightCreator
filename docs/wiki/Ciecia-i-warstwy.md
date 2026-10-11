🌐 [English](Cuts-and-Layers) · **Polski**

# Cięcia i warstwy

![Warstwy](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/19-layers-lock.png)

## Warstwy
* 30 kolorowych warstw. Każda ma **tryb** (Line, Fill, Fill + Line, Offset fill), prędkość, moc, liczbę przejść, interwał, kąt skanowania, overscan, skanowanie dwukierunkowe oraz przełączniki wyjścia i widoczności.
* Nowe warstwy startują z mocą 100 % i prędkością 1000 mm/min. Prędkość każdej warstwy jest przechowywana w **profilu urządzenia**: prędkość ustawiona dla warstwy staje się jej domyślną dla tego urządzenia i nowych dokumentów. Prędkości są pokazywane na sekundę lub na minutę (opcja urządzenia *Speed units*), w mm lub calach.
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

Każda warstwa materiału pamięta też swój **kolor** i **etykietę** (przeznaczenie, np. *Cut*); zastosowanie materiału ustawia parametry, kolor i nazwę jego warstw. Materiały mają **kategorię** (Drewno, Metal, Tworzywa, Papier i skóra, Kamień i szkło, Inne albo własną), a listę w zakładce Materials można po niej filtrować. Każda pozycja pokazuje swoje warstwy z kolorem i głównymi ustawieniami.

**Export library… / Import library…** (w oknie Material library i w zakładce Materials) zapisują i wczytują jeden plik `.lcb` z materiałami globalnymi i urządzeniami wraz z ich bibliotekami. Okno pozwala zaznaczyć dokładnie, co trafia do pliku lub co jest z niego brane; materiały, które się różnią, otwierają okno różnic. Zakładki panelu bocznego można przeciągać, aby zmienić ich kolejność.

![Plik biblioteki](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/35-library-export.png)
![Zakładka Materials](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/33-materials-tab.png)
![Różnice](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/34-material-merge.png)
![Materiały](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/11-materials.png)

## Okno podglądu
Otwiera się przybliżone do obiektów do wypalenia; dwie ikony w linii tytułu dopasowują widok do obszaru roboczego lub do wszystkich obiektów. Przycisk *Preview…* na pasku sterowania jest błękitny, gdy okno jest otwarte, i zamyka je po ponownym kliknięciu. Całe zadanie, a każda operacja (warstwa, wypełnienie, linia, offset, obraz, przejście) w swoim kolorze lub kolorze warstwy, przełączniki operacji, opcjonalne ruchy jałowe, zero maszyny w prawdziwym rogu, odtwarzanie w czasie zadania (15 s, czas rzeczywisty, ×10 do ×200), suwak i sumy czasu, długości, przejść i ruchów.
![Podgląd](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/12-preview.png)
