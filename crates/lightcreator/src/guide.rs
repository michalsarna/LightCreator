//! The built-in quick guide: short descriptions of the program's options.
//! Written in English and Polish; other interface languages show the English text.
use crate::i18n::{lang, Lang};

pub struct Section {
    pub title: &'static str,
    pub lines: &'static [&'static str],
}

const EN: &[Section] = &[
    Section {
        title: "Getting started",
        lines: &[
            "The start window asks which device you work with. Create a device profile first (Add device…): name, controller (GRBL, Marlin, Ruida, Trocen), laser type, work area, units and how to connect. Pick the profile and press Continue.",
            "Laser > Switch device… returns to the start window; Laser > Device settings… edits the active profile. Everything is stored in your profile list and remembered.",
            "Settings > Language and Colour scheme change the interface; Settings > View options sets the grids, the work-area background colour and the line thickness.",
        ],
    },
    Section {
        title: "Drawing and editing",
        lines: &[
            "Tool bar (left): select (V), node edit (N), rectangle (R), ellipse (E), triangle (Y), star (S), polygon (G, asks for the number of sides, 3 to 360), line (L), polyline (P), text (T), pan (H), zoom (Z). Drag to draw; Shift keeps shapes square. Press Escape twice to return to the select tool.",
            "Selection: click, Shift+click to add. Drag a rectangle from left to right to select what is wholly inside, from right to left to select everything the rectangle touches. Ctrl+A selects all.",
            "Node edit: drag nodes and Bézier handles, double-click a segment to add a node, Delete removes nodes. The bar on the canvas switches nodes between corner and smooth, makes segments straight or curved, adds or deletes nodes, opens or closes a path and rounds the selected corners.",
            "Arrange menu: align, centre on bed, flip, rotate, order, group (Ctrl+G), lock (Ctrl+L), convert to path or curves, union / intersection / subtract / exclusive or, offset shape and round corners. Right-click opens the same commands as a context menu.",
            "Round corners: replaces sharp straight corners with arcs of the radius you give. With two straight lines selected it joins them with a rounded corner instead. Too large a radius is limited to what fits.",
            "Locked objects cannot be moved, edited or deleted, but they can be selected and copied; a copy is not locked.",
        ],
    },
    Section {
        title: "Text, images and import",
        lines: &[
            "Text tool: click on the work area, then type in the Properties tab. Choose font, size, style and spacing; Convert to curves turns text into editable shapes.",
            "File > Import: SVG, pictures (PNG, JPEG, BMP, GIF, WebP) and Adobe Illustrator / PDF files. A picture opens an import window with rotate, flip, brightness, contrast, gamma, auto levels, negative and size. Arrange > Adjust image… changes it later; Trace image… turns it into vector outlines.",
        ],
    },
    Section {
        title: "Layers and cut settings",
        lines: &[
            "Every object belongs to one of 30 colour layers. Click a colour in the strip at the bottom to make it active (selected objects move to it); double-click a layer name for all its options.",
            "A layer has a mode (Line, Fill, Fill + Line, Offset fill), speed, power, passes, interval, scan angle, overscan and, for pictures, a dithering method and minimum power. The Layers tab lists layers in burn order; use the arrows to change it.",
            "Material library (Laser menu) holds starting values for common materials; apply one to the active layer or save your own. Always test on scrap.",
        ],
    },
    Section {
        title: "Clipart gallery",
        lines: &[
            "The Clipart tab holds ready-made pictures in categories. A blue dot marks pictures that come with the program, a green dot those you saved or downloaded. Click a picture to place it on the active layer, in the middle of the work area, at the size set in the tab. A picture with several parts is grouped.",
            "Online… searches open icon collections on the internet (Iconify). Click a result to see its collection, licence and author, give it a name and a category (an existing one or a new one) and save it. The file is stored on your disk and shows up in the gallery with a green dot. Orange dots mark pictures whose licence asks you to credit the author. You can also download a picture from a web address, or add SVG files from your computer with Add file….",
            "Right-click a picture of yours to rename it, move it to another category, see its details (source, licence, author) or delete it. Right-click a shipped picture to save a copy in your own category.",
            "Only save pictures you are allowed to use and check the licence of the source before you sell what you make.",
        ],
    },
    Section {
        title: "Preview and output",
        lines: &[
            "Toolpaths shows the paths on the work area; Preview… opens the preview window with every operation in its own colour, travel moves, a time estimate and a play slider. The red ring is the machine zero.",
            "Start sends the job to the connected GRBL or Marlin laser; Frame traces the outline. For Ruida and Trocen, Start exports a PLT or DXF file for the controller's own software. File > Export saves SVG, G-code or PLT / DXF.",
            "Device tab: connect, jog, home, unlock, pause, stop. Console tab: everything sent to and received from the controller.",
            "Network: a device can connect over TCP, for example to ser2net on a Raspberry Pi. On Linux, Laser > Share over network (ser2net)… prepares ser2net for the laser plugged into this computer.",
        ],
    },
    Section {
        title: "Camera",
        lines: &[
            "Give a device a camera URL (MJPEG stream or JPEG snapshot) to get the Camera view button. It can float, open as a separate window or dock as a tab; rotate the picture in the device settings.",
            "Camera overlay shows a photo or the live picture under your design; drag its four corners to line it up with the material. The Overlay button hides it again.",
        ],
    },
    Section {
        title: "Keyboard shortcuts",
        lines: &[
            "Ctrl+N new, Ctrl+O open, Ctrl+S save, Ctrl+Shift+S save as, Ctrl+I import SVG, Ctrl+Z undo, Ctrl+Y redo.",
            "Ctrl+C copy, Ctrl+V paste, Ctrl+D duplicate, Delete delete, Ctrl+A select all, Arrow keys nudge (Shift = 10 ×).",
            "Ctrl+G group, Ctrl+Shift+G ungroup, Ctrl+L lock, Ctrl+Shift+L unlock, F1 this guide.",
            "Mouse wheel zooms, middle button drags the view; the work area is fitted to the window until you zoom by hand (View > Fit bed to window turns it on again).",
        ],
    },
];

