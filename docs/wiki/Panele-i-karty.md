🌐 [English](Panes-and-Tabs) · **Polski**

# Panele i karty

Jak ułożyć obszar roboczy programu LightCreator: rysunek obok podglądu, dwa rysunki naraz, podgląd na drugim monitorze. Wszystko tutaj działa tak samo w LightCreatorze, MillCreatorze i PenCreatorze.

```
+--------------------------------------------+------------------+
| rysunek A | rysunek B | + || Podgląd | +    | Właściwości | ... |
|---------------------------||----------------|                  |
|                           ||                |   panel boczny   |
|         panel 1           ||    panel 2     | (własne zakładki)|
|                           ||                |                  |
+--------------------------------------------+------------------+
```

## Co to jest karta

Obszar roboczy pokazuje **karty**, na pasku nad nim:

* każdy otwarty **rysunek** (kropka na karcie oznacza niezapisane zmiany);
* **Podgląd**;
* **Podgląd G-code**: G-code projektu jako tekst;
* **Analiza G-code** pliku;
* **Widok z kamery**, gdy jest pokazany jako karta w obszarze roboczym.

**+** na końcu paska tworzy nowy rysunek. Każda karta istnieje raz: otwarcie projektu, który już jest otwarty, pokazuje jego kartę, a projekt ma jeden podgląd, jeden tekst G-code i jeden widok z kamery.

**Panel** to część obszaru roboczego z własnym paskiem kart. Paneli może być **najwyżej trzy**. Przy jednym panelu (tak jest na początku) widać po prostu jeden pasek.

## Drugi panel

Najpierw otwórz co najmniej **dwie karty**: panel potrzebuje własnej karty, więc przy jednej karcie nic się nie stanie. Potem albo:

* **Widok → Podziel widok: obok siebie** lub **Widok → Podziel widok: jeden nad drugim**: ostatnia karta z paska trafia do nowego panelu; albo
* kliknij prawym przyciskiem kartę, którą chcesz przenieść, i wybierz **Przenieś na panel 2**.

## Trzeci panel

Kliknij prawym przyciskiem kartę w panelu, który ma co najmniej dwie karty, i wybierz **Przenieś na panel 3**. Pozycje **Podziel widok** tworzą tylko drugi panel.

## Przenoszenie karty

* **Przeciągnij kartę** na pasek innego panelu: kreska pokazuje, gdzie trafi. Możesz też upuścić ją na środek tego panelu (panel dostaje ramkę): karta trafia na koniec jego paska.
* Przeciągnij ją wzdłuż jej paska, żeby zmienić kolejność.
* Albo użyj jej menu (prawy przycisk): **Przenieś na panel N**.

Przeciąganie przenosi kartę tylko między istniejącymi panelami: nowy panel powstaje z menu.

## Rozmiary i kierunek

* **Przeciągnij linię między dwoma panelami**, żeby zmienić ich rozmiary. Panel nie zrobi się mniejszy niż 160 punktów.
* **Kliknij linię dwukrotnie**, żeby wyrównać panele.
* Przy dwóch lub trzech panelach **Widok → Podziel widok: …** zmienia kierunek: obok siebie albo jeden nad drugim. Karty zostają na swoich miejscach.

## Powrót do jednego panelu

* **Widok → Jeden widok** zbiera wszystkie karty z powrotem w jednym panelu.
* Zamknięcie ostatniej karty panelu zamyka panel, a pozostałe się zsuwają. Tak samo działa przeniesienie jego ostatniej karty gdzie indziej.
* Zamknięcie ostatniego otwartego rysunku (po pytaniu o zapis, gdy ma niezapisane zmiany) pyta **Zamknąć program?**: **Zamknij program** zamyka go, **Wróć do ekranu startowego** (albo Escape) pokazuje ekran startowy.

## Zamykanie karty

Kliknij jej krzyżyk, kliknij ją **środkowym przyciskiem** albo wybierz **Zamknij kartę** z jej menu (prawy przycisk). Rysunek z niezapisanymi zmianami najpierw zapyta.

## Na którym rysunku działają narzędzia

Jeden rysunek jest **aktywny**: na nim działają narzędzia, menu i klawisze. Rysunek w innym panelu jest pokazany tylko do oglądania, z własnym przybliżeniem. **Kliknij go albo jego kartę, a stanie się aktywny.**

## Karta w osobnym oknie

**Podgląd**, **tekst G-code** i **Widok z kamery** można przenieść do osobnego okna, na przykład na drugi monitor. Rysunku nie można.

1. Kliknij kartę prawym przyciskiem i wybierz **Otwórz w osobnym oknie**.
2. Przesuń okno, gdzie chcesz. Okno pamięta swoje miejsce i rozmiar.
3. **Wróć do głównego okna**, na górze okna, albo zamknięcie okna przywraca kartę do ostatnio używanego panelu.

**Widok → Jeden widok** nie rusza tych okien.

## Trzy sposoby pracy

* **Rysunek i podgląd obok siebie:** otwórz podgląd (**Laser → Okno podglądu…** albo przycisk **Podgląd…**; Ctrl+Shift+P), a potem **Widok → Podziel widok: obok siebie**. Sprawdź, czy do panelu 2 trafiła karta podglądu; jeśli nie, przeciągnij ją tam.
* **Dwa rysunki naraz:** otwórz oba, a potem **Widok → Podziel widok: obok siebie**. Kliknij ten, który chcesz edytować: drugi zostaje na widoku z własnym przybliżeniem.
* **Podgląd na drugim monitorze:** kliknij prawym przyciskiem kartę podglądu, **Otwórz w osobnym oknie**, i przeciągnij okno na ten monitor.

## Panel boczny

Panel po prawej to coś innego niż panele obszaru roboczego. Ma własne zakładki: **Właściwości**, **Warstwy**, **Urządzenie**, **Sterowanie**, **Konsola**, **Clipart**, **Opis** i **Nagrywanie** (panel dla każdej kamery, zdjęcia z boku, długie linie), a także kamerę lub wtyczkę, gdy są potrzebne (te dwie zostają na końcu rzędu).

* **Przeciągnij zakładkę** w jej rzędzie, żeby zmienić kolejność. Kolejność jest zapamiętywana.
* **Przeciągnij lewą krawędź** panelu, żeby zmienić jego szerokość.
* Strzałka na początku rzędu **ukrywa panel**: nazwy zakładek zostają na wąskim pasku, pisane pionowo. Kliknij nazwę, żeby otworzyć panel na tej zakładce; przycisk na pasku (**Pokaż panel**) też go otwiera.

Zakładki panelu bocznego i karty obszaru roboczego się nie mieszają: nie można przeciągnąć jednych do drugich.

## Co jest zapamiętywane

Panele, karty w nich, kierunek i rozmiary zapisują się w ustawieniach programu i wracają przy następnym uruchomieniu, bez kart tego, co nie jest już otwarte. Tak samo kolejność zakładek panelu bocznego i to, czy jest ukryty.

Zobacz też: [Narzędzia projektowe](Narzedzia-projektowe), [Skróty klawiszowe](Skroty-klawiszowe), [Menu](Menu).
