🌐 [English](GRBL-Simulator) · **Polski**

# Symulator GRBL

[grbl-sim](https://github.com/grbl/grbl-sim) kompiluje prawdziwy firmware GRBL 1.1h do programu na PC, który mówi protokołem szeregowym GRBL, więc strumieniowanie, status, wstrzymanie posuwu i reset można testować bez lasera.

**Nie jest objęte:** łącze USB-serial, wyjście mocy lasera, krańcówki i bazowanie, taktowanie ATmega328P. Pozytywny wynik to dobry znak, nie zastępuje testu przy niskiej mocy na maszynie.

## Budowanie
Linux lub macOS z `git`, `make` i kompilatorem C:
```sh
tools/grbl-sim/build.sh
```
Skrypt pobiera GRBL `v1.1h.20190825` i grbl-sim do `target/grbl-sim/` oraz nakłada dwie łatki (`grbl-sim-fixes.patch`, `grbl-fixes.patch`) naprawiające problemy występujące tylko na PC: opóźnione odpowiedzi `ok`, wywołania systemowe przy każdym odpytaniu, zawieszające się silniki przez wspólną flagę przerwań, osierocone symulatory i gubione żądania resetu. Nie zmieniają zachowania GRBL na prawdziwym sterowniku. `GRBL_REF` i `SIM_REF` wybierają inne wersje.

## Użycie z aplikacji
```sh
python3 tools/grbl-sim/serve.py        # nasłuchuje na 127.0.0.1:3333
```
W *Laser → Device settings* wybierz sterownik **GRBL**, połączenie **TCP**, host `127.0.0.1`, port `3333` i połącz. Każde połączenie uruchamia świeży symulator z ustawieniami domyślnymi. Dla szybszych przebiegów wyślij w konsoli: `$110=30000`, `$111=30000`, `$120=5000`, `$121=5000`, `$32=1`. `-v` zachowuje ślady kroków i bloków.

Wirtualny port szeregowy w Linuksie:
```sh
socat PTY,raw,link=/tmp/ttyGRBL,echo=0 "EXEC:'target/grbl-sim/grbl/grbl/sim/grbl_sim.exe -n -s /dev/null -b /dev/null',pty,raw,echo=0"
```

## Testy automatyczne
`crates/lightcreator/src/grbl_sim_tests.rs` steruje prawdziwym wątkiem portu szeregowego wobec symulatora:

| Test | Sprawdza |
|---|---|
| `grbl_sim_keeps_up_with_real_time` | symulator działa w czasie rzeczywistym |
| `grbl_sim_streams_a_generated_job` | wygenerowane zadanie w pełni potwierdzone, kończy bezczynnie w zerze |
| `grbl_sim_long_job_keeps_character_counting_in_sync` | gęste wypełnienie z zawsze pełnym oknem 120 bajtów |
| `grbl_sim_rejected_line_does_not_desync_the_stream` | `error:20` nie psuje reszty zadania |
| `grbl_sim_feed_hold_and_resume` | `!` wstrzymuje, `~` wznawia |
| `grbl_sim_abort_then_new_job_runs` | miękki reset w trakcie, potem nowe zadanie działa |
| `grbl_sim_job_sent_right_after_connect_is_not_lost` | zadanie wysłane podczas restartu GRBL po połączeniu nie ginie |

```sh
tools/grbl-sim/build.sh
cargo test -p lightcreator grbl_sim -- --ignored --test-threads=1
```
`LC_GRBL_SIM_EXE` wskazuje symulator zbudowany gdzie indziej. Workflow *Tests* uruchamia je przy każdym pull requeście. Dla grblHAL jego [Simulator](https://github.com/grblHAL/Simulator) powinien działać z `serve.py --sim` (niesprawdzone). Symulator zajmuje około dwóch rdzeni procesora.