const PL: &[Section] = &[
    Section {
        title: "Pierwsze kroki",
        lines: &[
            "Okno startowe pyta, z którym urządzeniem pracujesz. Najpierw utwórz profil urządzenia (Add device…): nazwa, sterownik (GRBL, Marlin, Ruida, Trocen), typ lasera, obszar roboczy, jednostki i sposób połączenia. Wybierz profil i naciśnij Continue.",
            "Laser > Switch device… wraca do okna startowego, a Laser > Device settings… edytuje aktywny profil. Profile są zapamiętywane.",
            "Settings > Language i Colour scheme zmieniają interfejs; Settings > View options ustawia siatki, kolor tła obszaru roboczego i grubość linii.",
        ],
    },
    Section {
        title: "Rysowanie i edycja",
        lines: &[
            "Pasek narzędzi (po lewej): zaznaczanie (V), edycja węzłów (N), prostokąt (R), elipsa (E), trójkąt (Y), gwiazda (S), wielokąt (G, pyta o liczbę boków od 3 do 360), linia (L), łamana (P), tekst (T), przesuwanie widoku (H), zoom (Z). Rysuj przeciągając; Shift utrzymuje proporcje. Dwa razy Escape wracają do zaznaczania.",
            "Zaznaczanie: kliknięcie, Shift+kliknięcie dodaje. Ramka od lewej do prawej zaznacza tylko to, co jest w niej w całości; od prawej do lewej także to, czego dotyka. Ctrl+A zaznacza wszystko.",
            "Edycja węzłów: przeciągaj węzły i uchwyty Béziera, dwukrotne kliknięcie odcinka dodaje węzeł, Delete usuwa węzły. Pasek na płótnie przełącza węzły między narożnikiem a gładkim, robi odcinki prostymi lub krzywymi, dodaje i usuwa węzły, otwiera lub zamyka ścieżkę i zaokrągla zaznaczone rogi.",
            "Menu Arrange: wyrównanie, środek stołu, odbicia, obroty, kolejność, grupowanie (Ctrl+G), blokowanie (Ctrl+L), zamiana na ścieżkę lub krzywe, suma / część wspólna / odejmowanie / różnica symetryczna, offset kształtu i zaokrąglanie rogów. Prawy klawisz myszy otwiera te same polecenia jako menu kontekstowe.",
            "Zaokrąglanie rogów: zamienia ostre proste narożniki na łuki o podanym promieniu. Przy zaznaczonych dwóch prostych łączy je zaokrąglonym narożnikiem. Zbyt duży promień jest ograniczany do tego, co się mieści.",
            "Zablokowanych obiektów nie można przesuwać, edytować ani usuwać, ale można je zaznaczać i kopiować; kopia nie jest zablokowana.",
        ],
    },
    Section {
        title: "Tekst, obrazy i import",
        lines: &[
            "Narzędzie tekstu: kliknij na obszarze roboczym i wpisz tekst w zakładce Properties. Wybierz czcionkę, rozmiar, styl i odstępy; Convert to curves zamienia tekst w edytowalne kształty.",
            "File > Import: SVG, obrazy (PNG, JPEG, BMP, GIF, WebP) i pliki Adobe Illustrator / PDF. Obraz otwiera okno importu z obrotem, odbiciem, jasnością, kontrastem, gamma, automatycznymi poziomami, negatywem i rozmiarem. Arrange > Adjust image… zmienia go później, a Trace image… zamienia w kontury wektorowe.",
        ],
    },
    Section {
        title: "Warstwy i ustawienia cięcia",
        lines: &[
            "Każdy obiekt należy do jednej z 30 kolorowych warstw. Kliknij kolor na dolnym pasku, aby uczynić warstwę aktywną (zaznaczone obiekty przechodzą na nią); dwukrotne kliknięcie nazwy otwiera wszystkie opcje warstwy.",
            "Warstwa ma tryb (Line, Fill, Fill + Line, Offset fill), prędkość, moc, przejścia, odstęp, kąt, overscan, a dla obrazów rastrowanie i moc minimalną. Zakładka Layers pokazuje warstwy w kolejności wypalania; strzałkami zmieniasz ją.",
            "Biblioteka materiałów (menu Laser) zawiera wartości startowe dla typowych materiałów; zastosuj je do aktywnej warstwy lub zapisz własne. Zawsze testuj na ścinku.",
        ],
    },
    Section {
        title: "Galeria clipartów",
        lines: &[
            "Zakładka Clipart zawiera gotowe obrazki w kategoriach. Niebieska kropka oznacza obrazki dostarczone z programem, zielona obrazki zapisane lub pobrane przez Ciebie. Kliknij obrazek, aby umieścić go na aktywnej warstwie, na środku obszaru roboczego, w rozmiarze ustawionym w zakładce. Obrazek z wieloma częściami jest grupowany.",
            "Online… przeszukuje otwarte kolekcje ikon w internecie (Iconify). Kliknij wynik, aby zobaczyć kolekcję, licencję i autora, nadaj mu nazwę i kategorię (istniejącą lub nową) i zapisz. Plik trafia na Twój dysk i pojawia się w galerii z zieloną kropką. Pomarańczowa kropka oznacza obrazki, których licencja wymaga podania autora. Możesz też pobrać obrazek spod adresu internetowego albo dodać pliki SVG z komputera przyciskiem Add file….",
            "Kliknij prawym przyciskiem własny obrazek, aby zmienić jego nazwę, przenieść do innej kategorii, zobaczyć szczegóły (źródło, licencję, autora) lub go usunąć. Prawy przycisk na obrazku z zestawu zapisuje jego kopię w Twojej kategorii.",
            "Zapisuj tylko obrazki, z których wolno Ci korzystać, i sprawdź licencję źródła, zanim zaczniesz sprzedawać wykonane prace.",
        ],
    },
    Section {
        title: "Podgląd i wyjście",
        lines: &[
            "Toolpaths pokazuje ścieżki na obszarze roboczym; Preview… otwiera okno podglądu z każdą operacją w osobnym kolorze, ruchami jałowymi, szacowanym czasem i suwakiem odtwarzania. Czerwony pierścień to zero maszyny.",
            "Start wysyła zadanie do podłączonego lasera GRBL lub Marlin, a Frame obrysowuje zadanie. Dla Ruida i Trocen Start eksportuje plik PLT lub DXF dla oprogramowania sterownika. File > Export zapisuje SVG, G-code lub PLT / DXF.",
            "Zakładka Device: połączenie, ręczne przesuwanie, bazowanie, odblokowanie, pauza, stop. Zakładka Console: wszystko, co wysłano do sterownika i od niego odebrano.",
            "Sieć: urządzenie może łączyć się przez TCP, np. z ser2net na Raspberry Pi. Na Linuksie Laser > Share over network (ser2net)… przygotowuje ser2net dla lasera podłączonego do tego komputera.",
        ],
    },
    Section {
        title: "Kamera",
        lines: &[
            "Podaj w urządzeniu adres URL kamery (strumień MJPEG lub zdjęcie JPEG), aby pojawił się przycisk Camera view. Okno może pływać, być osobne albo zadokowane jako zakładka; obrót obrazu ustawisz w ustawieniach urządzenia.",
            "Camera overlay pokazuje zdjęcie lub obraz na żywo pod projektem; przeciągnij cztery narożniki, aby dopasować go do materiału. Przycisk Overlay ukrywa go.",
        ],
    },
    Section {
        title: "Skróty klawiszowe",
        lines: &[
            "Ctrl+N nowy, Ctrl+O otwórz, Ctrl+S zapisz, Ctrl+Shift+S zapisz jako, Ctrl+I import SVG, Ctrl+Z cofnij, Ctrl+Y ponów.",
            "Ctrl+C kopiuj, Ctrl+V wklej, Ctrl+D powiel, Delete usuń, Ctrl+A zaznacz wszystko, strzałki przesuwają (Shift = 10 ×).",
            "Ctrl+G grupuj, Ctrl+Shift+G rozgrupuj, Ctrl+L zablokuj, Ctrl+Shift+L odblokuj, F1 ten przewodnik.",
            "Kółko myszy zbliża, środkowy przycisk przesuwa widok; obszar roboczy jest dopasowany do okna, dopóki nie zmienisz zbliżenia ręcznie (View > Fit bed to window włącza to z powrotem).",
        ],
    },
];

pub fn sections() -> &'static [Section] {
    if lang() == Lang::Pl {
        PL
    } else {
        EN
    }
}
