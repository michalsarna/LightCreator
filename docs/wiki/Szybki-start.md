🌐 [English](Quick-Start) · **Polski**

# Szybki start

Pierwsze zadanie krok po kroku: od okna startowego do wypalonego elementu. Każda etykieta poniżej to ta, którą pokazuje program (w nawiasie angielska); menu są opisane pozycja po pozycji na stronie [Menu](Menu).

> ⚠️ Pierwszy test zawsze przy niskiej mocy na odpadzie, w okularach ochronnych i przy maszynie. Zobacz [Bezpieczeństwo i ograniczenia](Bezpieczenstwo-i-ograniczenia).

## 1. Uruchom program i wybierz urządzenie

Zainstaluj i uruchom LightCreator ([Instalacja](Instalacja)). **Okno startowe** ma dwie części.

![Okno startowe](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/01-start.png)

* **Urządzenie** („Wybierz urządzenie, na którym będziesz pracować.”): lista twoich urządzeń z przyciskami **Dodaj urządzenie…** (Add device…), **Edytuj…**, **Powiel**, **Importuj…** i **Nie mam urządzenia** (I have no device).
* **Start** („Otwórz projekt albo zacznij pusty.”): **Nowy**, **Otwórz…**, **Pokaż plik G-code…** i ostatnio otwierane pliki. Pod spodem wybierasz **Język** i **Schemat kolorów**.

Za pierwszym razem lista jest pusta. Naciśnij **Dodaj urządzenie…**. Jedno okno pyta o podstawy:

* **Nazwa**, **Sterownik** (GRBL, Marlin, Smoothieware, Ruida, Trocen) i **Typ połączenia**: **Port** szeregowy (z przyciskiem **Odśwież**, który szuka portów od nowa) albo sieciowy **Host** i **Port TCP**.
* **Odczytaj z urządzenia** (Read from device, niebieski): gdy maszyna jest podłączona, wczytuje ze sterownika pole robocze i prędkość.
* **Jednostki**, **Jednostki prędkości**, **Obszar roboczy** (X i Y), **Zero maszyny (0,0)** (narożnik, w którym maszyna ma zero), **Maks. wartość S ($30)**, **Typ lasera** i, jeśli ją masz, **Kamera**.
* **Warto sprawdzić**: lista tego, czego okno nie mogło wiedzieć (na przykład tryb lasera i `$30` jeszcze nie odczytane).

**Dodaj urządzenie** wpisuje urządzenie na listę i je zaznacza. **Więcej ustawień…** otwiera całe okno urządzenia z tym, co wpisałeś. **Nie mam urządzenia** tworzy zwykłe urządzenie o nazwie **Dummy**, do przygotowania projektu bez maszyny.

Zaznacz urządzenie i naciśnij **Nowy**. (Gdy nie ma żadnego urządzenia, **Nowy** i **Otwórz…** tworzą i używają urządzenia Dummy.) Edytor otwiera się z pustym rysunkiem w karcie. Urządzenie sprawdzisz lub zmienisz później w **Maszyna → Ustawienia urządzenia…** (Machine → Device settings…), a inne wybierzesz przez **Maszyna → Zmień urządzenie…**. Wszystkie ustawienia są na stronie [Urządzenia i sterowniki](Urzadzenia-i-sterowniki).

## 2. Narysuj kształt albo zaimportuj projekt

* **Rysowanie:** wybierz narzędzie po lewej albo w menu **Wstawianie** (Insert), na przykład **Prostokąt (R)**, i przeciągnij je na obszarze roboczym (Shift trzyma kwadrat). **Tekst (T)**: kliknij i pisz. Rozmiar i położenie można wpisać w zakładce **Właściwości** (Properties) po prawej.
* **Import:** **Plik → Importuj → Importuj SVG…** (Ctrl+I) dodaje kształty z pliku SVG; każdy kolor pliku trafia na warstwę. To samo podmenu wczytuje obrazy, Illustrator / PDF, DXF, pliki LightBurn i UGS. Plik można też przeciągnąć na okno.
* **Gotowe obrazki:** zakładka **Clipart**, zobacz [Galeria clipartów](Galeria-clipartow).

