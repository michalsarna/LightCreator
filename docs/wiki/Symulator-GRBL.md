🌐 [English](GRBL-Simulator) · **Polski**

# Symulator GRBL

[grbl-sim](https://github.com/grbl/grbl-sim) kompiluje prawdziwy firmware GRBL 1.1h do programu na PC, który mówi protokołem szeregowym GRBL, więc strumieniowanie, status, wstrzymanie posuwu i reset można testować bez lasera.

**Nie jest objęte:** łącze USB-serial, wyjście mocy lasera, krańcówki i bazowanie, taktowanie ATmega328P. Pozytywny wynik to dobry znak, nie zastępuje testu przy niskiej mocy na maszynie.

## Budowanie
Linux lub macOS z `git`, `make` i kompilatorem C:
```sh
tools/grbl-sim/build.sh
```
Symulator jest w [baseCreator](https://github.com/michalsarna/baseCreator) (`tools/grbl-sim/`, wspólny dla LightCreator, MillCreator i PenCreator). Skrypt pobiera GRBL `v1.1h.20190825` i grbl-sim do `target/grbl-sim/` oraz nakłada łatki (`grbl-sim-fixes.patch`, `grbl-fixes.patch`, a na macOS `grbl-sim-macos.patch`) naprawiające problemy występujące tylko na PC: opóźnione odpowiedzi `ok`, wywołania systemowe przy każdym odpytaniu, zawieszające się silniki przez wspólną flagę przerwań, osierocone symulatory, gubione żądania resetu, bufor odbiorczy gubiący bajty (zepsute lub sklejone linie) oraz zegar, przez który symulator działał na macOS dużo za wolno. Nie zmieniają zachowania GRBL na prawdziwym sterowniku. `GRBL_REF` i `SIM_REF` wybierają inne wersje.

## Użycie z aplikacji
```sh
python3 tools/grbl-sim/serve.py        # nasłuchuje na 127.0.0.1:3333
```
W *Machine → Device settings* wybierz sterownik **GRBL**, połączenie **TCP**, host `127.0.0.1`, port `3333` i połącz. Każde połączenie uruchamia świeży symulator z ustawieniami domyślnymi. Dla szybszych przebiegów wyślij w konsoli: `$110=30000`, `$111=30000`, `$120=5000`, `$121=5000`, `$32=1`. `-v` zachowuje ślady kroków i bloków.

Wirtualny port szeregowy w Linuksie:
```sh
socat PTY,raw,link=/tmp/ttyGRBL,echo=0 "EXEC:'target/grbl-sim/grbl/grbl/sim/grbl_sim.exe -n -s /dev/null -b /dev/null',pty,raw,echo=0"
```

## Testy automatyczne
`crates/lightcreator/src/grbl_sim_tests.rs` steruje prawdziwym wątkiem portu szeregowego wobec symulatora:

Łącze szeregowe jest testowane w baseCreator (`base_ui::sim_test`: czas rzeczywisty, długie zadanie z liczeniem znaków, odrzucona linia, wstrzymanie posuwu, przerwanie, wyścig przy połączeniu, pauza `M0`, kopia i przywracanie ustawień, wrzeciono i łuki). Własne testy LightCreator (`crates/lightcreator/src/grbl_sim_tests.rs`) uruchamiają to, co program generuje:

| Test | Sprawdza |
|---|---|
| `grbl_sim_streams_a_generated_job` | wygenerowane zadanie w pełni potwierdzone, kończy bezczynnie |
| `grbl_sim_runs_a_job_with_arcs_and_ends_where_expected` | łuki, koniec w zerze |
| `grbl_sim_starts_with_laser_mode_off_and_accepts_enabling_it` | odczyt i zapis `$32` |
| `grbl_sim_focus_beam_switches_on_with_m3_and_off_with_m5` | wiązka ostrości |
| `grbl_sim_pass_by_pass_and_object_by_object_both_run` | obie kolejności przejść |

```sh
# w baseCreator: tools/grbl-sim/build.sh (do target/grbl-sim tam) albo GRBL_SIM_EXE wskazuje gotowy symulator
cargo test -p lightcreator grbl_sim -- --ignored --test-threads=1
```
`GRBL_SIM_EXE` wskazuje symulator zbudowany gdzie indziej. Workflow *Tests* uruchamia je przy każdym pull requeście. Dla grblHAL jego [Simulator](https://github.com/grblHAL/Simulator) powinien działać z `serve.py --sim` (niesprawdzone). Symulator zajmuje około dwóch rdzeni procesora.
