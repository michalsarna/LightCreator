🌐 [English](Cuts-and-Layers) · **Polski**

# Cięcia i warstwy

![Warstwy](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/19-layers-lock.png)

![Materiały](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/34-material-layers.png)

## Warstwy
* 64 kolorowe warstwy. Każda ma **tryb** (Line, Fill, Fill + Line, Offset fill), prędkość, moc, liczbę przejść, interwał, kąt skanowania, overscan, skanowanie dwukierunkowe oraz przełączniki wyjścia i widoczności.
* Nowe warstwy startują z mocą 100 % i prędkością 1000 mm/min. Ustawienia warstw należą do **projektu**: plik projektu zawiera je razem z profilem urządzenia; sam profil urządzenia nie przechowuje prędkości warstw (starsze profile nadal zawierają starą listę, która jest ignorowana). Prędkości są pokazywane na sekundę lub na minutę (opcja urządzenia *Speed units*), w mm lub calach.
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

## Okno podglądu
Otwiera się przybliżone do obiektów do wypalenia; dwie ikony w linii tytułu dopasowują widok do obszaru roboczego lub do wszystkich obiektów. Przycisk *Preview…* na pasku sterowania jest błękitny, gdy okno jest otwarte, i zamyka je po ponownym kliknięciu. Całe zadanie, a każda operacja (warstwa, wypełnienie, linia, offset, obraz, przejście) w swoim kolorze lub kolorze warstwy, przełączniki operacji, opcjonalne ruchy jałowe, zero maszyny w prawdziwym rogu, odtwarzanie w czasie zadania (15 s, czas rzeczywisty, ×10 do ×200), suwak i sumy czasu, długości, przejść i ruchów.
![Podgląd](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/12-preview.png)

## Materiały
Każde urządzenie ma listę **materiałów** (co najmniej jeden: nowe urządzenie zaczyna od pustego *Materiału domyślnego* i kilku punktów startowych dla swojego typu lasera: Sklejka 3 mm, Karton, Skóra 2 mm, Papier oraz dla CO2 Akryl 3 mm. Ich nazwy są tłumaczone na język interfejsu; nazwy nadane przez Ciebie zostają bez zmian. Liczby to ogólne wartości dla typowego lasera diodowego (ok. 10 W) lub CO2 (ok. 40 W): zawsze testuj na odpadzie). Materiał ma nazwę i ustawienia używanych warstw (tryb, prędkość, moc, przejścia, odstęp, kąt skanowania, overscan, kerf, perforacja, mostki, głowica, Z, rotacja...). Materiał może mieć **kolor** (przycisk przed listą materiałów w zakładce Layers, z krzyżykiem do wyczyszczenia): jest tylko wyświetlany, jako mały kwadrat na liście, żeby odróżniać materiały, i niczego nie zmienia w wyniku. Lista jest częścią urządzenia, więc podróżuje z profilem urządzenia (także w eksportach i kopiach zapasowych) oraz z każdym zapisanym projektem, razem z warstwami.

**Zakładka Warstwy** edytuje warstwy wybranego materiału, a jej nagłówek mówi, który to. W nagłówku można wybrać inny materiał (warstwy przyjmują jego ustawienia; warstwy, których nie definiuje, wracają do domyślnych, a nazwy, kolory i przełączniki warstw zostają), dodać nowy, pusty (pod wpisaną nazwą; warstwy wracają do domyślnych, a te, których używa projekt, do niego dołączają), zmienić nazwę lub usunąć (ostatniego nie można usunąć). Warstwa, której projekt używa, a wybrany materiał jej nie ma, jest do niego od razu dodawana z ustawieniami domyślnymi; zmieniona warstwa zmienia się w materiale. Otwierany projekt wnosi swój materiał do listy urządzenia (albo go tam znajduje).

## Szczelina, perforacja i mostki (warstwy liniowe)
* **Kerf offset:** przesuwa zamknięte kontury na zewnątrz o podaną odległość (do wewnątrz, gdy ujemna), by uwzględnić szerokość cięcia; otwory przesuwają się w drugą stronę. 0 = wyłączone. Używa tego tylko G-code; eksport HPGL / DXF i podgląd ścieżek pokazują kontury tak, jak je narysowano.
* **Perforacja:** kontur jest cięty przez *Cut length*, pomijany przez *Skip length* i tak dalej. Pominięte odcinki to przejazdy z wyłączonym laserem.
* **Mostki:** krótkie odcinki konturu zostają nieprzecięte albo cięte z procentem mocy warstwy (*Tab cut power*), żeby element trzymał się na miejscu. *Tab size* to długość wzdłuż konturu. *Even spacing* daje mostek co *Spacing* milimetrów (co najmniej *Min tabs*), *Tabs per shape* stałą liczbę; *Limit max tabs* ogranicza liczbę, a *Skip inner shapes* zostawia bez mostków kształty leżące wewnątrz innych. Mostki są rozmieszczane automatycznie, równomiernie wzdłuż konturu; ręczne ustawianie nie jest jeszcze możliwe.
Opcje są domyślnie wyłączone i wtedy nic nie zmieniają w G-code. Zapisują się z warstwą i w projekcie.

## Obrazy i tryb Obraz
Warstwa, która zawiera wyłącznie obrazy bitmapowe, jest zawsze w trybie **Obraz**; trybu nie można zmienić, ale prędkość, moc, przejścia, odstęp, kąt skanowania, overscan, rastrowanie i resztę można. Kształty wektorowe położone na takiej warstwie nie są wypalane (warstwa z obrazami i konturami zachowuje wybrany tryb). **Kąt skanowania** obraca linie wypalania obrazu o ten kąt.

Niezaimplementowane z ustawień cięcia innych programów: podwarstwy (kilka ustawień na jednym kolorze) i ręczne ustawianie mostków.
