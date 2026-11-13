//! The quick guide of LightCreator: what is its own. The rest is the shared guide (`base_ui::guide`).
use crate::i18n::{lang, Lang};
use base_ui::dialogs::Section;
use base_ui::guide::{Own, Picture};

const EN: &[Section] = &[
    Section {
        title: "Getting started",
        lines: &[
            "The start window asks which device you work with. Create a device profile first (Add device…): name, controller (GRBL, Marlin, Ruida, Trocen), laser type, work area, units and how to connect. Pick the profile and press New or Open…. Without a machine press I have no device (or just press New): the stand-in Dummy is used.",
            "Everything is stored in your profile list and remembered.",
            "After New, the window of File > Project setup… asks about the piece of material: a block (its size and thickness) or a cylinder on the rotary attachment. Sides to work on: 2 sides gives side A and side B, turned over round the horizontal (H) or vertical (V) axis, with corner marks to put the material back: a switch over the work area shows one side or the other, with the ghosts of the other side turned over. Click a ghost to align the selected shapes to it or to snap to it; a ghost never moves. Save G-code asks whether both sides go into one file, with or without a stop (M0) to turn the material over, or into two files ending in _A and _B; each side says at its head which one it is. Preview G-code asks the same and shows the whole job in one tab or each side in a tab of its own; Save G-code in that tab asks too.",
            "The alignment marks of two sides are a job of their own: they lie just outside the material, so Save G-code offers them in a file of their own (name_marks), burned first on the sheet under the material; the material is then put into them for side A and, turned over, for side B. They are drawn in red round the material, not as shapes.",
            "4 sides (post) and 6 sides (block) in File > Project setup…: a post with a base a × b and a height H lies along X and is burned on its four long faces A to D, rolled a quarter turn back between them (the front comes up); a block a × b × c is burned on all six faces, E and F being its left and right end turned up. Each face is a work area of its own, as big as the face; the work area is always the material (also for 2 sides). Round the shown face the faces beside it are unfolded with their ghosts, the face under it shows through. Save G-code puts all the faces in one file (with a stop before each face that says how to lay the object, and a note to focus on its top) or a file for each face (_A … _F).",
            "Preview > 3D shows the burned object as big as the material of the setup, with the jobs of all its faces on it (the shown face plays up to the slider): drag to turn it, the wheel zooms, the cube at the corner looks from a side. Burn colours draws the lines darker for more power, as on wood. Flat goes back to the work area.",
        ],
    },
    Section {
        title: "Layers and cut settings",
        lines: &[
            "Every object belongs to one of 64 colour layers. Click a colour in the strip at the bottom to make it active (selected objects move to it); double-click a layer name for all its options.",
            "A layer has a mode (Line, Fill, Fill + Line, Offset fill), speed, power, passes, air assist, interval, scan angle, overscan and, for layers that hold pictures, a dithering method and minimum power. The Layers tab lists layers in burn order; use the arrows to change it. The three switches at the end of each row are output (burn), visible and air pump.",
            "Save preset… and Load preset above the settings of a layer keep its way of burning under a name and put it on the active layer later. Check a preset on a piece of scrap, as the same power gives a different result on another laser.",
            "On a Line layer, Lead in / Lead out gives every closed outline a short line or arc from the waste side and an overlap past its start, so the burn mark where the laser starts is not on the part.",
            "Speeds are shown per second or per minute, in millimetres or inches, as set in the device options (Speed units). New layers start at 1000 mm/min; the settings of the layers belong to the project.",
        ],
    },
    Section {
        title: "Preview and output",
        lines: &[
            "Control bar: the name of the current tool, New / Open / Save, switches for the grid, the secondary grid and snapping, the Toolpaths button (green while the paths are shown), Preview… and Camera view (light blue while their window is open; press again to close it) and Overlay. Start becomes STOP while a job runs; Start and Frame need a connected laser.",
            "Toolpaths shows the paths on the work area; Preview… opens the preview window, zoomed to the objects to burn, with every operation in its own colour, travel moves, a time estimate and a play slider. The zoom bar in the lower left corner of the picture zooms in and out and fits the view to the work area or to all objects. The red ring is the machine zero.",
            "Start sends the job to the connected GRBL or Marlin laser; Frame traces the outline. For Ruida and Trocen, Start exports a PLT or DXF file for the controller's own software. File > Export saves SVG, G-code or PLT / DXF.",
            "Device tab: connect, then (under a line) Home, Unlock, Pause and Resume in a block of 2 x 2 and the wide STOP button (live only while the machine works). Control tab: the same four buttons and STOP at the top, the position, the step and speed of a jog and the jog arrows (the orange one cancels a jog), the focus beam, the macros, start from, frame power and the live position; the work zero is not moved from there. Console tab: everything sent to and received from the controller.",
            "Network: a device can connect over TCP, for example to ser2net on a Raspberry Pi. On Linux, the entry Share over network (ser2net)… of the Machine menu prepares ser2net for the laser plugged into this computer.",
        ],
    },
    Section {
        title: "Camera",
        lines: &[
            "Give a device a camera URL (MJPEG stream or JPEG snapshot) to get the Camera view button. It can float, open as a separate window or dock as a tab (the three icons in the window); rotate the picture in the device settings. The mouse wheel or the zoom buttons in the lower left corner of the picture zoom it, drag moves it, and the third button or a double click returns to the whole picture.",
            "The Overlay button (also in the camera window) shows or hides the picture under the design; while the camera window is open it shows the live picture. In the window of View > Overlay from… a USB camera can be chosen, and the corners are kept with the device.",
        ],
    },
];

