🌐 [English](GRBL-Codes) · **Polski**

# Kody GRBL

Co LightCreator wysyła do sterownika GRBL i co sterownik odpowiada. Fakty o GRBL dotyczą **GRBL 1.1** (firmware większości laserów diodowych); inne wersje i odmiany (grblHAL, FluidNC, wersje producentów) mogą się różnić. Nic z tego nie było sprawdzane na prawdziwej maszynie; własne strumieniowanie programu testowano na [symulatorze GRBL](Symulator-GRBL).

## Znaki czasu rzeczywistego

Pojedyncze bajty, na które GRBL reaguje od razu, także w środku pracy. Nie są liniami i nie dostają `ok`.

| Bajt | Znaczenie | Użycie w LightCreator |
|---|---|---|
| `?` | Raport stanu | Pytanie cztery razy na sekundę; odpowiedź wypełnia stan i pozycję |
| `!` | Wstrzymanie posuwu (feed hold) | **Pause** |
| `~` | Start cyklu / wznowienie | **Resume** |
| Ctrl+X (0x18) | Miękki reset | **STOP** oraz raz po połączeniu, żeby sterownik był w znanym stanie |

Po miękkim resecie w ruchu prawdziwy GRBL 1.1 zgłasza `ALARM:3` (pozycja utracona) i wymaga `$X` lub bazowania.

## Polecenia systemowe (`$`)

| Polecenie | Znaczenie | Użycie w LightCreator |
|---|---|---|
| `$H` | Cykl bazowania | **Home**, „Home the machine” po połączeniu, **Live laser Position** włączona przy spoczynku |
| `$HX`, `$HY` | Bazowanie jednej osi (tylko gdy firmware to zgłasza) | Przyciski *Home X* / *Home Y* |
| `$X` | Zdjęcie blokady alarmu | **Unlock**, „Clear an alarm” po połączeniu. Pomija bazowanie: używaj tylko, gdy głowica stoi w bezpiecznym miejscu |
| `$$` | Wypisanie ustawień | Strona **Firmware ($$)** oraz raz po połączeniu, by sprawdzić tryb lasera (`$32`) i `$30` |
| `$N=wartość` | Zapis ustawienia | Ta sama strona, po jednej wartości, po potwierdzeniu |
| `$J=...` | Jog (ruch, który można natychmiast anulować i który nie działa w czasie pracy) | Strzałki przesuwania, np. `$J=G91 G21 X10 Y0 F3000` |
| `$G`, `$#`, `$I`, `$C`, `$N` | Stan parsera, przesunięcia współrzędnych, informacje o wersji, tryb sprawdzania, linie startowe | Nieużywane |

## G-code, który pisze program

| Kod | Znaczenie |
|---|---|
| `G21` | Milimetry |
| `G90` | Współrzędne bezwzględne |
| `G17` | Płaszczyzna XY |
| `G0 X.. Y..` | Szybki ruch (laser wyłączony) |
| `G1 X.. Y.. S.. F..` | Ruch roboczy z mocą `S` i posuwem `F` |
| `G2` / `G3` | Łuk zgodnie / przeciwnie do ruchu wskazówek zegara, gdy urządzenie zapisuje łuki |
| `G91` | Współrzędne względne, tylko wewnątrz joga |

| Kod | Znaczenie |
|---|---|
| `M3 S0` | Stała moc: laser trzyma `S`, dopóki trwa program (wiązka ogniskowania, warstwy ze stałą mocą) |
| `M4 S0` | Moc dynamiczna: moc podąża za prędkością, więc w narożnikach nie wypala głębiej (wymaga `$32=1`, trybu lasera) |
| `M5` | Laser wyłączony |
| `M8` / `M9` | Wspomaganie powietrzem włączone / wyłączone (lub własne polecenia urządzenia) |
| `M2` | Koniec programu |

`S` biegnie od 0 do **S-value max** głowicy (`$30`). Praca zaczyna się też własnymi liniami startowymi urządzenia, hakami i makrami, jeśli je podałeś.

## Ustawienia (`$n`)

Lista poniżej to ta, którą pokazuje strona Firmware; najedź na wiersz, by zobaczyć objaśnienie. Wartości oznaczone jako **krytyczne** mogą włączyć laser lub poruszyć maszynę w złą stronę, więc strona prosi o potwierdzenie przed ich zapisem.

