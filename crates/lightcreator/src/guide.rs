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
        title: "Window and bars",
        lines: &[
            "The window has its own flat frame: drag the title bar to move it, double-click it to maximise, use the buttons at its right end to minimise, maximise or close, and resize from the edges. File > Open recent lists the last 10 files.",
            "Control bar: the name of the current tool, New / Open / Save, switches for the grid, the secondary grid and snapping, the Toolpaths button (green while the paths are shown), Preview… and Camera view (light blue while their window is open; press again to close it) and Overlay. Start becomes STOP while a job runs; Start and Frame need a connected laser.",
            "The arrange bar between the work area and the side panel holds align, centre, turn, mirror, group, lock and order commands for the selection; hover an icon for its name. The tool bar holds union, intersection, subtract and exclusive or. A double arrow at the edge of a bar means more icons are out of sight: scroll to reach them.",
        ],
    },
    Section {
        title: "Drawing and editing",
        lines: &[
            "Tool bar (left): select (V), node edit (N), rectangle (R), ellipse (E), triangle (Y), star (S), polygon (G, asks for the number of sides, 3 to 360), heart (K), line (L), polyline (P) and text (T). Drag to draw; Shift keeps shapes square. Press Escape twice to return to the select tool.",
            "Selection: click, Shift+click to add. Drag a rectangle from left to right to select what is wholly inside, from right to left to select everything the rectangle touches. Ctrl+A selects all.",
            "Node edit: open it with the tool bar button (N), Arrange > Edit nodes or by double-clicking a shape; pictures from the gallery can be edited too. Text must be converted to curves first. Drag nodes and Bézier handles, double-click a segment to add a node, Delete removes nodes. The bar on the canvas switches nodes between corner and smooth, makes segments straight or curved, adds or deletes nodes, opens or closes a path and rounds the selected corners.",
            "Arrange menu: align, centre on bed, centre on each other, flip, rotate, group (Ctrl+G), lock (Ctrl+L), order, edit nodes, convert to path or curves and round corners (curve editing), union / intersection / subtract / exclusive or, offset shape, then adjust and trace image. Right-click opens the same commands as a context menu.",
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
            "A layer has a mode (Line, Fill, Fill + Line, Offset fill), speed, power, passes, air assist, interval, scan angle, overscan and, for layers that hold pictures, a dithering method and minimum power. The Layers tab lists layers in burn order; use the arrows to change it. The three switches at the end of each row are output (burn), visible and air pump.",
            "Speeds are shown per second or per minute, in millimetres or inches, as set in the device options (Speed units). New layers start at 1000 mm/min; the speed you set for a layer is remembered in the device profile and used for new documents.",
            "Material library (Laser menu) holds starting values for common materials; apply one to the active layer or save your own. Always test on scrap.",
        ],
    },
    Section {
        title: "Clipart gallery",
        lines: &[
            "The Clipart tab holds ready-made pictures in categories. A blue dot marks pictures that come with the program, a green dot those you saved or downloaded. Click a picture to place it on the active layer, in the middle of the work area, at the size set in the tab. A picture with several parts is grouped. The arrow on a category's title line folds it up or opens it again.",
            "Online… searches open icon collections on the internet (Iconify). Click a result to see its collection, licence and author, give it a name and a category (an existing one or a new one) and save it. The file is stored on your disk and shows up in the gallery with a green dot. Orange dots mark pictures whose licence asks you to credit the author. You can also download a picture from a web address, or add SVG files from your computer with Add file….",
            "Right-click a picture of yours to rename it, move it to another category, see its details (source, licence, author) or delete it. Right-click a shipped picture to save a copy in your own category.",
            "Only save pictures you are allowed to use and check the licence of the source before you sell what you make.",
        ],
    },
    Section {
        title: "Preview and output",
        lines: &[
            "Toolpaths shows the paths on the work area; Preview… opens the preview window, zoomed to the objects to burn, with every operation in its own colour, travel moves, a time estimate and a play slider. The icons in its title line fit the view to the work area or to all objects. The red ring is the machine zero.",
            "Start sends the job to the connected GRBL or Marlin laser; Frame traces the outline. For Ruida and Trocen, Start exports a PLT or DXF file for the controller's own software. File > Export saves SVG, G-code or PLT / DXF.",
            "Device tab: connect, jog with the arrow buttons (the orange one cancels a jog), home, unlock, pause, resume and the wide STOP button; jog step, jog feed and frame power sit above them. Console tab: everything sent to and received from the controller.",
            "Network: a device can connect over TCP, for example to ser2net on a Raspberry Pi. On Linux, Laser > Share over network (ser2net)… prepares ser2net for the laser plugged into this computer.",
        ],
    },
    Section {
        title: "Camera",
        lines: &[
            "Give a device a camera URL (MJPEG stream or JPEG snapshot) to get the Camera view button. It can float, open as a separate window or dock as a tab (the three icons in the window); rotate the picture in the device settings. The + and − buttons or the mouse wheel zoom the picture, drag moves it and Default zoom returns to the whole picture.",
            "Camera overlay shows a photo or the live picture under your design; drag its four corners to line it up with the material. The Overlay button (also in the camera window) shows or hides it; while the camera window is open it shows the live picture.",
        ],
    },
    Section {
        title: "Keyboard shortcuts",
        lines: &[
            "Ctrl+N new, Ctrl+O open, Ctrl+S save, Ctrl+Shift+S save as, Ctrl+I import SVG, Ctrl+Z undo, Ctrl+Y redo.",
            "Ctrl+C copy, Ctrl+V paste, Ctrl+D duplicate, Delete delete, Ctrl+A select all, Arrow keys nudge (Shift = 10 ×).",
            "Ctrl+G group, Ctrl+Shift+G ungroup, Ctrl+L lock, Ctrl+Shift+L unlock, F1 this guide.",
            "Navigation: the group under the tools holds pan (H), zoom in (Z), zoom out (X) and two fit buttons, fit work area (Ctrl+0) and fit all objects (Ctrl+9). Hold Space to pan with any tool and release it to go back. Mouse wheel zooms, middle button drags the view; the work area is fitted to the window until you zoom by hand (Fit work area turns it on again).",
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
        title: "Okno i paski",
        lines: &[
            "Okno ma własną, płaską ramkę: przeciągaj pasek tytułu, by je przesunąć, dwuklik maksymalizuje, przyciski na jego prawym końcu minimalizują, maksymalizują i zamykają, a rozmiar zmieniasz za krawędzie. File > Open recent pokazuje ostatnie 10 plików.",
            "Pasek sterowania: nazwa bieżącego narzędzia, New / Open / Save, przełączniki siatki, siatki pomocniczej i przyciągania, przycisk Toolpaths (zielony, gdy ścieżki są pokazane), Preview… i Camera view (błękitne, gdy ich okno jest otwarte; ponowne kliknięcie zamyka je) oraz Overlay. Start zmienia się w STOP podczas pracy; Start i Frame wymagają podłączonego lasera.",
            "Pasek wyrównania między obszarem roboczym a panelem bocznym zawiera polecenia wyrównania, środkowania, obrotu, odbicia, grupowania, blokowania i kolejności dla zaznaczenia; nazwę ikony pokaże podpowiedź. Pasek narzędzi zawiera sumę, część wspólną, odejmowanie i różnicę symetryczną. Podwójna strzałka na brzegu paska oznacza, że dalsze ikony są poza widokiem: przewiń, by je zobaczyć.",
        ],
    },
    Section {
        title: "Rysowanie i edycja",
        lines: &[
            "Pasek narzędzi (po lewej): zaznaczanie (V), edycja węzłów (N), prostokąt (R), elipsa (E), trójkąt (Y), gwiazda (S), wielokąt (G, pyta o liczbę boków od 3 do 360), serce (K), linia (L), łamana (P) i tekst (T). Rysuj przeciągając; Shift utrzymuje proporcje. Dwa razy Escape wracają do zaznaczania.",
            "Zaznaczanie: kliknięcie, Shift+kliknięcie dodaje. Ramka od lewej do prawej zaznacza tylko to, co jest w niej w całości; od prawej do lewej także to, czego dotyka. Ctrl+A zaznacza wszystko.",
            "Edycja węzłów: otwierasz ją przyciskiem na pasku narzędzi (N), przez Arrange > Edit nodes albo dwukrotnym kliknięciem kształtu; cliparty z galerii też można edytować. Tekst trzeba najpierw zamienić na krzywe. Przeciągaj węzły i uchwyty Béziera, dwukrotne kliknięcie odcinka dodaje węzeł, Delete usuwa węzły. Pasek na płótnie przełącza węzły między narożnikiem a gładkim, robi odcinki prostymi lub krzywymi, dodaje i usuwa węzły, otwiera lub zamyka ścieżkę i zaokrągla zaznaczone rogi.",
            "Menu Arrange: wyrównanie, środek stołu, wyśrodkowanie względem siebie, odbicia, obroty, grupowanie (Ctrl+G), blokowanie (Ctrl+L), kolejność, edycja węzłów, zamiana na ścieżkę lub krzywe i zaokrąglanie rogów (edycja krzywych), suma / część wspólna / odejmowanie / różnica symetryczna, offset kształtu, a dalej dostosowanie i trasowanie obrazu. Prawy klawisz myszy otwiera te same polecenia jako menu kontekstowe.",
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
            "Warstwa ma tryb (Line, Fill, Fill + Line, Offset fill), prędkość, moc, przejścia, pompę powietrza, odstęp, kąt, overscan, a dla warstw z obrazami rastrowanie i moc minimalną. Zakładka Layers pokazuje warstwy w kolejności wypalania; strzałkami zmieniasz ją. Trzy przełączniki na końcu wiersza to wyjście (wypalaj), widoczność i pompa powietrza.",
            "Prędkości są pokazywane na sekundę lub na minutę, w milimetrach lub calach, zależnie od opcji urządzenia (Speed units). Nowe warstwy startują z 1000 mm/min; prędkość ustawiona dla warstwy jest zapamiętywana w profilu urządzenia i używana w nowych dokumentach.",
            "Biblioteka materiałów (menu Laser) zawiera wartości startowe dla typowych materiałów; zastosuj je do aktywnej warstwy lub zapisz własne. Zawsze testuj na ścinku.",
        ],
    },
    Section {
        title: "Galeria clipartów",
        lines: &[
            "Zakładka Clipart zawiera gotowe obrazki w kategoriach. Niebieska kropka oznacza obrazki dostarczone z programem, zielona obrazki zapisane lub pobrane przez Ciebie. Kliknij obrazek, aby umieścić go na aktywnej warstwie, na środku obszaru roboczego, w rozmiarze ustawionym w zakładce. Obrazek z wieloma częściami jest grupowany. Strzałka w linii tytułu kategorii zwija ją lub rozwija.",
            "Online… przeszukuje otwarte kolekcje ikon w internecie (Iconify). Kliknij wynik, aby zobaczyć kolekcję, licencję i autora, nadaj mu nazwę i kategorię (istniejącą lub nową) i zapisz. Plik trafia na Twój dysk i pojawia się w galerii z zieloną kropką. Pomarańczowa kropka oznacza obrazki, których licencja wymaga podania autora. Możesz też pobrać obrazek spod adresu internetowego albo dodać pliki SVG z komputera przyciskiem Add file….",
            "Kliknij prawym przyciskiem własny obrazek, aby zmienić jego nazwę, przenieść do innej kategorii, zobaczyć szczegóły (źródło, licencję, autora) lub go usunąć. Prawy przycisk na obrazku z zestawu zapisuje jego kopię w Twojej kategorii.",
            "Zapisuj tylko obrazki, z których wolno Ci korzystać, i sprawdź licencję źródła, zanim zaczniesz sprzedawać wykonane prace.",
        ],
    },
    Section {
        title: "Podgląd i wyjście",
        lines: &[
            "Toolpaths pokazuje ścieżki na obszarze roboczym; Preview… otwiera okno podglądu, przybliżone do obiektów do wypalenia, z każdą operacją w osobnym kolorze, ruchami jałowymi, szacowanym czasem i suwakiem odtwarzania. Ikony w linii tytułu dopasowują widok do obszaru roboczego lub do wszystkich obiektów. Czerwony pierścień to zero maszyny.",
            "Start wysyła zadanie do podłączonego lasera GRBL lub Marlin, a Frame obrysowuje zadanie. Dla Ruida i Trocen Start eksportuje plik PLT lub DXF dla oprogramowania sterownika. File > Export zapisuje SVG, G-code lub PLT / DXF.",
            "Zakładka Device: połączenie, ręczne przesuwanie strzałkami (pomarańczowy przycisk anuluje ruch), bazowanie, odblokowanie, pauza, wznowienie i szeroki przycisk STOP; krok, posuw i moc obrysu są nad nimi. Zakładka Console: wszystko, co wysłano do sterownika i od niego odebrano.",
            "Sieć: urządzenie może łączyć się przez TCP, np. z ser2net na Raspberry Pi. Na Linuksie Laser > Share over network (ser2net)… przygotowuje ser2net dla lasera podłączonego do tego komputera.",
        ],
    },
    Section {
        title: "Kamera",
        lines: &[
            "Podaj w urządzeniu adres URL kamery (strumień MJPEG lub zdjęcie JPEG), aby pojawił się przycisk Camera view. Okno może pływać, być osobne albo zadokowane jako zakładka (trzy ikony w oknie); obrót obrazu ustawisz w ustawieniach urządzenia. Przyciski + i − lub kółko myszy powiększają obraz, przeciąganie go przesuwa, a Default zoom wraca do całego obrazu.",
            "Camera overlay pokazuje zdjęcie lub obraz na żywo pod projektem; przeciągnij cztery narożniki, aby dopasować go do materiału. Przycisk Overlay (także w oknie kamery) pokazuje lub ukrywa go; przy otwartym oknie kamery pokazuje obraz na żywo.",
        ],
    },
    Section {
        title: "Skróty klawiszowe",
        lines: &[
            "Ctrl+N nowy, Ctrl+O otwórz, Ctrl+S zapisz, Ctrl+Shift+S zapisz jako, Ctrl+I import SVG, Ctrl+Z cofnij, Ctrl+Y ponów.",
            "Ctrl+C kopiuj, Ctrl+V wklej, Ctrl+D powiel, Delete usuń, Ctrl+A zaznacz wszystko, strzałki przesuwają (Shift = 10 ×).",
            "Ctrl+G grupuj, Ctrl+Shift+G rozgrupuj, Ctrl+L zablokuj, Ctrl+Shift+L odblokuj, F1 ten przewodnik.",
            "Nawigacja: osobna grupa pod narzędziami zawiera przesuwanie widoku (H), powiększanie (Z), pomniejszanie (X) oraz dwa przyciski dopasowania: do obszaru roboczego (Ctrl+0) i do wszystkich obiektów (Ctrl+9). Przytrzymaj spację, aby przesuwać widok dowolnym narzędziem; po puszczeniu wraca poprzednie. Kółko myszy zbliża, środkowy przycisk przesuwa widok; obszar roboczy jest dopasowany do okna, dopóki nie zmienisz zbliżenia ręcznie (przycisk dopasowania do obszaru włącza to z powrotem).",
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