Zaznaczaj kliknięciem albo ramką, przesuwaj przeciąganiem lub strzałkami (1 mm, Shift: 10 mm). Menu **Rozmieść** (Arrange) i wąski pasek obok obszaru roboczego wyrównują, środkują, obracają i grupują. Zobacz [Narzędzia projektowe](Narzedzia-projektowe).

![Edytor](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/02-editor-properties.png)

## 3. Rozłóż kształty na warstwy i ustaw warstwy

Kolor kształtu to jego **warstwa**, a warstwa mówi, jak go wypalić.

* Rząd kolorów pod obszarem roboczym to **pasek warstw**. Zaznacz kształty i kliknij kolor: przejdą na tę warstwę. Gdy nic nie jest zaznaczone, kliknięcie wybiera warstwę, na którą trafiają nowe kształty.
* **Dwuklik** w kolor (albo w warstwę w zakładce **Warstwy**) otwiera **Opcje warstwy** (Layer options): **Nazwa**, **Kolor**, **Wyjście (wypal tę warstwę)**, **Widoczna**, a dalej **Tryb** (**Linia** jedzie po obrysach, **Wypełnienie** skanuje wnętrze, **Wypełnienie + linia**, **Wypełnienie przesunięciem**), **Prędkość**, **Moc (%)**, **Przejścia**, **Odstęp linii** wypełnienia i więcej. Nowa warstwa zaczyna od mocy 100 % i 1000 mm/min.
* Zakładka **Warstwy** pokazuje używane warstwy (**Pokaż wszystkie 64 warstw** pokazuje pozostałe) w kolejności wypalania; przesuwaj je w górę lub w dół, aby zmienić kolejność. Jej nagłówek pokazuje **materiał**, którego ustawienia mają warstwy; tam wybierzesz inny, a zarządzasz nimi w **Laser → Materiały…**.

![Warstwy](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/03-layers.png)

Wszystkie ustawienia są na stronie [Cięcia i warstwy](Ciecia-i-warstwy).

## 4. Obejrzyj zadanie przed wypalaniem

* **Ścieżki** (Toolpaths; pasek nad obszarem roboczym, zielony, gdy włączony) rysuje ścieżki na samym rysunku.
* **Podgląd…** na tym samym pasku albo **Laser → Okno podglądu…** (Ctrl+Shift+P) otwiera kartę **Podgląd**: każda operacja w swoim kolorze (**Koloruj według**: **Operacja** albo **Warstwa**), **Pokaż przejazdy**, **Odtwarzaj** z suwakiem i prędkością (**Dopasuj do 15 s**, **Czas rzeczywisty**, ×10, ×50, ×200), lista **Operacje** z przełącznikiem przy każdej i sumy: czas, długość cięcia, przejścia i ruchy.

![Podgląd](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/12-preview.png)

* **Podgląd G-code** (Preview G-code; obok **Zapisz G-code** na pasku) tworzy program i pokazuje go jako tekst w karcie **Podgląd G-code: nazwa.gcode**, niczego nie zapisując. Słowa są pokolorowane, a tekst można edytować. **Zapisz G-code** w karcie zapisuje go do pliku, **Wygeneruj ponownie** (Make again) tworzy go od nowa z projektu (gdy są zmiany, najpierw pyta: **Stracić zmiany?**), **Analizuj** (Analyse) pokazuje jego ścieżki. Dopóki go nie edytowałeś, tekst tworzy się od nowa sam, gdy zapiszesz projekt, zmienisz jego notatki (zakładka **Opis**, która trafia do pliku jako komentarze `; Note:`), zmienisz urządzenie albo przełączysz postprocesory.