| `$` | Nazwa | Jednostka |
|---|---|---|
| 0 | Czas impulsu kroku | µs |
| 1 | Opóźnienie wyłączenia silników | ms |
| 2 / 3 | Odwrócenie impulsu kroku / kierunku | maska |
| 4, 5, 6 | Odwrócenie pinu włączenia silników / krańcówek / sondy | 0 / 1 |
| 10 | Opcje raportu stanu | maska |
| 11 | Odchylenie na złączach (junction deviation) | mm |
| 12 | Tolerancja łuku | mm |
| 13 | Raport w calach | 0 / 1 |
| 20 | Programowe limity | 0 / 1 |
| 21 | Sprzętowe limity | 0 / 1 |
| 22 | Cykl bazowania | 0 / 1 |
| 23 | Odwrócenie kierunku bazowania | maska |
| 24 / 25 | Posuw / szukanie przy bazowaniu | mm/min |
| 26 | Odbicie styków przy bazowaniu | ms |
| 27 | Odjazd od krańcówki po bazowaniu | mm |
| 30 | Maksymalna prędkość wrzeciona (S dla 100 %), **krytyczne** | S |
| 31 | Minimalna prędkość wrzeciona, **krytyczne** | S |
| 32 | Tryb lasera, **krytyczne** | 0 / 1 |
| 100–102 | Kroki na mm osi X, Y, Z | kroki/mm |
| 110–112 | Maksymalna prędkość osi X, Y, Z | mm/min |
| 120–122 | Przyspieszenie osi X, Y, Z | mm/s² |
| 130–132 | Maksymalny przesuw osi X, Y, Z, **krytyczne** | mm |

Dla lasera: `$32=1` (tryb lasera) i `$30` równe S-value max profilu (1000 w większości płyt, 255 w niektórych). LightCreator ostrzega po połączeniu, gdy tryb lasera jest wyłączony lub `$30` różni się od profilu.

## Stany w raporcie stanu

`<Idle|MPos:10.000,20.000,0.000|FS:0,0>`: pierwsze słowo to stan.

| Stan | Znaczenie |
|---|---|
| `Idle` | Czeka na pracę |
| `Run` | Porusza się (praca, obrys, jog) |
| `Hold:0` / `Hold:1` | Wstrzymanie: 0 = zatrzymał się, 1 = jeszcze hamuje |
| `Jog` | Ruch joga |
| `Home` | Bazowanie |
| `Alarm` | Zablokowany: odblokuj `$X` albo bazuj |
| `Door:0`–`Door:3` | Stany drzwi bezpieczeństwa |
| `Check` | Tryb sprawdzania (`$C`): ruchy są tylko sprawdzane |
| `Sleep` | Uśpiony |

## Odpowiedzi

* `ok`: linia została zrozumiana i trafiła do planera. LightCreator liczy bajty linii czekających na `ok` i trzyma 128-bajtowy bufor odbiorczy sterownika pełny, ale nie przepełniony (albo wysyła po jednej linii w strumieniowaniu *Simple*).
* `error:N`: linia została odrzucona. Praca idzie dalej (linia liczy się jako odpowiedziana), a konsola pokazuje błąd.
* `ALARM:N`: maszyna stanęła i jest zablokowana.
* `[MSG:...]`, `[GC:...]`, `Grbl 1.1h ['$' for help]`: informacje, nigdy nie liczone jako odpowiedzi. Baner po resecie mówi programowi, że GRBL jest gotowy.

### `error:N`

