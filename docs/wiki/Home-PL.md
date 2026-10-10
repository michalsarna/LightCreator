🌐 [English](Home) · **Polski**

<p align="center"><img src="https://raw.githubusercontent.com/michalsarna/LightCreator/main/assets/icons/icon-256.png" alt="LightCreator" width="128"></p>

# LightCreator

Otwartoźródłowy odpowiednik [LightBurn](https://lightburnsoftware.com/) dla grawerek i ploterów laserowych CNC, napisany w Rust. Projektuj, ustawiaj cięcie i grawerowanie, podglądaj ścieżkę i wysyłaj ją do maszyny, na macOS, Windows i Linuksie.

![Edytor](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/06-light-polish.png)

## Od czego zacząć

| Chcę… | Przejdź do |
|---|---|
| Zainstalować program | [Instalacja](Instalacja) |
| Wykonać pierwsze wypalanie | [Szybki start](Szybki-start) |
| Rysować, edytować i importować | [Narzędzia projektowe](Narzedzia-projektowe) |
| Ustawić prędkość, moc, wypełnienie, obrazy | [Cięcia i warstwy](Ciecia-i-warstwy) |
| Podłączyć laser lub kamerę | [Urządzenia i sterowniki](Urzadzenia-i-sterowniki) |
| Użyć gotowych obrazków | [Galeria clipartów](Galeria-clipartow) |
| Sprawdzić skrót klawiszowy | [Skróty klawiszowe](Skroty-klawiszowe) |
| Znaleźć plik ustawień | [Ustawienia i pliki](Ustawienia-i-pliki) |
| Poznać kod | [Architektura](Architektura) |
| Testować bez lasera | [Symulator GRBL](Symulator-GRBL) |
| Wiedzieć, co jest sprawdzone | [Bezpieczeństwo i ograniczenia](Bezpieczenstwo-i-ograniczenia) |

## W skrócie

* **Projektowanie:** kształty, edycja węzłów Béziera, tekst, operacje logiczne, offset, zaokrąglanie rogów, grupowanie, blokowanie; import SVG / Illustrator / PDF i bitmap, trasowanie obrazów.
* **Grawerowanie:** linia, wypełnienie, wypełnienie offsetem i obrazy rastrowe z sześcioma metodami ditheringu i mocą w skali szarości; 30 warstw cięcia w stylu LightBurn; podgląd z szacowanym czasem.
* **Urządzenia:** profil dla każdej maszyny; GRBL i Marlin przez port szeregowy lub TCP (ser2net); Ruida i Trocen przez eksport PLT / DXF.
* **Pomocnicze:** biblioteka materiałów, nakładka z kamery z wyrównaniem czterech rogów, galeria 310 clipartów z wyszukiwaniem online.
* **Interfejs:** siedem języków (English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी) i cztery schematy kolorów.

> ⚠️ **Bezpieczeństwo:** lasery są niebezpieczne. Testuj G-code przy niskiej mocy, noś okulary ochronne, nie zostawiaj pracującej maszyny bez nadzoru. Zobacz [Bezpieczeństwo i ograniczenia](Bezpieczenstwo-i-ograniczenia).

Najnowsza wersja: **v0.0.5** · [Wydania](https://github.com/michalsarna/LightCreator/releases) · [Zgłoszenia](https://github.com/michalsarna/LightCreator/issues) · licencja MIT
