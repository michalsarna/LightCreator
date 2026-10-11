🌐 [English](Devices-and-Controllers) · **Polski**

# Urządzenia i sterowniki

![Konfiguracja urządzenia](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/05-device-config.png)

## Osie, pole robocze i kontrola przed zadaniem
* **Zero maszyny** może być w każdym z czterech narożników; **Invert axes** odwraca X i/lub Y, a **Panel orientation** (natywna, obrót w lewo, obrót w prawo) obraca pole robocze tak, jak panel stoi przed tobą (pole jest pokazane obrócone, a G-code odpowiednio mapowany). Wszystko przechodzi przez jedną transformację projekt → maszyna, więc strzałki ręcznego przesuwu, znacznik zera, ramka, G-code, HPGL i DXF są spójne.
* **Marginesy** to pasy przy krawędziach, których maszyna nie może użyć (zaciski, rama); są kreskowane na kanwie. **Limity programowe** to opcjonalny własny prostokąt w polu użytecznym (obrys pomarańczowy). Przyciski przesuwu zatrzymują się na brzegu dozwolonego obszaru; głowica stojąca na zewnątrz zawsze może wrócić do środka.
* **Kontrola przed zadaniem:** Start, Frame, zapis G-code i eksport do pliku najpierw sprawdzają zadanie. Obiekt lub ścieżka (z overscanem) poza polem użytecznym lub limitami programowymi to **błąd**: nic nie jest wysyłane, obrysowywane ani eksportowane, a okno proponuje *Select objects*. Warstwy palące z mocą 0 % lub z obiektami bez ścieżki to **ostrzeżenia**, które można zatwierdzić. Zapis G-code nie przycina już cicho współrzędnych.
* **Po połączeniu** sterownik GRBL jest raz odczytywany (`$$`). Jeśli tryb lasera (`$32`) jest wyłączony, okno ostrzega, że laser może się odpalić na ruchach jałowych, i po potwierdzeniu proponuje *Enable laser mode ($32=1)*; jeśli `$30` sterownika różni się od S max profilu, można *Use the value from the controller*.

## Ustawienia G-code urządzenia
* **Prędkości:** *Max cutting speed* i *Acceleration* (0 = nieznane) obok prędkości jałowej. Z przyspieszeniem szacowanie czasu uwzględnia rampy startu i hamowania (ciąg ruchów z jedną prędkością ma jedną rampę, każdy ruch jałowy własną); prędkość cięcia jest ograniczona do maksimum; warstwa szybsza od maksimum dostaje ostrzeżenie przed zadaniem, a panel warstwy podpowiada overscan równy drodze hamowania.
* **Dialekt:** automatyczny ze sterownika albo GRBL (`M3`), GRBL dynamic (`M4`), Marlin, Smoothieware. Smoothieware jest też sterownikiem (strumieniowanie jak GRBL, ruch ręczny i ustawienia jak Marlin) i zapisuje S jako ułamek pełnej mocy (ustaw *S-value max* na 1); szczegóły to założenia z dokumentacji, nie sprawdzone na sprzęcie.
* **Strona Advanced** (z ostrzeżeniem): własny początek zadania (zastępuje jednostki, tryb bezwzględny, płaszczyznę i linię trybu lasera), własny koniec (zastępuje końcowe `M2`; laser jest zawsze wyłączany, a głowica odsyłana do zera przed nim) i polecenia air assist. Puste = wbudowane; znaki sterujące i zbyt długi tekst są odrzucane; *Restore defaults* je czyści.
* **Tryb mocy warstwy:** jak w urządzeniu, stała (`M3`) lub dynamiczna (`M4`); G-code wstawia przełączenie między warstwami (tylko dialekty GRBL).
* **Łuki (G2 / G3):** domyślnie wyłączone. Po włączeniu kontury i pierścienie są dopasowywane łukami w granicach *Arc tolerance*; linie skanowania i obrazy zostają liniami. *Curve tolerance* określa, jak drobno spłaszczane są krzywe Béziera i tekst (domyślnie 0,05 mm). Nie każdy firmware dobrze obsługuje łuki (Marlin wymaga ARC_SUPPORT), więc testuj z wyłączonym laserem.

## Zachowanie połączenia i ustawienia sterownika
* **Strumieniowanie:** *Count the buffer* (domyślne, szybkie: kilka linii w buforze sterownika) lub *Simple* (jedna linia i czekanie na jej `ok`; wolne, dla firmware odpowiadającego błędnie). Zadanie, które przez 10 s czeka na odpowiedź, gdy maszyna stoi, jest zgłaszane w konsoli i w oknie (*Keep waiting* / *Stop the job*); nic nie jest wznawiane ani wysyłane samo.
* **Wiązanie z USB:** pole portu przyjmuje nazwę portu albo `VID:PID`; *Bind to this port* zapamiętuje identyfikatory USB wybranego portu, więc port jest znajdowany po zmianie nazwy. *Connect again automatically* próbuje co kilka sekund po zerwaniu łącza, aż naciśniesz Disconnect.
* **Po połączeniu:** opcjonalnie bazowanie maszyny (`$H`, `G28`) i/lub kasowanie alarmu (`$X`, `M999`; pomija bazowanie, więc tylko gdy wiadomo, że głowica jest bezpieczna). Jeśli firmware zgłosi budowę z bazowaniem pojedynczych osi (opcja `H` w `$I`), w zakładce Device pojawią się *Home X* / *Home Y*.
* **Strona Firmware ($$)** (GRBL): odczytuje wszystkie ustawienia z opisem, jednostką i typową wartością; krytyczne (`$30`, `$31`, `$32`, `$130`–`$132`) są wyróżnione. Wartość jest sprawdzana (zakres, maska, 0/1) i zapisywana dopiero po potwierdzeniu, potem odczytywana ponownie. *Back up to a file* zapisuje linie `$n=wartość` z datą i `$I` firmware; *Restore from a file* sprawdza każdą linię, pokazuje różnice, zapisuje po potwierdzeniu i porównuje odpowiedź; *Factory settings ($RST=$)* wymaga potwierdzenia i wpisania `RESET`. Dla Marlina strona pokazuje raport `M503`.

