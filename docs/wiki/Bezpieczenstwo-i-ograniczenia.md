🌐 [English](Safety-and-Limitations) · **Polski**

# Bezpieczeństwo i ograniczenia

> ⚠️ **Lasery są niebezpieczne.** Testuj G-code przy niskiej mocy, noś okulary ochronne, nigdy nie zostawiaj pracującej maszyny bez nadzoru i sprawdź w maszynie ustawienia `$30` (S-max) i `$32` (tryb lasera).

## Co jest sprawdzone
Cała logika geometrii, operacji logicznych, offsetów, rastra, G-code, eksportu i tłumaczeń jest pokryta testami jednostkowymi, a zrzuty ekranu pochodzą z prawdziwego interfejsu. Strumieniowanie GRBL jest sprawdzane od początku do końca na prawdziwym firmware GRBL 1.1h w [symulatorze](Symulator-GRBL).

## Czego nie sprawdzono
**Nic nie zostało jeszcze uruchomione na prawdziwym sprzęcie.** Nadal bez testów na maszynach: strumieniowanie GRBL i Marlin, oś Z i przystawka rotacyjna (model uchwytu i rolek to założenie), hooki i makra, strefy zakazane, kilka głowic lasera (`T<n>` i skalowanie S), wiązka ogniskowania, wyrównanie do wskaźnika i próba na sucho, tryb prostego strumieniowania i zgłoszenie zawieszenia, wiązanie z USB i ponowne łączenie, bazowanie / odblokowanie po połączeniu, strona Firmware ($$) (zapisy, kopia, przywracanie, ustawienia fabryczne; opcja bazowania osi wykrywana z `$I`), dialekt Smoothieware, łuki (G2 / G3), własny początek / koniec G-code, tryb mocy warstwy, zero w czterech narożnikach, odwrócone osie i obrócony panel, limity programowe i kontrola zadania, ostrzeżenia po połączeniu ($32 / $30), ścieżka Ruida / Trocen, drukowanie na papierowej drukarce (`lp`, czasownik drukowania Windows), kamera na żywo (wiele kamer, ustawienia obrazu, korekcja obiektywu, wyrównanie według punktów odniesienia), liczniki konserwacji, ogólne szablony urządzeń i pierwsze uruchomienie, odczyt ustawień z urządzenia i import Illustratora (tylko pliki zgodne z PDF). Wartości materiałów to ogólne punkty wyjścia; testuj na odpadzie.

## Serwer WWW i API
Po włączeniu (Opcje programu) serwer WWW i API **nie mają hasła ani szyfrowania**. Domyślnie nasłuchują tylko na tym komputerze; z opcją *Dostępny z innych komputerów w sieci* każdy w sieci może zobaczyć maszynę oraz ją zatrzymać lub poruszyć. Używaj tylko w sieci, której ufasz.

## Jeszcze niezaimplementowane
Bezpośrednie protokoły binarne Ruida / Trocen, kąty skanowania obrazów, kerning i kształtowanie pism złożonych (Devanagari, arabski) w tekście, automatyczna kalibracja obiektywu kamery (korekcję wpisuje się ręcznie). Kamery RTSP nie są obsługiwane (użyj MJPEG).

## Bezpieczeństwo oprogramowania
Luki zgłaszaj prywatnie przez [GitHub security advisories](https://github.com/michalsarna/LightCreator/security/advisories/new). Wspierane jest tylko najnowsze wydanie.
