🌐 [English](Cuts-and-Layers) · **Polski**

# Cięcia i warstwy

![Warstwy](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/19-layers-lock.png)

![Materiały](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/34-material-layers.png)

## Warstwy
* 64 kolorowe warstwy. Każda ma **tryb** (Linia, Wypełnienie, Wypełnienie + linia, Wypełnienie przesunięciem), prędkość, moc, liczbę przejść, interwał, kąt skanowania, overscan, skanowanie dwukierunkowe oraz przełączniki wyjścia i widoczności. Warstwa może też mieć **opis** (do 500 znaków, w ustawieniach warstwy): do czego służy, na przykład „przecięcie na wylot, dwa przejścia”. Opis widać w podpowiedzi warstwy na liście i na pasku warstw, zapisuje się w projekcie i wędruje razem z ustawieniami warstwy w materiale.
* Nowe warstwy startują z mocą 100 % i prędkością 1000 mm/min. Ustawienia warstw należą do **projektu**: plik projektu zawiera je razem z profilem urządzenia; sam profil urządzenia nie przechowuje prędkości warstw (starsze profile nadal zawierają starą listę, która jest ignorowana). Prędkości są pokazywane na sekundę lub na minutę (opcja urządzenia **Jednostki prędkości**), w mm lub calach.
* **Pompa powietrza:** każda warstwa ma w ustawieniach **Asysta powietrzna** przełącznik **Włączona** (Air pump on). G-code włącza pompę (`M8`; w Marlinie wentylator), gdy zaczyna się warstwa, która jej chce, wyłącza, gdy zaczyna się warstwa bez niej, i na końcu.
* Rząd kolorów pod obszarem roboczym to **pasek warstw**: kliknięcie koloru przenosi zaznaczone kształty na tę warstwę (gdy nic nie jest zaznaczone, wybiera warstwę, na którą trafiają nowe kształty), a dwuklik otwiera okno **Opcje warstwy** (tak samo dwuklik w warstwę w zakładce **Warstwy**). Z **Ustawienia → Opcje widoku… → Pokaż nazwy warstw** warstwa, która ma kształty, pokazuje swoją nazwę obok koloru. Na liście warstw każdy wiersz pokazuje **tryb** jako ikonę przed prędkością (nazwa jest w podpowiedzi); kliknięcie otwiera wybór innego trybu, z wyjątkiem warstwy z obrazami, która zawsze jest w trybie **Obraz**.
* Trzy przełączniki na końcu wiersza, z nagłówkami w postaci ikon, to wyjście (wypalaj), widoczność i pompa powietrza. **Ustawienia obrazu** (rastrowanie, moc minimalna) pokazują się tylko dla warstw z bitmapami.
* Zakładka **Warstwy** wymienia używane warstwy (i aktywną; **Pokaż wszystkie 64 warstw** pokazuje resztę) w **kolejności wypalania**; przesuwaj je w górę lub w dół, by ją zmienić. Zakładka **Właściwości** pokazuje kolor warstwy zaznaczenia.

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

## Podgląd
Podgląd jest **kartą obszaru roboczego**, obok rysunków: otwiera go **Podgląd…** (Preview…) na pasku nad obszarem roboczym albo **Laser → Okno podglądu…** (Ctrl+Shift+P; przycisk jest podświetlony, gdy podgląd jest otwarty, i zamyka go po ponownym naciśnięciu), a zamyka krzyżyk karty. Jak każdą kartę można go przeciągnąć na inny panel, a jego menu prawego przycisku ma **Otwórz w osobnym oknie** (**Wróć do głównego okna** odkłada go z powrotem). Otwiera się przybliżony do obiektów do wypalenia; dwie ikony po prawej stronie jego linii tytułu dopasowują widok do wszystkich obiektów albo do obszaru roboczego. Pokazuje całe zadanie, a każdą operację (warstwa, wypełnienie, linia, offset, obraz, przejście) w jej własnym kolorze lub w kolorze jej warstwy (**Koloruj według**: **Operacja** albo **Warstwa**), przełącznik przy każdej operacji na liście **Operacje**, **Pokaż przejazdy** (domyślnie wyłączone) i odtwarzanie: **Odtwarzaj** / **Pauza**, suwak i prędkość (**Dopasuj do 15 s**: całe zadanie w 15 sekund, **Czas rzeczywisty**, ×10 (domyślnie), ×50, ×200). Pod obrazem są sumy: czas (tam, gdzie stoi suwak, z całości), długość cięcia, przejścia i ruchy. **Ścieżki** (Toolpaths) na pasku rysują te same ścieżki na samym rysunku.
![Podgląd](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/12-preview.png)

