# Wtyczki

Wtyczki dodają LightCreatorowi rzeczy bez zmieniania programu: **clipart, presety maszyn, tłumaczenia, importery innych formatów plików, generatory kształtów i postprocesory G-code.** Są wspólne dla LightCreatora, MillCreatora i PenCreatora: wtyczka leży w folderze `plugins` we wspólnym folderze danych (`baseCreator`), a każdy program używa tych części, które do niego pasują.

**Settings → Plugins…** otwiera okno: instalacja wtyczki z pliku `.zip`, włączanie i wyłączanie, usuwanie, otwarcie folderu oraz ustawianie wtyczek z opcjami: każda wtyczka, w której jest coś do wyboru (**postprocesor**), ma własną kartę w prawym panelu, gdzie wybiera się ją dla następnego eksportu G-code.

* **Importery:** **File → Import → Import with a plugin…**, przeciągnięcie pliku albo Otwórz na pliku, którego rozszerzenie czyta jakaś wtyczka. Kształty są dodawane do rysunku (zaznaczone, jedno cofnięcie).
* **Generatory:** **File → Import → Generate shape…** pokazuje formularz generatora (liczby, przełączniki, teksty, listy) i dodaje kształty, które tworzy.
* **Postprocesory:** gdy jeden jest wybrany w karcie jego wtyczki, **Export G-code…** przepuszcza program przez niego. Może zmieniać tylko linie G-code; linie będące komendami kontrolera (`$…`, `~`, `!`, `?`) albo ze znakami sterującymi są odrzucane i nic się nie zapisuje. Wysyłanie zadania prosto do maszyny nie przechodzi przez postprocesor.
* Presety i pliki danych (`mill-tools`) wtyczek nie są używane przez LightCreatora; bierze z nich clipart, importery, generatory i postprocesory.

**Jak używać G-code tools (i każdego postprocesora), krok po kroku.** 1. *Settings → Plugins…* → **Oficjalne wtyczki** (Official plugins) → zainstaluj **G-code tools** (albo **Zainstaluj wtyczkę z pliku zip…** (Install plugin from zip…)). 2. W prawym panelu, po pozostałych kartach, pojawia się karta **G-code tools** (gdy panel jest zwinięty: na wąskim pasku). 3. Otwórz ją i wybierz **Number the lines (N10, N20, …)** albo **Remove comments**; **Brak** (None) wyłącza postprocesor. 4. **Export G-code…** (menu Laser / Milling / Plot albo przycisk **Zapisz G-code** (Save G-code)) zapisuje teraz przetworzony plik; pasek stanu mówi, którego postprocesora użyto. Zadanie uruchomione przyciskiem **Start** jest wysyłane bez zmian, a podgląd G-code pokazuje plik tak, jak go zapisano.

**Bezpieczeństwo.** Skrypty wtyczek (Rhai) działają w piaskownicy bez dostępu do plików, sieci, systemu i maszyny, z limitem czasu i pamięci. Mimo to wtyczka to czyjś kod, który decyduje, co usłyszy Twoja maszyna: instaluj tylko wtyczki, którym ufasz, i obejrzyj G-code nowego postprocesora, zanim go uruchomisz na maszynie.

## Oficjalne wtyczki

**Settings → Plugins… → Official plugins** pokazuje oficjalną listę wtyczek, prowadzoną w repozytorium [baseCreator-plugins](https://github.com/michalsarna/baseCreator-plugins). Naciśnij **Check for updates**, żeby ją wczytać: widać na niej *wszystkie* wtyczki, także te dla innego programu (oznaczone *For: …*). **Install** pobiera wtyczkę z oficjalnej strony (sprawdzana jest suma kontrolna) do folderu wtyczek wspólnego dla wszystkich programów; **Update** pojawia się, gdy lista ma nowszą wersję (opis zmian po najechaniu). Wtyczka wymagająca nowszego programu niż Twój jest pokazana, ale nie do instalacji. Sprawdzanie aktualizacji przy starcie programu jest **włączone** (odznacz **Sprawdzaj aktualizacje wtyczek przy starcie** (Check for plugin updates at start), żeby je wyłączyć): gdy jakieś wtyczki są nieaktualne, okno je wymienia i daje **Update all**, **Plugins** i **Later**. Nic nie instaluje się bez Twojego kliknięcia.

Własną wtyczkę napiszesz i wyślesz na listę według [przewodnika dla autorów](https://github.com/michalsarna/baseCreator-plugins/blob/main/docs/GUIDE.md) (po angielsku) z szablonami w repozytorium wtyczek; błędy i pomysły zgłaszaj w issues tamtego repozytorium.

Jak napisać wtyczkę (manifest, formularz, funkcje skryptu, przykłady): [PLUGINS.md w baseCreator](https://github.com/michalsarna/baseCreator/blob/main/docs/PLUGINS.md) (po angielsku).

Kilka postprocesorów może być włączonych naraz: zaznacz je na kartach wtyczek; działają **w kolejności zaznaczania** (linie jednego trafiają do następnego), a pasek stanu wymienia je połączone znakiem `+`. **None** wyłącza wszystkie.