const PL: &[Section] = &[
    Section {
        title: "Pierwsze kroki",
        lines: &[
            "Okno startowe pyta, z którym urządzeniem pracujesz. Najpierw utwórz profil urządzenia (Dodaj urządzenie…): nazwa, sterownik (GRBL, Marlin, Ruida, Trocen), typ lasera, obszar roboczy, jednostki i sposób połączenia. Wybierz profil i naciśnij Nowy lub Otwórz…. Bez maszyny naciśnij Nie mam urządzenia (albo po prostu Nowy): użyte będzie zastępcze urządzenie Dummy.",
            "Profile są zapamiętywane.",
            "Po wybraniu Nowy okno Plik > Ustawienia projektu… pyta o kawałek materiału: blok (rozmiar i grubość) albo walec na obrotnicy. Obrabiane strony: 2 strony dają stronę A i stronę B, odwracane wokół osi poziomej (H) albo pionowej (V), ze znacznikami narożników do ponownego ułożenia materiału: przełącznik nad obszarem roboczym pokazuje jedną albo drugą stronę, z duchami drugiej strony odwróconymi. Kliknij ducha, żeby wyrównać do niego zaznaczone kształty lub je do niego przyciągać; duch nigdy się nie przesuwa. Zapisz G-code pyta, czy obie strony zapisać w jednym pliku, z zatrzymaniem (M0) na odwrócenie materiału lub bez, czy w dwóch plikach z końcówką _A i _B; każda strona ma w nagłówku informację, którą jest. Podgląd G-code pyta tak samo i pokazuje cały projekt w jednej karcie albo każdą stronę w osobnej; Zapisz G-code w tej karcie też pyta.",
            "Znaczniki wyrównania dwóch stron są osobnym zadaniem: leżą tuż poza materiałem, więc Zapisz G-code proponuje je w osobnym pliku (nazwa_marks), wypalanym najpierw na arkuszu pod materiałem; potem materiał wkłada się w nie dla strony A i, po odwróceniu, dla strony B. Są rysowane na czerwono wokół materiału, nie jako kształty.",
            "4 strony (słupek) i 6 stron (kostka) w Plik > Ustawienia projektu…: słupek o podstawie a × b i wysokości H leży wzdłuż X i jest wypalany na czterech długich ścianach A–D, obracany między nimi o ćwierć obrotu do tyłu (przód idzie do góry); kostka a × b × c jest wypalana na wszystkich sześciu ścianach, E i F to jej lewy i prawy koniec skierowany do góry. Każda ściana ma własny obszar roboczy wielkości ściany; obszar roboczy to zawsze materiał (także dla 2 stron). Wokół pokazanej ściany są rozłożone ściany sąsiednie z ich duchami, a ściana pod nią prześwituje. Zapisz G-code zapisuje wszystkie ściany w jednym pliku (z zatrzymaniem przed każdą ścianą, które mówi, jak ułożyć przedmiot, i przypomnieniem o ostrości na jego górze) albo w pliku dla każdej ściany (_A … _F).",
            "Podgląd > 3D pokazuje wypalony przedmiot wielkości materiału z ustawień, z zadaniami wszystkich jego ścian (pokazana ściana odtwarza się do suwaka): przeciągnij, by go obrócić, kółko przybliża, kostka w rogu ustawia widok od strony. Kolory wypalenia rysują linie ciemniejsze przy większej mocy, jak na drewnie. Płasko wraca do obszaru roboczego.",
        ],
    },
    Section {
        title: "Warstwy i ustawienia cięcia",
        lines: &[
            "Każdy obiekt należy do jednej z 64 kolorowych warstw. Kliknij kolor na dolnym pasku, aby uczynić warstwę aktywną (zaznaczone obiekty przechodzą na nią); dwukrotne kliknięcie nazwy otwiera wszystkie opcje warstwy.",
            "Warstwa ma tryb (Linia, Wypełnienie, Wypełnienie + linia, Wypełnienie przesunięciem), prędkość, moc, przejścia, pompę powietrza, odstęp, kąt, overscan, a dla warstw z obrazami rastrowanie i moc minimalną. Karta Warstwy pokazuje warstwy w kolejności wypalania; strzałkami zmieniasz ją. Trzy przełączniki na końcu wiersza to wyjście (wypalaj), widoczność i pompa powietrza.",
            "Zapisz preset… i Wczytaj preset nad ustawieniami warstwy zachowują sposób wypalania pod nazwą i nakładają go później na aktywną warstwę. Sprawdź preset na odpadzie, bo ta sama moc daje inny wynik na innym laserze.",
            "Na warstwie Linia Wejście / Wyjście daje każdemu zamkniętemu obrysowi krótką linię albo łuk od strony odpadu i zakładkę za punktem startu, żeby ślad w miejscu włączenia lasera nie był na części.",
            "Prędkości są pokazywane na sekundę lub na minutę, w milimetrach lub calach, zależnie od opcji urządzenia (Jednostki prędkości). Nowe warstwy startują z 1000 mm/min; ustawienia warstw należą do projektu.",
        ],
    },
    Section {
        title: "Podgląd i wyjście",
        lines: &[
            "Pasek sterowania: nazwa bieżącego narzędzia, Nowy / Otwórz / Zapisz, przełączniki siatki, siatki pomocniczej i przyciągania, przycisk Ścieżki (zielony, gdy ścieżki są pokazane), Podgląd… i Widok z kamery (błękitne, gdy ich okno jest otwarte; ponowne kliknięcie zamyka je) oraz Nakładka. Start zmienia się w STOP podczas pracy; Start i Obrys wymagają podłączonego lasera.",
            "Ścieżki pokazują trasy na obszarze roboczym; Podgląd… otwiera okno podglądu, przybliżone do obiektów do wypalenia, z każdą operacją w osobnym kolorze, ruchami jałowymi, szacowanym czasem i suwakiem odtwarzania. Pasek powiększenia w lewym dolnym rogu obrazu przybliża i oddala oraz dopasowuje widok do obszaru roboczego lub do wszystkich obiektów. Czerwony pierścień to zero maszyny.",
            "Start wysyła zadanie do podłączonego lasera GRBL lub Marlin, a Obrys obrysowuje zadanie. Dla Ruida i Trocen Start eksportuje plik PLT lub DXF dla oprogramowania sterownika. Plik > Eksportuj zapisuje SVG, G-code lub PLT / DXF.",
            "Karta Urządzenie: połączenie, a pod linią Bazuj, Odblokuj, Pauza i Wznów w układzie 2 x 2 oraz szeroki przycisk STOP (aktywny tylko, gdy maszyna pracuje). Karta Sterowanie: te same cztery przyciski i STOP u góry, pozycja, krok i posuw ruchu oraz strzałki (pomarańczowa anuluje ruch), wiązka ostrości, makra, start od, moc obrysu i pozycja na żywo; zera pracy nie przesuwa się stamtąd. Karta Konsola: wszystko, co wysłano do sterownika i od niego odebrano.",
            "Sieć: urządzenie może łączyć się przez TCP, np. z ser2net na Raspberry Pi. Na Linuksie pozycja Udostępnij w sieci (ser2net)… menu Maszyna przygotowuje ser2net dla lasera podłączonego do tego komputera.",
        ],
    },
    Section {
        title: "Kamera",
        lines: &[
            "Podaj w urządzeniu adres URL kamery (strumień MJPEG lub zdjęcie JPEG), aby pojawił się przycisk Widok z kamery. Okno może pływać, być osobne albo zadokowane jako karta (trzy ikony w oknie); obrót obrazu ustawisz w ustawieniach urządzenia. Kółko myszy lub przyciski powiększenia w lewym dolnym rogu obrazu powiększają go, przeciąganie go przesuwa, a trzeci przycisk lub podwójne kliknięcie wraca do całego obrazu.",
            "Przycisk Nakładka (także w oknie kamery) pokazuje lub ukrywa obraz pod projektem; przy otwartym oknie kamery pokazuje obraz na żywo. W oknie Widok > Nakładka z… można wybrać kamerę USB, a narożniki są zapisywane przy urządzeniu.",
        ],
    },
];

/// The guide in English or Polish.
pub fn sections_in(polish: bool) -> Vec<Section> {
    let first = if polish { PL } else { EN };
    base_ui::guide::sections(Own { first, last: &[], machine: true, picture: Picture::Camera }, polish)
}

/// The guide in the language of the interface.
pub fn sections() -> Vec<Section> {
    sections_in(lang() == Lang::Pl)
}
