🌐 [English](Safety-and-Limitations) · **Polski**

# Bezpieczeństwo i ograniczenia

> ⚠️ **Lasery są niebezpieczne.** Testuj G-code przy niskiej mocy, noś okulary ochronne, nigdy nie zostawiaj pracującej maszyny bez nadzoru i sprawdź w maszynie ustawienia `$30` (S-max) i `$32` (tryb lasera).

## Co jest sprawdzone
Cała logika geometrii, operacji logicznych, offsetów, rastra, G-code, eksportu i tłumaczeń jest pokryta testami jednostkowymi, a zrzuty ekranu pochodzą z prawdziwego interfejsu. Strumieniowanie GRBL jest sprawdzane od początku do końca na prawdziwym firmware GRBL 1.1h w [symulatorze](Symulator-GRBL).

## Czego nie sprawdzono
**Nic nie zostało jeszcze uruchomione na prawdziwym sprzęcie.** Nadal bez testów na maszynach: strumieniowanie GRBL i Marlin, ścieżka Ruida / Trocen, kamera na żywo, odczyt ustawień z urządzenia i import Illustratora (tylko pliki zgodne z PDF). Wartości materiałów to ogólne punkty wyjścia; testuj na odpadzie.

## Jeszcze niezaimplementowane
Bezpośrednie protokoły binarne Ruida / Trocen, kąty skanowania obrazów, kerning i kształtowanie pism złożonych (Devanagari, arabski) w tekście, korekcja zniekształceń obiektywu kamery. Kamery RTSP nie są obsługiwane (użyj MJPEG).

## Bezpieczeństwo oprogramowania
Luki zgłaszaj prywatnie przez [GitHub security advisories](https://github.com/michalsarna/LightCreator/security/advisories/new). Wspierane jest tylko najnowsze wydanie.
