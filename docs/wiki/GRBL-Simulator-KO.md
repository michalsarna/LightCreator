🌐 [English](GRBL-Simulator) · [Polski](Symulator-GRBL) · [简体中文](GRBL-Simulator-ZH) · **한국어**

> ⚠️ 이 페이지는 영어판을 기계 번역한 것으로, 부정확한 부분이 있을 수 있습니다. 의심스러우면 [영어판](GRBL-Simulator)을 기준으로 하십시오. 수정 제안을 환영합니다.

# GRBL 시뮬레이터

[grbl-sim](https://github.com/grbl/grbl-sim)은 실제 GRBL 1.1h 펌웨어를 GRBL 시리얼 프로토콜로 통신하는 PC 프로그램으로 컴파일하므로, 레이저 없이 스트리밍, 상태, 이송 일시 정지, 리셋을 테스트할 수 있습니다.

**다루지 않는 것:** USB-시리얼 연결, 레이저 출력, 리밋 스위치와 원점 복귀, ATmega328P 타이밍. 여기서 통과하는 것은 좋은 신호일 뿐, 기계에서의 저출력 테스트를 대신하지는 않습니다.

## 빌드
시뮬레이터는 [baseCreator](https://github.com/michalsarna/baseCreator)에 있습니다(`tools/grbl-sim/`, LightCreator, MillCreator, PenCreator가 공유). `git`, `make`, C 컴파일러가 있는 Linux 또는 macOS가 필요합니다.
```sh
tools/grbl-sim/build.sh
```
이 스크립트는 GRBL `v1.1h.20190825`와 grbl-sim을 `target/grbl-sim/`으로 가져오고, PC에서만 생기는 문제를 고치는 패치(`grbl-sim-fixes.patch`, `grbl-fixes.patch`, macOS에서는 `grbl-sim-macos.patch`)를 적용합니다: 지연된 `ok` 응답, 폴링마다의 시스템 호출, 공유 인터럽트 플래그로 인한 스테퍼 정지, 고아가 된 시뮬레이터, 유실된 리셋 요청, 바이트를 잃는 수신 링 버퍼(깨지거나 붙은 줄), 그리고 macOS에서 시뮬레이터를 지나치게 느리게 만든 클록. 이 패치들은 실제 컨트롤러에서의 GRBL 동작을 바꾸지 않습니다. `GRBL_REF`와 `SIM_REF`로 다른 버전을 선택할 수 있습니다.

## 앱에서 사용하기
```sh
python3 tools/grbl-sim/serve.py        # listens on 127.0.0.1:3333
```
**기계 → 장치 설정…**(Machine → Device settings…)에서 컨트롤러 **GRBL**을 고르고, **연결**(Connection) 페이지에서 연결 유형 **네트워크 (TCP, 예: ser2net)**(Network (TCP, e.g. ser2net))를 호스트 `127.0.0.1`, TCP 포트 `3333`으로 설정한 다음 **장치**(Device) 탭에서 **연결**(Connect)을 누릅니다. 연결할 때마다 기본 설정의 새 시뮬레이터가 시작됩니다. 더 빠르게 실행하려면 콘솔에서 `$110=30000`, `$111=30000`, `$120=5000`, `$121=5000`, `$32=1`을 보내십시오. `-v`는 스텝 / 블록 추적을 남깁니다.

Linux의 가상 시리얼 포트:
```sh
socat PTY,raw,link=/tmp/ttyGRBL,echo=0 "EXEC:'target/grbl-sim/grbl/grbl/sim/grbl_sim.exe -n -s /dev/null -b /dev/null',pty,raw,echo=0"
```

## 자동 테스트
시리얼 연결은 baseCreator에서 테스트됩니다(`base_ui::sim_test`: 실시간 확인, 문자 수 계산을 사용하는 긴 작업, 거부된 줄, 이송 일시 정지, 중단, 연결 경쟁 상태, `M0` 일시 정지, 설정 백업/복원, 스핀들과 호). LightCreator 자체 테스트(`crates/lightcreator/src/grbl_sim_tests.rs`)는 LightCreator가 생성한 것을 실행합니다.

| 테스트 | 확인 내용 |
|---|---|
| `grbl_sim_streams_a_generated_job` | 생성된 작업이 모두 확인 응답을 받고 유휴 상태로 끝남 |
| `grbl_sim_runs_a_job_with_arcs_and_ends_where_expected` | 호, 원점에서 끝남 |
| `grbl_sim_starts_with_laser_mode_off_and_accepts_enabling_it` | `$32` 읽기와 쓰기 |
| `grbl_sim_focus_beam_switches_on_with_m3_and_off_with_m5` | 초점 빔 |
| `grbl_sim_pass_by_pass_and_object_by_object_both_run` | 패스의 두 가지 순서 |

```sh
# in baseCreator: tools/grbl-sim/build.sh (into target/grbl-sim there), or point GRBL_SIM_EXE at a build
cargo test -p lightcreator grbl_sim -- --ignored --test-threads=1
```
`GRBL_SIM_EXE`는 다른 곳에서 빌드한 시뮬레이터를 가리킵니다. *Tests* 워크플로가 모든 풀 리퀘스트에서 이 테스트를 실행합니다. grblHAL의 경우 그 [Simulator](https://github.com/grblHAL/Simulator)가 `serve.py --sim`과 함께 동작할 것입니다(시험하지 않음). 시뮬레이터는 CPU 코어 약 두 개를 사용합니다.