## Materiały
Każde urządzenie ma listę **materiałów** (co najmniej jeden: nowe urządzenie zaczyna od pustego *Materiału domyślnego* i kilku punktów startowych dla swojego typu lasera: Sklejka 3 mm, Karton, Skóra 2 mm, Papier oraz dla CO2 Akryl 3 mm. Ich nazwy są tłumaczone na język interfejsu; nazwy nadane przez Ciebie zostają bez zmian. Liczby to ogólne wartości dla typowego lasera diodowego (ok. 10 W) lub CO2 (ok. 40 W): zawsze testuj na odpadzie). Materiał ma nazwę i ustawienia używanych warstw (tryb, prędkość, moc, przejścia, odstęp, kąt skanowania, overscan, kerf, perforacja, mostki, głowica, Z, rotacja...). Materiał może mieć **kolor**: ustawia się go przyciskiem koloru w wierszu, który otwiera się przy dodawaniu materiału (**+**) lub zmianie nazwy (ołówek), z krzyżykiem do wyczyszczenia. Jest tylko wyświetlany, żeby odróżniać materiały (barwi ikonę przed listą i jest małym kwadratem przed każdą nazwą na liście) i niczego nie zmienia w wyniku. Lista jest częścią urządzenia, więc podróżuje z profilem urządzenia (także w eksportach i kopiach zapasowych) oraz z każdym zapisanym projektem, razem z warstwami.

**Zakładka Warstwy** edytuje warstwy wybranego materiału, a jej nagłówek mówi, który to. W nagłówku można wybrać inny materiał (warstwy przyjmują jego ustawienia; warstwy, których nie definiuje, wracają do domyślnych, a nazwy, kolory i przełączniki warstw zostają), dodać nowy, pusty (pod wpisaną nazwą; warstwy wracają do domyślnych, a te, których używa projekt, do niego dołączają), zmienić nazwę lub usunąć (ostatniego nie można usunąć). Warstwa, której projekt używa, a wybrany materiał jej nie ma, jest do niego od razu dodawana z ustawieniami domyślnymi; zmieniona warstwa zmienia się w materiale. Otwierany projekt wnosi swój materiał do listy urządzenia (albo go tam znajduje).

## Szczelina, perforacja i mostki (warstwy liniowe)
* **Kerf offset:** przesuwa zamknięte kontury na zewnątrz o podaną odległość (do wewnątrz, gdy ujemna), by uwzględnić szerokość cięcia; otwory przesuwają się w drugą stronę. 0 = wyłączone. Używa tego tylko G-code; eksport HPGL / DXF i podgląd ścieżek pokazują kontury tak, jak je narysowano.
* **Perforacja:** kontur jest cięty przez **Długość cięcia** (Cut length), pomijany przez **Długość pominięcia** (Skip length) i tak dalej. Pominięte odcinki to przejazdy z wyłączonym laserem.
* **Mostki:** krótkie odcinki konturu zostają nieprzecięte albo cięte z procentem mocy warstwy (**Moc cięcia mostka** (Tab cut power)), żeby element trzymał się na miejscu. **Rozmiar mostka** (Tab size) to długość wzdłuż konturu. **Równe odstępy** (Even spacing) daje mostek co **Odstęp** (Spacing) milimetrów (co najmniej **Min. mostków** (Min tabs)), **Mostków na kształt** (Tabs per shape) stałą liczbę; **Ogranicz maks. liczbę mostków** (Limit max tabs) ogranicza liczbę, a **Pomiń kształty wewnętrzne** (Skip inner shapes) zostawia bez mostków kształty leżące wewnątrz innych. Mostki są rozmieszczane automatycznie, równomiernie wzdłuż konturu; ręczne ustawianie nie jest jeszcze możliwe.
Opcje są domyślnie wyłączone i wtedy nic nie zmieniają w G-code. Zapisują się z warstwą i w projekcie.

## Obrazy i tryb Obraz
Warstwa, która zawiera wyłącznie obrazy bitmapowe, jest zawsze w trybie **Obraz**; trybu nie można zmienić, ale prędkość, moc, przejścia, odstęp, kąt skanowania, overscan, rastrowanie i resztę można. Kształty wektorowe położone na takiej warstwie nie są wypalane (warstwa z obrazami i konturami zachowuje wybrany tryb). **Kąt skanowania** obraca linie wypalania obrazu o ten kąt.

Niezaimplementowane z ustawień cięcia innych programów: podwarstwy (kilka ustawień na jednym kolorze) i ręczne ustawianie mostków.

**Okno materiałów.** **Laser → Materiały…** (Materials…) otwiera okno do zarządzania materiałami urządzenia: po lewej lista (kropka oznacza materiał używany przez projekt, kwadrat pokazuje jego kolor), z przyciskami dodawania materiału (**+**), kopiowania, przesuwania w górę i w dół oraz usuwania, a po prawej nazwa i kolor wybranego materiału (**Save** je zatwierdza) oraz jego warstwy ze wszystkimi ustawieniami, które można tam zmieniać, dodawać (**Add layer**) lub usuwać. **Export…** zapisuje wybrany materiał do pliku (`.json`), **Export all…** wszystkie, a **Import…** wczytuje taki plik; materiał o nazwie, która już jest na liście, zostaje pod inną nazwą (*Nazwa (2)*) albo, przy **Replace materials that have the same name**, ją zastępuje. Plik jest przy imporcie ściśle sprawdzany (rozmiar, liczba materiałów, każda liczba), a plik z nowszej wersji jest odrzucany.

![Okno materiałów](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/41-materials-window.png)