| N | Znaczenie |
|---|---|
| 1 | Słowo G-code to litera bez liczby albo brak litery |
| 2 | Liczba ma zły format albo jej brakuje |
| 3 | Polecenie `$` nie jest znane lub obsługiwane |
| 4 | Wartość ujemna tam, gdzie potrzebna dodatnia |
| 5 | Bazowanie nie jest włączone (`$22`) |
| 6 | Czas impulsu kroku poniżej 3 µs |
| 7 | Odczyt EEPROM się nie udał; przywrócono domyślne (często raz po wgraniu firmware) |
| 8 | Polecenie `$` działa tylko, gdy GRBL jest bezczynny |
| 9 | G-code zablokowany w czasie alarmu lub joga |
| 10 | Limity programowe wymagają włączonego bazowania |
| 11 | Linia dłuższa, niż GRBL przyjmuje; została odrzucona |
| 12 | Ustawienie przekracza obsługiwaną częstotliwość kroków |
| 13 | Drzwi bezpieczeństwa są otwarte |
| 14 | Linia startowa lub informacje o wersji za długie dla EEPROM |
| 15 | Cel joga poza przesuwem maszyny |
| 16 | Jog bez `=` albo z zabronionym G-code |
| 17 | Tryb lasera wymaga wyjścia PWM wrzeciona |
| 20 | Nieobsługiwane lub błędne polecenie G-code |
| 21 | Dwa polecenia jednej grupy modalnej w linii |
| 22 | Brak ustawionego posuwu |
| 23 | Polecenie wymaga liczby całkowitej |
| 24 | Dwa polecenia, które oba wymagają słów osi, w jednej linii |
| 25 | Powtórzone słowo G-code w linii |
| 26 | Polecenie wymaga słów osi, a ich nie ma |
| 27 | Numer linii `N` poza zakresem 1 – 9 999 999 |
| 28 | Brak wymaganej wartości `P` lub `L` |
| 29 | Istnieją tylko układy G54–G59 (G59.1–G59.3 nie) |
| 30 | G53 wymaga trybu ruchu G0 lub G1 |
| 31 | Nieużyte słowa osi przy aktywnym anulowaniu trybu ruchu (G80) |
| 32 | Łuk bez słów osi w wybranej płaszczyźnie |
| 33 | Nieprawidłowy cel ruchu (łuk, który nie może istnieć) |
| 34 | Błąd matematyczny przy łuku zadanym promieniem |
| 35 | Łuk z przesunięciami bez słowa I, J lub K swojej płaszczyzny |
| 36 | Nadmiarowe słowa G-code, których żadne polecenie nie używa |
| 37 | G43.1 może przesunąć tylko skonfigurowaną oś |
| 38 | Numer narzędzia powyżej obsługiwanego maksimum |

### `ALARM:N`

| N | Znaczenie | Co zrobić |
|---|---|---|
| 1 | Zadziałała sprzętowa krańcówka; pozycja prawdopodobnie utracona | Sprawdź krańcówkę, zbazuj ponownie |
| 2 | Cel G-code poza przesuwem (limit programowy); pozycja zachowana | Wystarczy `$X`; sprawdź pracę względem obszaru roboczego |
| 3 | Reset w ruchu; pozycja nie jest gwarantowana | Zbazuj ponownie |
| 4 | Sonda: nie w stanie początkowym | Sprawdź sondę (LightCreator jej nie używa) |
| 5 | Sonda: brak kontaktu w zakresie przesuwu | Jak wyżej |
| 6 | Bazowanie nieudane: reset w trakcie cyklu | Zbazuj ponownie |
| 7 | Bazowanie nieudane: otwarto drzwi bezpieczeństwa | Zamknij je, zbazuj ponownie |
| 8 | Bazowanie nieudane: krańcówka nie zwolniła się przy odjeździe | Zwiększ `$27` lub sprawdź okablowanie |
| 9 | Bazowanie nieudane: nie znaleziono krańcówki | Sprawdź krańcówkę, kierunek (`$23`) i okablowanie |

Przycisk *Unlock* wysyła `$X`. Nie porusza maszyną: jeśli głowica ruszyła się w czasie blokady, pozycja w programie nie zgadza się z rzeczywistością, dopóki nie zbazujesz.

## Gdzie czytać dalej

* Wiki GRBL 1.1: [Commands](https://github.com/gnea/grbl/wiki/Grbl-v1.1-Commands), [Configuration](https://github.com/gnea/grbl/wiki/Grbl-v1.1-Configuration), [Laser mode](https://github.com/gnea/grbl/wiki/Grbl-v1.1-Laser-Mode) i [Jogging](https://github.com/gnea/grbl/wiki/Grbl-v1.1-Jogging).
* [Urządzenia i sterowniki](Urzadzenia-i-sterowniki) oraz [Bezpieczeństwo i ograniczenia](Bezpieczenstwo-i-ograniczenia) w tej wiki.