## Profile urządzeń
Okno startowe wybiera **profil urządzenia** i go zapamiętuje. Profil zawiera sterownik, typ lasera (dioda lub CO2), jednostki (mm lub cale; geometria jest zawsze przechowywana w mm), jednostki prędkości (na sekundę lub na minutę, przeliczane automatycznie; G-code zawsze niesie posuw w mm/min po `G21`), domyślną prędkość każdej warstwy, pole robocze, zero maszyny, maks. wartość S, prędkość jałową, prędkość transmisji, port, krok i posuw ręcznego przesuwania, moc obrysu i wyrównanie kamery. Okno konfiguracji ma zmienny rozmiar i listę stron po lewej: **General** (nazwa, sterownik, jednostki, prędkości, przesuw), **Hardware** (pole robocze, zero maszyny), **Laser** (typ lasera, maks. S, moc obrysu), **G-code**, **Connection** (szeregowe lub TCP; tylko dla sterowników z połączeniem na żywo) i **Camera**. Profile zapisane przez wcześniejsze wersje są wczytywane bez zmian; ustawienia lasera są na liście głowic (na razie jedna). Edytor wymaga co najmniej jednego profilu.

## Sterowniki
| Sterownik | Obsługa |
|---|---|
| **GRBL** | strumieniowanie przez port szeregowy lub TCP: połączenie, ręczne przesuwanie, bazowanie, odblokowanie, pauza / wznowienie, stop, obrys, postęp; *Read from device* wczytuje pole robocze, S max, prędkość jałową i tryb lasera |
| **Marlin** | to samo (funkcja lasera, moc inline); odczyt posuwu |
| **Ruida**, **Trocen** | eksport HPGL / DXF z jednym piórem lub warstwą na kolor, do otwarcia w programie producenta (np. RDWorks). Wypełnienie staje się liniami kreskowania; obrazy rastrowe są pomijane. Bezpośrednie protokoły binarne nie są zaimplementowane |

## Zakładka Device i pasek sterowania
* **Start** jest aktywny, gdy laser jest podłączony (Ruida i Trocen eksportują plik), a podczas pracy zamienia się w czerwony **STOP**; **Frame** wymaga połączenia i jest wyłączony w trakcie pracy.
* W zakładce Device strzałki ręcznego przesuwania, Home, Unlock, Pause i Resume to duże przyciski-ikony jednej wielkości (pomarańczowy kwadrat anuluje ruch); STOP to szeroki czerwony przycisk pod nimi. Krok, posuw i moc obrysu są razem nad nimi.

![Zakładka Device](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/27-device-tab.png)

## Konsola
Zakładka Console pokazuje wszystko, co wysłano do sterownika i co odebrano; odpytywanie stanu jest domyślnie ukryte.
![Konsola](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/04-console.png)

## TCP i ser2net
Urządzenie może łączyć się przez sieć, np. z `ser2net` na Raspberry Pi. Ustaw *Connection type* na *Network (TCP)* i podaj host oraz port; prędkość transmisji konfiguruje się w ser2net.
```yaml
connection: &laser
  accepter: tcp,3333
  connector: serialdev,/dev/ttyUSB0,115200n81,local
```
W Linuksie **Laser → Share over network (ser2net)…** zrobi to za ciebie: wybiera port, prędkość i port TCP, zapisuje konfigurację ser2net 4.x (YAML) lub 3.x, pokazuje adres i może ją skopiować, zapisać albo zainstalować i zrestartować usługę (prosi o uprawnienia administratora przez `pkexec`, robi kopię starego pliku). Opcja tylko lokalna nadaje się do tunelu SSH; inaczej port jest otwarty dla sieci.
![ser2net](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/18-ser2net.png)

## Kamera
Podaj urządzeniu adres kamery (strumień MJPEG lub zdjęcie JPEG, http / https; RTSP nie jest obsługiwany), a pojawi się przycisk **Camera view**. Widok może pływać w oknie programu, otwierać się jako osobne okno systemu lub być przypięty jako zakładka panelu bocznego; można go obracać o 90° / 180° / 270° (ustawienie urządzenia). Obraz można powiększać (przyciski + / − lub kółko myszy, do 8×), przeciągać i przywracać cały przyciskiem *Default zoom*; trzy ikony wybierają okno pływające, osobne lub zakładkę, a adres nie jest wyświetlany. Przycisk **Overlay** (na pasku sterowania, za *Camera view*, i w oknie kamery) pokazuje obraz pod projektem; jest zawsze widoczny i wyszarzony, dopóki nie ma obrazu.
![Zakładka kamery](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/17-camera-tab.png)

**Wyrównanie nakładki:** zdjęcie lub obraz na żywo leży pod projektem, wyrównany czterema przeciąganymi rogami (korekcja perspektywy), z przezroczystością, odbiciem i obrotem; zapisywane dla urządzenia.
![Nakładka](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/10-camera-overlay.png)