Karty można przeciągać do maksymalnie trzech paneli (**Widok → Podziel widok: obok siebie**), tak aby rysunek i podgląd stały obok siebie. Zobacz [Narzędzia projektowe](Narzedzia-projektowe#widok-i-nawigacja).

## 5. Zapisz G-code albo wyślij zadanie do maszyny

**Bez maszyny (albo dla karty pamięci czy innego programu):** **Zapisz G-code** na pasku, **Plik → Eksportuj → Eksportuj G-code…** albo **Laser → Eksportuj G-code…** (Ctrl+E). Zadanie jest najpierw sprawdzane (obiekty poza obszarem roboczym to błąd; warstwy z mocą 0 % to ostrzeżenie), potem plik jest zapisywany, przez postprocesory włączone w [Wtyczkach](Wtyczki). Dla Ruida i Trocen użyj **Plik → Eksportuj → Eksportuj PLT / DXF…** i otwórz plik w programie producenta sterownika. [Symulator GRBL](Symulator-GRBL) wykonuje zadanie bez lasera.

**Z podłączonym laserem (GRBL, Marlin, Smoothieware):**

1. W zakładce **Urządzenie** (Device) naciśnij **Połącz** (zielony). Po połączeniu GRBL jest raz odczytywany: jeśli tryb lasera (`$32`) jest wyłączony, okno ostrzega i proponuje jego włączenie.
2. W zakładce **Sterowanie** (Control) ustaw **Moc obrysu** (0 % trzyma laser wyłączony przy obrysie; 1–2 % daje słabą kropkę na laserze diodowym).
3. **Obrys** (Outline; laser jedzie po linii, która obejmuje wszystkie kształty) albo **Obrys ramka** (Outline frame; prostokąt wokół nich, to samo co **Maszyna → Obrys**) pokazuje, gdzie zadanie się wypali. Połóż tam materiał.
4. **START** sprawdza zadanie i je wysyła. W trakcie pracy **Pauza** i **Wznów** wstrzymują je i wznawiają, a szeroki czerwony **STOP** je kończy. **Maszyna → Rozpocznij zadanie** robi to samo co START; **Maszyna → Start od wiersza…** wznawia zadanie od wiersza programu.
5. Zakładka **Konsola** pokazuje wszystko, co wysłano do sterownika i od niego odebrano.

Opcje zadania pod przyciskami (**Element po elemencie**, **Tylko zaznaczenie**) i to, skąd zadanie startuje (**Start od**), opisuje strona [Urządzenia i sterowniki](Urzadzenia-i-sterowniki#zakładka-device-i-pasek-sterowania).

## 6. Kamera (jeśli urządzenie ją ma)

Gdy w ustawieniach urządzenia (strona **Kamera**) jest adres kamery, pasek pokazuje **Widok z kamery**: obraz na żywo z przyciskami **Zrób zdjęcie**, **Nagraj wideo** i nagrywaniem poklatkowym. **Nakładka** (Overlay, na pasku) kładzie zdjęcie albo obraz z kamery pod projektem, aby ułożyć pracę na materiale: gdy żadna nakładka nie jest pokazana, otwiera okno **Nakładka z kamery** (to samo co **Widok → Nakładka z…**), w którym wczytujesz obraz albo włączasz kamerę i przeciągasz jego cztery rogi na miejsce. Nakładka jest zapisywana w pliku projektu. Zobacz [Urządzenia i sterowniki](Urzadzenia-i-sterowniki#kamera).

## 7. Zapisz projekt

**Plik → Zapisz** (Ctrl+S) zapisuje projekt `.cre` z rysunkiem, warstwami, materiałem, notatkami i obrazem nakładki. Automatyczny zapis (**Ustawienia → Opcje programu…**) chroni pracę w międzyczasie. Zobacz [Ustawienia i pliki](Ustawienia-i-pliki).

Wbudowany **Krótki przewodnik** (F1) opisuje opcje w programie i ma wyszukiwarkę.
