//! Tiny translation layer. English is the source language: `tr("English text")` returns the
//! translation for the active language, or the English text itself when none exists.
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Lang {
    En,
    Pl,
    De,
    It,
    Fi,
    Zh,
    Hi,
}

impl Lang {
    pub const ALL: [Lang; 7] = [Lang::En, Lang::Pl, Lang::De, Lang::It, Lang::Fi, Lang::Zh, Lang::Hi];

    /// Name shown in the language menu (always in the language itself, plus English for scripts that
    /// might lack a font on a bare system).
    pub fn name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Pl => "Polski",
            Lang::De => "Deutsch",
            Lang::It => "Italiano",
            Lang::Fi => "Suomi",
            Lang::Zh => "中文 (Chinese)",
            Lang::Hi => "हिन्दी (Hindi)",
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Pl => "pl",
            Lang::De => "de",
            Lang::It => "it",
            Lang::Fi => "fi",
            Lang::Zh => "zh",
            Lang::Hi => "hi",
        }
    }
    pub fn from_code(s: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.code() == s)
    }
    fn index(self) -> u8 {
        Lang::ALL.iter().position(|l| *l == self).unwrap_or(0) as u8
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn set_lang(l: Lang) {
    CURRENT.store(l.index(), Ordering::Relaxed);
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn lang() -> Lang {
    Lang::ALL[CURRENT.load(Ordering::Relaxed) as usize % Lang::ALL.len()]
}

/// Translate an English UI string.
pub fn tr(s: &'static str) -> &'static str {
    let i = CURRENT.load(Ordering::Relaxed) as usize;
    if i == 0 {
        return s;
    }
    static MAP: OnceLock<HashMap<&'static str, &'static [&'static str; 7]>> = OnceLock::new();
    let map = MAP.get_or_init(|| TABLE.iter().map(|row| (row[0], row)).collect());
    match map.get(s) {
        Some(row) if !row[i].is_empty() => row[i],
        _ => s,
    }
}

/// Translate and substitute each `{}` with the next argument.
pub fn trf(s: &'static str, args: &[&dyn Display]) -> String {
    let mut out = tr(s).to_string();
    for a in args {
        out = out.replacen("{}", &a.to_string(), 1);
    }
    out
}

// Columns: English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी
#[rustfmt::skip]
const TABLE: &[[&str; 7]] = &[
    // ---- menus ----
    ["File", "Plik", "Datei", "File", "Tiedosto", "文件", "फ़ाइल"],
    ["Edit", "Edycja", "Bearbeiten", "Modifica", "Muokkaa", "编辑", "संपादन"],
    ["Arrange", "Rozmieść", "Anordnen", "Disponi", "Järjestä", "排列", "व्यवस्थित करें"],
    ["View", "Widok", "Ansicht", "Vista", "Näytä", "视图", "दृश्य"],
    ["Laser", "Laser", "Laser", "Laser", "Laser", "激光", "लेज़र"],
    ["Settings", "Ustawienia", "Einstellungen", "Impostazioni", "Asetukset", "设置", "सेटिंग्स"],
    ["Help", "Pomoc", "Hilfe", "Aiuto", "Ohje", "帮助", "सहायता"],
    ["New", "Nowy", "Neu", "Nuovo", "Uusi", "新建", "नया"],
    ["Open…", "Otwórz…", "Öffnen…", "Apri…", "Avaa…", "打开…", "खोलें…"],
    ["Save", "Zapisz", "Speichern", "Salva", "Tallenna", "保存", "सहेजें"],
    ["Save as…", "Zapisz jako…", "Speichern unter…", "Salva come…", "Tallenna nimellä…", "另存为…", "इस रूप में सहेजें…"],
    ["Import SVG…", "Importuj SVG…", "SVG importieren…", "Importa SVG…", "Tuo SVG…", "导入 SVG…", "SVG आयात करें…"],
    ["Export SVG…", "Eksportuj SVG…", "SVG exportieren…", "Esporta SVG…", "Vie SVG…", "导出 SVG…", "SVG निर्यात करें…"],
    ["Export G-code…", "Eksportuj G-code…", "G-Code exportieren…", "Esporta G-code…", "Vie G-koodi…", "导出 G-code…", "G-code निर्यात करें…"],
    ["Quit", "Zakończ", "Beenden", "Esci", "Lopeta", "退出", "बाहर निकलें"],
    ["Undo", "Cofnij", "Rückgängig", "Annulla", "Kumoa", "撤销", "पूर्ववत करें"],
    ["Redo", "Ponów", "Wiederholen", "Ripeti", "Tee uudelleen", "重做", "फिर से करें"],
    ["Copy", "Kopiuj", "Kopieren", "Copia", "Kopioi", "复制", "कॉपी करें"],
    ["Paste", "Wklej", "Einfügen", "Incolla", "Liitä", "粘贴", "चिपकाएँ"],
    ["Duplicate", "Powiel", "Duplizieren", "Duplica", "Monista", "复制副本", "डुप्लिकेट"],
    ["Delete", "Usuń", "Löschen", "Elimina", "Poista", "删除", "हटाएँ"],
    ["Select all", "Zaznacz wszystko", "Alles auswählen", "Seleziona tutto", "Valitse kaikki", "全选", "सभी चुनें"],
    ["Align left", "Wyrównaj do lewej", "Links ausrichten", "Allinea a sinistra", "Tasaa vasemmalle", "左对齐", "बाएँ संरेखित करें"],
    ["Align centre (H)", "Wyśrodkuj w poziomie", "Horizontal zentrieren", "Centra orizzontalmente", "Keskitä vaakasuunnassa", "水平居中", "क्षैतिज केंद्रित करें"],
    ["Align right", "Wyrównaj do prawej", "Rechts ausrichten", "Allinea a destra", "Tasaa oikealle", "右对齐", "दाएँ संरेखित करें"],
    ["Align top", "Wyrównaj do góry", "Oben ausrichten", "Allinea in alto", "Tasaa ylös", "顶部对齐", "ऊपर संरेखित करें"],
    ["Align centre (V)", "Wyśrodkuj w pionie", "Vertikal zentrieren", "Centra verticalmente", "Keskitä pystysuunnassa", "垂直居中", "ऊर्ध्वाधर केंद्रित करें"],
    ["Align bottom", "Wyrównaj do dołu", "Unten ausrichten", "Allinea in basso", "Tasaa alas", "底部对齐", "नीचे संरेखित करें"],
    ["Centre on bed", "Wyśrodkuj na stole", "Auf Arbeitsfläche zentrieren", "Centra sul piano", "Keskitä alustalle", "在工作台居中", "बेड पर केंद्रित करें"],
    ["Flip horizontal", "Odbij w poziomie", "Horizontal spiegeln", "Rifletti orizzontalmente", "Käännä vaakasuunnassa", "水平翻转", "क्षैतिज पलटें"],
    ["Flip vertical", "Odbij w pionie", "Vertikal spiegeln", "Rifletti verticalmente", "Käännä pystysuunnassa", "垂直翻转", "ऊर्ध्वाधर पलटें"],
    ["Rotate 90° CW", "Obróć 90° w prawo", "90° im Uhrzeigersinn drehen", "Ruota di 90° orari", "Kierrä 90° myötäpäivään", "顺时针旋转 90°", "90° दक्षिणावर्त घुमाएँ"],
    ["Rotate 90° CCW", "Obróć 90° w lewo", "90° gegen den Uhrzeigersinn drehen", "Ruota di 90° antiorari", "Kierrä 90° vastapäivään", "逆时针旋转 90°", "90° वामावर्त घुमाएँ"],
    ["Bring to front", "Przesuń na wierzch", "In den Vordergrund", "Porta in primo piano", "Tuo eteen", "置于顶层", "सबसे आगे लाएँ"],
    ["Send to back", "Przesuń na spód", "In den Hintergrund", "Porta in secondo piano", "Vie taakse", "置于底层", "सबसे पीछे भेजें"],
    ["Convert to path", "Zamień na ścieżkę", "In Pfad umwandeln", "Converti in tracciato", "Muunna poluksi", "转换为路径", "पाथ में बदलें"],
    ["Grid array…", "Tablica siatki…", "Rasteranordnung…", "Matrice a griglia…", "Ruudukkotaulukko…", "网格阵列…", "ग्रिड ऐरे…"],
    ["Grid", "Siatka", "Raster", "Griglia", "Ruudukko", "网格", "ग्रिड"],
    ["Snap to grid", "Przyciągaj do siatki", "Am Raster einrasten", "Aggancia alla griglia", "Kohdista ruudukkoon", "对齐网格", "ग्रिड पर स्नैप करें"],
    ["Toolpath preview", "Podgląd ścieżek", "Werkzeugweg-Vorschau", "Anteprima percorsi", "Työstöratojen esikatselu", "刀路预览", "टूलपाथ पूर्वावलोकन"],
    ["Fit bed to window", "Dopasuj stół do okna", "Arbeitsfläche einpassen", "Adatta il piano alla finestra", "Sovita alusta ikkunaan", "适应窗口", "बेड को विंडो में फिट करें"],
    ["Device settings…", "Ustawienia urządzenia…", "Geräteeinstellungen…", "Impostazioni dispositivo…", "Laitteen asetukset…", "设备设置…", "डिवाइस सेटिंग्स…"],
    ["Frame", "Obrys", "Rahmen", "Cornice", "Kehys", "走边框", "फ़्रेम"],
    ["Start job", "Rozpocznij zadanie", "Auftrag starten", "Avvia lavoro", "Aloita työ", "开始任务", "जॉब शुरू करें"],
    ["About", "O programie", "Über", "Informazioni", "Tietoja", "关于", "के बारे में"],
    ["Language", "Język", "Sprache", "Lingua", "Kieli", "语言", "भाषा"],
    ["Colour scheme", "Schemat kolorów", "Farbschema", "Combinazione colori", "Värimaailma", "配色方案", "रंग योजना"],
    ["Dark", "Ciemny", "Dunkel", "Scuro", "Tumma", "深色", "गहरा"],
    ["Light Dark", "Jasnociemny", "Helldunkel", "Scuro chiaro", "Vaalea tumma", "浅深色", "हल्का गहरा"],
    ["Medium Light", "Średnio jasny", "Mittelhell", "Medio chiaro", "Keskivaalea", "中浅色", "मध्यम हल्का"],
    ["Light", "Jasny", "Hell", "Chiaro", "Vaalea", "浅色", "हल्का"],
    // ---- tools / control bar ----
    ["Select (V)", "Zaznaczanie (V)", "Auswahl (V)", "Selezione (V)", "Valinta (V)", "选择 (V)", "चयन (V)"],
    ["Rectangle (R)", "Prostokąt (R)", "Rechteck (R)", "Rettangolo (R)", "Suorakulmio (R)", "矩形 (R)", "आयत (R)"],
    ["Ellipse (E)", "Elipsa (E)", "Ellipse (E)", "Ellisse (E)", "Ellipsi (E)", "椭圆 (E)", "दीर्घवृत्त (E)"],
    ["Line (L)", "Linia (L)", "Linie (L)", "Linea (L)", "Viiva (L)", "直线 (L)", "रेखा (L)"],
    ["Polyline / pen (P)", "Linia łamana / pióro (P)", "Polylinie / Stift (P)", "Polilinea / penna (P)", "Murtoviiva / kynä (P)", "折线 / 钢笔 (P)", "पॉलीलाइन / पेन (P)"],
    ["Pan (H)", "Przesuwanie widoku (H)", "Verschieben (H)", "Sposta vista (H)", "Panoroi (H)", "平移 (H)", "पैन (H)"],
    ["Zoom (Z)", "Powiększenie (Z)", "Zoom (Z)", "Zoom (Z)", "Zoomaus (Z)", "缩放 (Z)", "ज़ूम (Z)"],
    ["Select", "Zaznaczanie", "Auswahl", "Selezione", "Valinta", "选择", "चयन"],
    ["Rectangle", "Prostokąt", "Rechteck", "Rettangolo", "Suorakulmio", "矩形", "आयत"],
    ["Ellipse", "Elipsa", "Ellipse", "Ellisse", "Ellipsi", "椭圆", "दीर्घवृत्त"],
    ["Line", "Linia", "Linie", "Linea", "Viiva", "直线", "रेखा"],
    ["Fill", "Wypełnienie", "Füllung", "Riempimento", "Täyttö", "填充", "भरें"],
    ["Fill + Line", "Wypełnienie + linia", "Füllung + Linie", "Riempimento + linea", "Täyttö + viiva", "填充 + 线条", "भरें + रेखा"],
    ["Layer {}", "Warstwa {}", "Ebene {}", "Livello {}", "Taso {}", "图层 {}", "लेयर {}"],
    ["Snap", "Przyciąganie", "Einrasten", "Aggancio", "Kohdistus", "吸附", "स्नैप"],
    ["Preview", "Podgląd", "Vorschau", "Anteprima", "Esikatselu", "预览", "पूर्वावलोकन"],
    ["Start", "Start", "Start", "Avvia", "Käynnistä", "开始", "शुरू"],
    ["Stream the job to the connected laser", "Wyślij zadanie do podłączonego lasera", "Auftrag an den verbundenen Laser senden", "Invia il lavoro al laser collegato", "Lähetä työ yhdistettyyn laseriin", "将任务发送到已连接的激光器", "जॉब को जुड़े लेज़र पर भेजें"],
    ["Trace the bounding box", "Obrysuj ramkę ograniczającą", "Begrenzungsrahmen abfahren", "Percorri il riquadro di delimitazione", "Seuraa rajauslaatikkoa", "沿边界框走一圈", "बाउंडिंग बॉक्स का आउटलाइन चलाएँ"],
    ["Save G-code", "Zapisz G-code", "G-Code speichern", "Salva G-code", "Tallenna G-koodi", "保存 G-code", "G-code सहेजें"],
    ["Layers", "Warstwy", "Ebenen", "Livelli", "Tasot", "图层", "लेयर"],
    ["{} — click to assign selection / set active", "{} — kliknij, aby przypisać zaznaczenie / ustawić jako aktywną", "{} — klicken, um die Auswahl zuzuweisen / zu aktivieren", "{} — clic per assegnare la selezione / impostare come attivo", "{} — napsauta määrittääksesi valinnan / asettaaksesi aktiiviseksi", "{} — 点击以分配所选对象 / 设为当前图层", "{} — चयन असाइन करने / सक्रिय सेट करने के लिए क्लिक करें"],
    // ---- status messages ----
    ["Ready", "Gotowe", "Bereit", "Pronto", "Valmis", "就绪", "तैयार"],
    ["Opened {}", "Otwarto {}", "{} geöffnet", "Aperto {}", "Avattu {}", "已打开 {}", "{} खोला गया"],
    ["Open failed: {}", "Nie udało się otworzyć: {}", "Öffnen fehlgeschlagen: {}", "Apertura non riuscita: {}", "Avaus epäonnistui: {}", "打开失败：{}", "खोलना विफल: {}"],
    ["Saved {}", "Zapisano {}", "{} gespeichert", "Salvato {}", "Tallennettu {}", "已保存 {}", "{} सहेजा गया"],
    ["Save failed: {}", "Nie udało się zapisać: {}", "Speichern fehlgeschlagen: {}", "Salvataggio non riuscito: {}", "Tallennus epäonnistui: {}", "保存失败：{}", "सहेजना विफल: {}"],
    ["Imported {} paths from {}", "Zaimportowano {} ścieżek z {}", "{} Pfade aus {} importiert", "Importati {} tracciati da {}", "Tuotu {} polkua kohteesta {}", "已从 {} 导入 {} 条路径", "{} से {} पाथ आयात किए गए"],
    ["Import failed: {}", "Nie udało się zaimportować: {}", "Import fehlgeschlagen: {}", "Importazione non riuscita: {}", "Tuonti epäonnistui: {}", "导入失败：{}", "आयात विफल: {}"],
    ["Exported {}", "Wyeksportowano {}", "{} exportiert", "Esportato {}", "Viety {}", "已导出 {}", "{} निर्यात किया गया"],
    ["Export failed: {}", "Nie udało się wyeksportować: {}", "Export fehlgeschlagen: {}", "Esportazione non riuscita: {}", "Vienti epäonnistui: {}", "导出失败：{}", "निर्यात विफल: {}"],
    ["Saved G-code {}", "Zapisano G-code {}", "G-Code {} gespeichert", "G-code salvato {}", "G-koodi tallennettu {}", "已保存 G-code {}", "G-code {} सहेजा गया"],
    ["Connect to a laser first (Laser panel)", "Najpierw połącz się z laserem (panel Laser)", "Zuerst mit einem Laser verbinden (Laser-Bereich)", "Collegarsi prima a un laser (pannello Laser)", "Yhdistä ensin laseriin (Laser-paneeli)", "请先连接激光器（激光面板）", "पहले लेज़र से कनेक्ट करें (लेज़र पैनल)"],
    ["Nothing to burn: no shapes on output layers", "Brak danych do wypalenia: żadnych kształtów na warstwach wyjściowych", "Nichts zu brennen: keine Formen auf Ausgabeebenen", "Niente da incidere: nessuna forma sui livelli di output", "Ei poltettavaa: ei muotoja tulostustasoilla", "无可雕刻内容：输出图层上没有图形", "जलाने के लिए कुछ नहीं: आउटपुट लेयर पर कोई आकार नहीं"],
    ["Starting job: {} lines, ~{}", "Start zadania: {} linii, ok. {}", "Auftrag startet: {} Zeilen, ca. {}", "Avvio lavoro: {} righe, ~{}", "Työn aloitus: {} riviä, ~{}", "开始任务：{} 行，约 {}", "जॉब शुरू: {} पंक्तियाँ, ~{}"],
    ["Disconnected", "Rozłączono", "Getrennt", "Disconnesso", "Ei yhteyttä", "未连接", "डिस्कनेक्टेड"],
    ["Connected to {} @ {}", "Połączono z {} @ {}", "Verbunden mit {} @ {}", "Connesso a {} @ {}", "Yhdistetty: {} @ {}", "已连接到 {} @ {}", "{} @ {} से कनेक्ट हुआ"],
    ["Cannot open {}: {}", "Nie można otworzyć {}: {}", "{} kann nicht geöffnet werden: {}", "Impossibile aprire {}: {}", "Kohdetta {} ei voi avata: {}", "无法打开 {}：{}", "{} नहीं खुल सका: {}"],
    ["Aborted (soft reset)", "Przerwano (reset programowy)", "Abgebrochen (Soft-Reset)", "Interrotto (soft reset)", "Keskeytetty (ohjelmallinen nollaus)", "已中止（软复位）", "रोका गया (सॉफ़्ट रीसेट)"],
    ["Connection lost", "Utracono połączenie", "Verbindung verloren", "Connessione persa", "Yhteys katkesi", "连接丢失", "कनेक्शन टूट गया"],
    ["Estimated time", "Szacowany czas", "Geschätzte Zeit", "Tempo stimato", "Arvioitu aika", "预计时间", "अनुमानित समय"],
    ["cut length", "długość cięcia", "Schnittlänge", "lunghezza taglio", "leikkauspituus", "切割长度", "कट लंबाई"],
    // ---- dialogs ----
    ["Device settings", "Ustawienia urządzenia", "Geräteeinstellungen", "Impostazioni dispositivo", "Laitteen asetukset", "设备设置", "डिवाइस सेटिंग्स"],
    ["Name", "Nazwa", "Name", "Nome", "Nimi", "名称", "नाम"],
    ["Work area X (mm)", "Obszar roboczy X (mm)", "Arbeitsbereich X (mm)", "Area di lavoro X (mm)", "Työalue X (mm)", "工作区域 X (mm)", "कार्य क्षेत्र X (mm)"],
    ["Work area Y (mm)", "Obszar roboczy Y (mm)", "Arbeitsbereich Y (mm)", "Area di lavoro Y (mm)", "Työalue Y (mm)", "工作区域 Y (mm)", "कार्य क्षेत्र Y (mm)"],
    ["Machine zero (0,0)", "Zero maszyny (0,0)", "Maschinennullpunkt (0,0)", "Zero macchina (0,0)", "Koneen nollapiste (0,0)", "机器原点 (0,0)", "मशीन शून्य (0,0)"],
    ["Front-left (GRBL default)", "Przód-lewo (domyślne GRBL)", "Vorne links (GRBL-Standard)", "Anteriore sinistra (predefinito GRBL)", "Etuvasen (GRBL-oletus)", "左前（GRBL 默认）", "आगे-बाएँ (GRBL डिफ़ॉल्ट)"],
    ["Back-left", "Tył-lewo", "Hinten links", "Posteriore sinistra", "Takavasen", "左后", "पीछे-बाएँ"],
    ["S-value max ($30)", "Maks. wartość S ($30)", "Max. S-Wert ($30)", "Valore S max ($30)", "S-arvon maksimi ($30)", "S 值最大值 ($30)", "S-मान अधिकतम ($30)"],
    ["Dynamic power (M4)", "Moc dynamiczna (M4)", "Dynamische Leistung (M4)", "Potenza dinamica (M4)", "Dynaaminen teho (M4)", "动态功率 (M4)", "डायनामिक पावर (M4)"],
    ["Travel speed (mm/min)", "Prędkość jałowa (mm/min)", "Eilganggeschwindigkeit (mm/min)", "Velocità di spostamento (mm/min)", "Siirtonopeus (mm/min)", "空走速度 (mm/min)", "यात्रा गति (mm/min)"],
    ["Return to origin", "Powrót do początku", "Zum Ursprung zurückkehren", "Ritorna all'origine", "Palaa origoon", "返回原点", "मूल बिंदु पर लौटें"],
    ["Baud rate", "Szybkość transmisji", "Baudrate", "Baud rate", "Siirtonopeus (baud)", "波特率", "बॉड दर"],
    ["Grid array", "Tablica siatki", "Rasteranordnung", "Matrice a griglia", "Ruudukkotaulukko", "网格阵列", "ग्रिड ऐरे"],
    ["Columns", "Kolumny", "Spalten", "Colonne", "Sarakkeet", "列数", "कॉलम"],
    ["Rows", "Wiersze", "Zeilen", "Righe", "Rivit", "行数", "पंक्तियाँ"],
    ["Gap X (mm)", "Odstęp X (mm)", "Abstand X (mm)", "Spazio X (mm)", "Väli X (mm)", "间距 X (mm)", "अंतर X (mm)"],
    ["Gap Y (mm)", "Odstęp Y (mm)", "Abstand Y (mm)", "Spazio Y (mm)", "Väli Y (mm)", "间距 Y (mm)", "अंतर Y (mm)"],
    ["Create array", "Utwórz tablicę", "Anordnung erstellen", "Crea matrice", "Luo taulukko", "创建阵列", "ऐरे बनाएँ"],
    ["About LightCreator", "O programie LightCreator", "Über LightCreator", "Informazioni su LightCreator", "Tietoja: LightCreator", "关于 LightCreator", "LightCreator के बारे में"],
    ["Version {}", "Wersja {}", "Version {}", "Versione {}", "Versio {}", "版本 {}", "संस्करण {}"],
    ["Open-source design & control software for laser engravers and cutters.", "Otwartoźródłowe oprogramowanie do projektowania i sterowania grawerami i wycinarkami laserowymi.", "Open-Source-Software zum Entwerfen und Steuern von Lasergravierern und -schneidern.", "Software open source per progettare e controllare incisori e taglierine laser.", "Avoimen lähdekoodin suunnittelu- ja ohjausohjelmisto laserkaiverrus- ja leikkuulaitteille.", "用于激光雕刻机和切割机的开源设计与控制软件。", "लेज़र उत्कीर्णक और कटर के लिए ओपन-सोर्स डिज़ाइन और नियंत्रण सॉफ़्टवेयर।"],
    ["Origin: written in Rust with egui/eframe.", "Pochodzenie: napisany w Rust z użyciem egui/eframe.", "Herkunft: in Rust mit egui/eframe geschrieben.", "Origine: scritto in Rust con egui/eframe.", "Alkuperä: kirjoitettu Rustilla egui/eframe-kirjastoilla.", "来源：使用 Rust 和 egui/eframe 编写。", "उत्पत्ति: Rust में egui/eframe के साथ लिखा गया।"],
    ["Inspired by LightBurn; interface inspired by VectorCraft.", "Zainspirowany LightBurn; interfejs inspirowany VectorCraft.", "Inspiriert von LightBurn; Oberfläche inspiriert von VectorCraft.", "Ispirato a LightBurn; interfaccia ispirata a VectorCraft.", "LightBurnin innoittama; käyttöliittymä VectorCraftin innoittama.", "灵感来自 LightBurn；界面灵感来自 VectorCraft。", "LightBurn से प्रेरित; इंटरफ़ेस VectorCraft से प्रेरित।"],
    ["Author: Michał Sarna", "Autor: Michał Sarna", "Autor: Michał Sarna", "Autore: Michał Sarna", "Tekijä: Michał Sarna", "作者：Michał Sarna", "लेखक: Michał Sarna"],
    ["MIT licensed.", "Licencja MIT.", "MIT-lizenziert.", "Licenza MIT.", "MIT-lisensioitu.", "MIT 许可证。", "MIT लाइसेंस।"],
    // ---- side panels ----
    ["Properties", "Właściwości", "Eigenschaften", "Proprietà", "Ominaisuudet", "属性", "गुण"],
    ["Cuts / Layers", "Cięcia / warstwy", "Schnitte / Ebenen", "Tagli / livelli", "Leikkaukset / tasot", "切割 / 图层", "कट / लेयर"],
    ["Nothing selected", "Nic nie zaznaczono", "Nichts ausgewählt", "Nessuna selezione", "Ei valintaa", "未选择任何内容", "कुछ चयनित नहीं"],
    ["Lock aspect ratio", "Zablokuj proporcje", "Seitenverhältnis sperren", "Blocca proporzioni", "Lukitse kuvasuhde", "锁定宽高比", "आस्पेक्ट रेशियो लॉक करें"],
    ["Rotate", "Obróć", "Drehen", "Ruota", "Kierrä", "旋转", "घुमाएँ"],
    ["Apply", "Zastosuj", "Anwenden", "Applica", "Käytä", "应用", "लागू करें"],
    ["Flip H", "Odbij H", "Spiegeln H", "Rifletti O", "Käännä V", "水平翻转", "पलटें H"],
    ["Flip V", "Odbij V", "Spiegeln V", "Rifletti V", "Käännä P", "垂直翻转", "पलटें V"],
    ["Centre", "Środek", "Mitte", "Centro", "Keskitä", "居中", "केंद्र"],
    ["Left", "Lewo", "Links", "Sinistra", "Vasen", "左", "बाएँ"],
    ["Mid-H", "Śr.-H", "Mitte-H", "Centro-O", "Kesk.-V", "水平中", "मध्य-H"],
    ["Right", "Prawo", "Rechts", "Destra", "Oikea", "右", "दाएँ"],
    ["Top", "Góra", "Oben", "Alto", "Ylä", "上", "ऊपर"],
    ["Mid-V", "Śr.-V", "Mitte-V", "Centro-V", "Kesk.-P", "垂直中", "मध्य-V"],
    ["Bottom", "Dół", "Unten", "Basso", "Ala", "下", "नीचे"],
    ["Align centre", "Wyśrodkuj", "Zentrieren", "Centra", "Keskitä", "居中对齐", "केंद्र संरेखित करें"],
    ["Align middle", "Wyrównaj do środka", "Mittig ausrichten", "Allinea al centro", "Tasaa keskelle", "居中对齐（垂直）", "मध्य संरेखित करें"],
    ["Layer", "Warstwa", "Ebene", "Livello", "Taso", "图层", "लेयर"],
    ["Show all 30 layers", "Pokaż wszystkie 30 warstw", "Alle 30 Ebenen anzeigen", "Mostra tutti i 30 livelli", "Näytä kaikki 30 tasoa", "显示全部 30 个图层", "सभी 30 लेयर दिखाएँ"],
    ["Output (burn this layer)", "Wyjście (wypal tę warstwę)", "Ausgabe (diese Ebene brennen)", "Output (incidi questo livello)", "Tuloste (polta tämä taso)", "输出（雕刻此图层）", "आउटपुट (इस लेयर को जलाएँ)"],
    ["Visible", "Widoczna", "Sichtbar", "Visibile", "Näkyvä", "可见", "दृश्यमान"],
    ["Cut settings — {}", "Ustawienia cięcia — {}", "Schnitteinstellungen — {}", "Impostazioni di taglio — {}", "Leikkausasetukset — {}", "切割设置 — {}", "कट सेटिंग्स — {}"],
    ["Mode", "Tryb", "Modus", "Modalità", "Tila", "模式", "मोड"],
    ["Speed (mm/s)", "Prędkość (mm/s)", "Geschwindigkeit (mm/s)", "Velocità (mm/s)", "Nopeus (mm/s)", "速度 (mm/s)", "गति (mm/s)"],
    ["Power (%)", "Moc (%)", "Leistung (%)", "Potenza (%)", "Teho (%)", "功率 (%)", "पावर (%)"],
    ["Passes", "Przejścia", "Durchgänge", "Passate", "Kierrokset", "次数", "पास"],
    ["Interval (mm)", "Odstęp linii (mm)", "Linienabstand (mm)", "Intervallo (mm)", "Riviväli (mm)", "线间距 (mm)", "अंतराल (mm)"],
    ["Scan angle", "Kąt skanowania", "Scanwinkel", "Angolo di scansione", "Skannauskulma", "扫描角度", "स्कैन कोण"],
    ["Overscan (mm)", "Nadjazd (mm)", "Überlauf (mm)", "Overscan (mm)", "Ylijuoksu (mm)", "超扫描 (mm)", "ओवरस्कैन (mm)"],
    ["Bidirectional", "Dwukierunkowe", "Bidirektional", "Bidirezionale", "Kaksisuuntainen", "双向", "द्विदिशीय"],
    ["No port", "Brak portu", "Kein Port", "Nessuna porta", "Ei porttia", "无端口", "कोई पोर्ट नहीं"],
    ["Refresh", "Odśwież", "Aktualisieren", "Aggiorna", "Päivitä", "刷新", "रीफ़्रेश"],
    ["Rescan serial ports", "Przeskanuj porty szeregowe", "Serielle Ports neu suchen", "Riesamina le porte seriali", "Hae sarjaportit uudelleen", "重新扫描串口", "सीरियल पोर्ट फिर स्कैन करें"],
    ["Disconnect", "Rozłącz", "Trennen", "Disconnetti", "Katkaise yhteys", "断开连接", "डिस्कनेक्ट"],
    ["Connect", "Połącz", "Verbinden", "Connetti", "Yhdistä", "连接", "कनेक्ट"],
    ["State", "Stan", "Status", "Stato", "Tila", "状态", "स्थिति"],
    ["Step", "Krok", "Schritt", "Passo", "Askel", "步长", "चरण"],
    ["Feed", "Posuw", "Vorschub", "Avanzamento", "Syöttö", "进给", "फ़ीड"],
    ["Up", "Góra", "Hoch", "Su", "Ylös", "上", "ऊपर"],
    ["Down", "Dół", "Runter", "Giù", "Alas", "下", "नीचे"],
    ["Stop", "Stop", "Stopp", "Stop", "Pysäytä", "停止", "रुकें"],
    ["Cancel jog", "Anuluj przesuw", "Jog abbrechen", "Annulla jog", "Peruuta ajo", "取消点动", "जॉग रद्द करें"],
    ["Home $H", "Bazuj $H", "Referenzfahrt $H", "Home $H", "Kotiutus $H", "回零 $H", "होम $H"],
    ["Unlock $X", "Odblokuj $X", "Entsperren $X", "Sblocca $X", "Avaa lukitus $X", "解锁 $X", "अनलॉक $X"],
    ["Pause", "Pauza", "Pause", "Pausa", "Tauko", "暂停", "रोकें"],
    ["Resume", "Wznów", "Fortsetzen", "Riprendi", "Jatka", "继续", "फिर शुरू करें"],
    ["STOP", "STOP", "STOPP", "STOP", "PYSÄYTÄ", "急停", "रुकें"],
    ["Frame power", "Moc obrysu", "Rahmenleistung", "Potenza cornice", "Kehyksen teho", "走边框功率", "फ़्रेम पावर"],
    ["0 % keeps the laser off while framing; 1–2 % shows a dim dot on diode lasers", "0 % utrzymuje laser wyłączony podczas obrysu; 1–2 % pokazuje słaby punkt na laserach diodowych", "0 % hält den Laser beim Abfahren des Rahmens aus; 1–2 % zeigt bei Diodenlasern einen schwachen Punkt", "Con 0 % il laser resta spento durante la cornice; 1–2 % mostra un punto debole sui laser a diodo", "0 % pitää laserin sammuksissa kehystyksen aikana; 1–2 % näyttää himmeän pisteen diodilasereilla", "0 % 表示走边框时不出光；1–2 % 可在二极管激光器上显示微弱光点", "0 % फ़्रेमिंग के दौरान लेज़र बंद रखता है; 1–2 % डायोड लेज़र पर हल्का बिंदु दिखाता है"],
    ["send G-code / $ command", "wyślij G-code / polecenie $", "G-Code / $-Befehl senden", "invia G-code / comando $", "lähetä G-koodi / $-komento", "发送 G-code / $ 命令", "G-code / $ कमांड भेजें"],
    ["Send", "Wyślij", "Senden", "Invia", "Lähetä", "发送", "भेजें"],
    // ---- start screen / device profiles ----
    ["Select device", "Wybierz urządzenie", "Gerät auswählen", "Seleziona dispositivo", "Valitse laite", "选择设备", "डिवाइस चुनें"],
    ["Choose the device you will work with.", "Wybierz urządzenie, na którym będziesz pracować.", "Wähle das Gerät, mit dem du arbeiten möchtest.", "Scegli il dispositivo con cui lavorerai.", "Valitse laite, jolla työskentelet.", "请选择您要使用的设备。", "वह डिवाइस चुनें जिस पर आप काम करेंगे।"],
    ["Create at least one device profile to continue.", "Aby kontynuować, utwórz co najmniej jeden profil urządzenia.", "Erstelle mindestens ein Geräteprofil, um fortzufahren.", "Crea almeno un profilo dispositivo per continuare.", "Luo vähintään yksi laiteprofiili jatkaaksesi.", "请至少创建一个设备配置文件后继续。", "जारी रखने के लिए कम से कम एक डिवाइस प्रोफ़ाइल बनाएँ।"],
    ["Continue", "Kontynuuj", "Weiter", "Continua", "Jatka", "继续", "जारी रखें"],
    ["Add device…", "Dodaj urządzenie…", "Gerät hinzufügen…", "Aggiungi dispositivo…", "Lisää laite…", "添加设备…", "डिवाइस जोड़ें…"],
    ["Edit…", "Edytuj…", "Bearbeiten…", "Modifica…", "Muokkaa…", "编辑…", "संपादित करें…"],
    ["Confirm delete", "Potwierdź usunięcie", "Löschen bestätigen", "Conferma eliminazione", "Vahvista poisto", "确认删除", "हटाने की पुष्टि करें"],
    ["Cancel", "Anuluj", "Abbrechen", "Annulla", "Peruuta", "取消", "रद्द करें"],
    ["Device configuration", "Konfiguracja urządzenia", "Gerätekonfiguration", "Configurazione dispositivo", "Laitteen määritys", "设备配置", "डिवाइस कॉन्फ़िगरेशन"],
    ["New device", "Nowe urządzenie", "Neues Gerät", "Nuovo dispositivo", "Uusi laite", "新设备", "नया डिवाइस"],
    ["e.g. My diode laser", "np. Mój laser diodowy", "z. B. Mein Diodenlaser", "es. Il mio laser a diodo", "esim. Diodilaserini", "例如：我的二极管激光器", "जैसे: मेरा डायोड लेज़र"],
    ["A profile name is required.", "Nazwa profilu jest wymagana.", "Ein Profilname ist erforderlich.", "Il nome del profilo è obbligatorio.", "Profiilin nimi vaaditaan.", "必须填写配置文件名称。", "प्रोफ़ाइल का नाम आवश्यक है।"],
    ["Switch device…", "Zmień urządzenie…", "Gerät wechseln…", "Cambia dispositivo…", "Vaihda laitetta…", "切换设备…", "डिवाइस बदलें…"],
    ["Device: {}", "Urządzenie: {}", "Gerät: {}", "Dispositivo: {}", "Laite: {}", "设备：{}", "डिवाइस: {}"],
];
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_row_is_complete_and_keeps_placeholders() {
        let mut seen = std::collections::HashSet::new();
        for row in TABLE {
            assert!(seen.insert(row[0]), "duplicate key {:?}", row[0]);
            let n = row[0].matches("{}").count();
            for (i, t) in row.iter().enumerate() {
                assert!(!t.is_empty(), "empty translation {i} for {:?}", row[0]);
                assert_eq!(t.matches("{}").count(), n, "placeholder mismatch in {:?} (column {i})", row[0]);
            }
        }
    }

    #[test]
    fn lookup_and_fallback() {
        set_lang(Lang::De);
        assert_eq!(tr("Save"), "Speichern");
        assert_eq!(tr("not in table"), "not in table");
        assert_eq!(trf("Opened {}", &[&"x.svg"]), "x.svg geöffnet");
        set_lang(Lang::En);
        assert_eq!(tr("Save"), "Save");
    }
}