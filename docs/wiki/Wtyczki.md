# Wtyczki

Wtyczki dodają LightCreatorowi rzeczy bez zmieniania programu: **clipart, presety maszyn, tłumaczenia, importery innych formatów plików, generatory kształtów i postprocesory G-code.** Są wspólne dla LightCreatora, MillCreatora i PenCreatora: wtyczka leży w folderze `plugins` we wspólnym folderze danych (`baseCreator`), a każdy program używa tych części, które do niego pasują.

**Settings → Plugins…** otwiera okno: instalacja wtyczki z pliku `.zip`, włączanie i wyłączanie, usuwanie, otwarcie folderu oraz wybór **postprocesora** dla następnego eksportu G-code.

* **Importery:** **File → Import → Import with a plugin…**, przeciągnięcie pliku albo Otwórz na pliku, którego rozszerzenie czyta jakaś wtyczka. Kształty są dodawane do rysunku (zaznaczone, jedno cofnięcie).
* **Generatory:** **File → Import → Generate shape…** pokazuje formularz generatora (liczby, przełączniki, teksty, listy) i dodaje kształty, które tworzy.
* **Postprocesory:** gdy jeden jest wybrany w oknie Wtyczki, **Export G-code…** przepuszcza program przez niego. Może zmieniać tylko linie G-code; linie będące komendami kontrolera (`$…`, `~`, `!`, `?`) albo ze znakami sterującymi są odrzucane i nic się nie zapisuje. Wysyłanie zadania prosto do maszyny nie przechodzi przez postprocesor.
* Presety i pliki danych (`mill-tools`) wtyczek nie są używane przez LightCreatora; bierze z nich clipart, importery, generatory i postprocesory.

**Bezpieczeństwo.** Skrypty wtyczek (Rhai) działają w piaskownicy bez dostępu do plików, sieci, systemu i maszyny, z limitem czasu i pamięci. Mimo to wtyczka to czyjś kod, który decyduje, co usłyszy Twoja maszyna: instaluj tylko wtyczki, którym ufasz, i obejrzyj G-code nowego postprocesora, zanim go uruchomisz na maszynie.

Jak napisać wtyczkę (manifest, formularz, funkcje skryptu, przykłady): [PLUGINS.md w baseCreator](https://github.com/michalsarna/baseCreator/blob/main/docs/PLUGINS.md) (po angielsku).
