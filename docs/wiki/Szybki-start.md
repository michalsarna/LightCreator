🌐 [English](Quick-Start) · **Polski**

# Szybki start

1. **Uruchom program** ([Instalacja](Instalacja)). **Okno startowe** pyta, z jakim urządzeniem pracujesz. Najpierw utwórz profil: nazwa, jednostki, pole robocze, punkt zero maszyny i ustawienia sterownika. Edytor otwiera się dopiero po wybraniu profilu.
   ![Okno startowe](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/01-start.png)
2. **Sprawdź ustawienia urządzenia** w *Laser → Device settings…*: sterownik (GRBL / Marlin / Ruida / Trocen), typ lasera, pole robocze, maksymalna wartość S, port szeregowy lub adres TCP. Dla GRBL użyj *Read from device*, by wczytać je z maszyny. Zobacz [Urządzenia i sterowniki](Urzadzenia-i-sterowniki).
3. **Narysuj lub zaimportuj** projekt: narzędzia po lewej, *File → Import* dla SVG, bitmap lub Illustrator / PDF. Zobacz [Narzędzia projektowe](Narzedzia-projektowe).
4. **Przypisz warstwy cięcia.** Kliknij kolor na pasku u dołu, aby przenieść zaznaczenie na tę warstwę; dwukrotne kliknięcie nazwy warstwy otwiera tryb, prędkość, moc i liczbę przejść. Zobacz [Cięcia i warstwy](Ciecia-i-warstwy).
   ![Warstwy](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/03-layers.png)
5. **Podejrzyj** zadanie: okno podglądu pokazuje każdą operację innym kolorem, opcjonalnie ruchy jałowe oraz suwak odtwarzania z szacowanym czasem.
   ![Podgląd](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/12-preview.png)
6. **Obrysuj** projekt (zakładka Device), żeby sprawdzić położenie bez wypalania (moc obrysu to ustawienie urządzenia).
7. **Uruchom** zadanie (przycisk jest aktywny po podłączeniu lasera, a w trakcie pracy zamienia się w **STOP**). W razie potrzeby użyj pauzy / wznowienia. Zakładka **Console** pokazuje całą komunikację.
8. **Nie masz lasera?** Wyeksportuj G-code (*File → Export*) albo użyj [Symulatora GRBL](Symulator-GRBL).

> ⚠️ Pierwszy test zawsze przy niskiej mocy na odpadzie. Zobacz [Bezpieczeństwo i ograniczenia](Bezpieczenstwo-i-ograniczenia).

Wbudowany **Przewodnik (F1)** opisuje każdą opcję i ma wyszukiwarkę (angielski i polski).
