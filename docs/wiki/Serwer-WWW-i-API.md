🌐 [English](Web-Server-and-API) · **Polski**

# Serwer WWW i API

![Opcje](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/36-options-web-server.png)

*Opcje programu → Serwer WWW i API* włącza mały serwer WWW wbudowany w LightCreator. Jest **domyślnie wyłączony**. Pokazuje w przeglądarce (także na telefonie) stan maszyny, projekt i kamerę oraz udostępnia API dla innych programów.

**Każde żądanie wymaga hasła dostępu** (ustawianego w tych samych opcjach przyciskiem *Zapisz* obok pola; na początku to `LightCreator`, zmień je, zanim wpuścisz inne komputery). Przeglądarka o nie zapyta (dowolna nazwa użytkownika, hasło w polu hasła); programy wysyłają je jako uwierzytelnianie HTTP Basic, np. `curl -u dowolny:LightCreator http://localhost:8765/api/status`. Złe hasło kończy się odpowiedzią 401 po krótkim oczekiwaniu. **Nie ma szyfrowania**, więc hasło i dane może przeczytać każdy, kto podsłuchuje sieć, a hasło leży w `app.ron` zapisane jako base64 (nie widać go na pierwszy rzut oka, ale base64 to kodowanie, a nie szyfrowanie). Domyślnie serwer nasłuchuje tylko na tym komputerze (`http://localhost:8765`). *Dostępny z innych komputerów w sieci* włącza nasłuch na wszystkich interfejsach: wtedy każdy w sieci może zobaczyć maszynę oraz ją zatrzymać lub poruszyć. Używaj tego tylko w sieci, której ufasz, i nigdy nie otwieraj portu do internetu.

## Strona WWW
`http://localhost:<port>/` ma pasek z urządzeniem, jego stanem, położeniem, czasem pracy zadania na tle szacowanego i paskiem postępu oraz listę **Theme**: strona zaczyna w schemacie kolorów programu (do wyboru są wszystkie 14 schematów; wybór zostaje w przeglądarce).

Na komputerze strona ma dwie kolumny: po lewej, na dwóch trzecich szerokości, **kamera** (lista wybiera aktywną kamerę, co zmienia aktywną kamerę urządzenia; bez kamery jest duża przekreślona kamera), po prawej, na jednej trzeciej, **panel sterowania** (STOP, Home, Cancel jog i osiem strzałek przesuwania, jeden krok przesuwania urządzenia), a pod nim **projekt** jako obraz. Dwukrotne kliknięcie obrazu (albo przycisk pełnego ekranu) powiększa go na całe okno, z przyciskami przybliżania, oddalania, przesuwania i resetowania widoku; działają też kółko myszy, przeciąganie i szczypanie dwoma palcami, a Esc zamyka widok.

Obraz z kamery jest obrócony tak jak w programie: domyślnie obowiązuje **Obrót kamery** (Camera rotation) z ustawień urządzenia (0°, 90°, 180°, 270° w prawo), a lista nad obrazem pozwala wybrać inny obrót, który przeglądarka zapamiętuje dla każdej kamery. `GET /api/cameras` podaje dla kamery `rotation` oraz `rotated` (ile z tego obrotu ma już zdjęcie).

Na telefonie strona ma jedną kolumnę: status, kamera, projekt, a na dole panel sterowania.

## API
Wszystkie odpowiedzi to JSON, o ile nie napisano inaczej. Polecenia wymagają `POST`.

| Żądanie | Działanie |
|---|---|
| `GET /api/status` | program, urządzenie (nazwa, sterownik, połączenie, stan, X, Y), zadanie (praca, wykonane / wszystkie linie, sekundy upłynęły i szacowane) i projekt (nazwa pliku, rewizja, opis) |
| `GET /api/project.svg` | projekt jako SVG |
| `GET /api/cameras` | kamery: numer, nazwa, włączona, aktywna, `url` (własny adres http / https kamery albo `null`) i `snapshot` |
| `GET /api/camera.jpg` | najnowszy obraz widoku kamery jako JPEG (tylko gdy Widok kamery jest otwarty w programie; inaczej 503) |
| `POST /api/stop` | zatrzymuje trwające zadanie (jak STOP) |
| `POST /api/home` | bazowanie, tylko przy połączeniu i bez trwającego zadania |
| `POST /api/cancel-jog` | anuluje ruch ręczny |
| `POST /api/jog?dir=…` | jeden krok: `up`, `down`, `left`, `right`, `up-left`, `up-right`, `down-left`, `down-right`; tylko przy połączeniu i bez trwającego zadania, w dozwolonym obszarze |
| `POST /api/camera/select?index=N` | ustawia kamerę N jako aktywną |

Polecenia podlegają regułom przycisków, które zastępują. Nic z tego nie było jeszcze próbowane na prawdziwym sprzęcie; sam serwer jest pokryty testami.
