🌐 [English](Devices-and-Controllers) · **Polski**

# Urządzenia i sterowniki

![Konfiguracja urządzenia](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/05-device-config.png)

## Profile urządzeń
Okno startowe wybiera **profil urządzenia** i go zapamiętuje. Profil zawiera sterownik, typ lasera (dioda lub CO2), jednostki (mm lub cale; geometria jest zawsze przechowywana w mm), pole robocze, zero maszyny, maks. wartość S, prędkość jałową, prędkość transmisji, port, krok i posuw ręcznego przesuwania, moc obrysu i wyrównanie kamery. Okno konfiguracji dzieli je na **Device**, **Connection** (szeregowe lub TCP) i **Camera**. Edytor wymaga co najmniej jednego profilu.

## Sterowniki
| Sterownik | Obsługa |
|---|---|
| **GRBL** | strumieniowanie przez port szeregowy lub TCP: połączenie, ręczne przesuwanie, bazowanie, odblokowanie, pauza / wznowienie, stop, obrys, postęp; *Read from device* wczytuje pole robocze, S max, prędkość jałową i tryb lasera |
| **Marlin** | to samo (funkcja lasera, moc inline); odczyt posuwu |
| **Ruida**, **Trocen** | eksport HPGL / DXF z jednym piórem lub warstwą na kolor, do otwarcia w programie producenta (np. RDWorks). Wypełnienie staje się liniami kreskowania; obrazy rastrowe są pomijane. Bezpośrednie protokoły binarne nie są zaimplementowane |

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
Podaj urządzeniu adres kamery (strumień MJPEG lub zdjęcie JPEG, http / https; RTSP nie jest obsługiwany), a pojawi się przycisk **Camera view**. Widok może pływać w oknie programu, otwierać się jako osobne okno systemu lub być przypięty jako zakładka panelu bocznego; można go obracać o 90° / 180° / 270° (ustawienie urządzenia). Przycisk **Overlay** i *View → Show camera overlay* pokazują obraz pod projektem.
![Zakładka kamery](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/17-camera-tab.png)

**Wyrównanie nakładki:** zdjęcie lub obraz na żywo leży pod projektem, wyrównany czterema przeciąganymi rogami (korekcja perspektywy), z przezroczystością, odbiciem i obrotem; zapisywane dla urządzenia.
![Nakładka](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/10-camera-overlay.png)
