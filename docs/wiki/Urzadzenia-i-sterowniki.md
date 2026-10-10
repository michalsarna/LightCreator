🌐 [English](Devices-and-Controllers) · **Polski**

# Urządzenia i sterowniki

![Konfiguracja urządzenia](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/05-device-config.png)

## Profile urządzeń
Okno startowe wybiera **profil urządzenia** i go zapamiętuje. Profil zawiera sterownik, typ lasera (dioda lub CO2), jednostki (mm lub cale; geometria jest zawsze przechowywana w mm), jednostki prędkości (na sekundę lub na minutę, przeliczane automatycznie; G-code zawsze niesie posuw w mm/min po `G21`), domyślną prędkość każdej warstwy, pole robocze, zero maszyny, maks. wartość S, prędkość jałową, prędkość transmisji, port, krok i posuw ręcznego przesuwania, moc obrysu i wyrównanie kamery. Okno konfiguracji dzieli je na **Device**, **Connection** (szeregowe lub TCP) i **Camera**. Edytor wymaga co najmniej jednego profilu.

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
