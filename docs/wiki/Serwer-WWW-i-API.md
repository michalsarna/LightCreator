🌐 [English](Web-Server-and-API) · **Polski**

# Serwer WWW i API

![Opcje](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/36-options-web-server.png)

*Opcje programu → Serwer WWW i API* włącza mały serwer WWW wbudowany w LightCreator. Jest **domyślnie wyłączony**. Pokazuje w przeglądarce (także na telefonie) stan maszyny, projekt i kamerę oraz udostępnia API dla innych programów.

**Każde żądanie wymaga hasła dostępu** (ustawianego w tych samych opcjach przyciskiem *Zapisz* obok pola; na początku to `LightCreator`, zmień je, zanim wpuścisz inne komputery). Przeglądarka o nie zapyta (dowolna nazwa użytkownika, hasło w polu hasła); programy wysyłają je jako uwierzytelnianie HTTP Basic, np. `curl -u dowolny:LightCreator http://localhost:8765/api/status`. Złe hasło kończy się odpowiedzią 401 po krótkim oczekiwaniu. **Nie ma szyfrowania**, więc hasło i dane może przeczytać każdy, kto podsłuchuje sieć, a hasło leży jawnym tekstem w `app.ron`. Domyślnie serwer nasłuchuje tylko na tym komputerze (`http://localhost:8765`). *Dostępny z innych komputerów w sieci* włącza nasłuch na wszystkich interfejsach: wtedy każdy w sieci może zobaczyć maszynę oraz ją zatrzymać lub poruszyć. Używaj tego tylko w sieci, której ufasz, i nigdy nie otwieraj portu do internetu.

## Strona WWW
`http://localhost:<port>/` pokazuje urządzenie i jego stan, położenie, czas pracy zadania na tle szacowanego, projekt jako obraz, kamerę (lista pozwala wybrać aktywną kamerę; zmienia ona aktywną kamerę urządzenia) oraz przyciski STOP, Home, Cancel jog i osiem strzałek przesuwania (jeden krok przesuwania urządzenia).

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
