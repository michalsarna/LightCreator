🌐 [English](Architecture) · [Polski](Architektura) · [简体中文](Architecture-ZH) · **한국어**

> ⚠️ 이 페이지는 영어판을 기계 번역한 것으로, 부정확한 부분이 있을 수 있습니다. 의심스러우면 [영어판](Architecture)을 기준으로 하십시오. 수정 제안을 환영합니다.

# 아키텍처

저장소는 두 개의 크레이트로 된 Cargo 워크스페이스입니다.

| 크레이트 | 용도 |
|---|---|
| `crates/lc-core` | UI와 독립적이며 단위 테스트가 있는 라이브러리 |
| `crates/lightcreator` | [egui](https://github.com/emilk/egui) 데스크톱 애플리케이션 |

## baseCreator (MillCreator와 공유)
레이저가 필요 없는 코드는 [baseCreator](https://github.com/michalsarna/baseCreator)에 있으며, `crates/lc-core/Cargo.toml`과 `crates/lightcreator/Cargo.toml`에서 태그로 고정됩니다.
* **base-core**: `geom`, `bezier`, `arcs`, `fillet`, `snap` (형상, 윤곽, 모서리 둥글리기) · `ops` (불리언 연산, 오프셋) · `text` (윤곽) · `image`, `trace`, `camimg` · `scene` (도형, 그룹, ID) · `svg`, `pdf` (읽기 기능) · `clipart` (라이브러리) · `units` · `grbl_settings`, `ser2net`.
* **base-ui**: `theme`, `fonts`, `icons`, `i18n` / `texts` · `menu`, `native_menu` (항목, 창 내부 및 macOS 메뉴 바, 표준 편집 명령) · `window` (테두리, 팝업, 버튼), `widgets` · `dialogs` (정보, 빠른 안내서, 업데이트 확인) · `clipart_ui` · `editor` (캔버스, 도구, 선택, 노드 편집, 편집 명령, 그림 대화 상자, 보기 옵션, 도구 팔레트, 배치 바, 속성 섹션, `EditorHost` 트레이트 뒤에 있는 제어 및 상태 바의 구성 요소) · `start` (시작 화면 머리글, 언어 및 색 구성표 줄, 창 아이콘) · `doc_tabs`, `work_panes` (열린 도면의 탭, 탭의 창 영역, 탭을 꺼내 여는 창) · `gcode_text_ui` (편집 가능한 텍스트로서의 프로젝트 G-code), `gcode_view_ui` (G-code 파일 분석) · `layer_bar`, `control_ui` (제어 탭), `job_ui` (줄부터 시작), `tools_ui` (보정) · `markdown_ui`, `units_ui`, `camera`, `camera_stream`, `camera_feed`, `camera_rec` (사진, 동영상, 타임랩스), `ffmpeg`.

`lc-core`는 base-core의 모듈을 다시 내보내므로(re-export) `lc_core::geom` 등이 계속 동작합니다.

## lc-core
`doc` (문서의 레이저 측면: 레이어, 장치, `.cre` 프로젝트의 laser 섹션; 장면과 프로젝트 컨테이너는 base-core에서 옴) · `gcode`, `export` (G-code, PLT / DXF) · `raster`, `cutopts` · `svg`, `pdf`, `clipart` (가져온 것을 레이어에 배치) · `materials`, `profiles`, `zones`, `check`, `hooks` · `overlay_data` (프로젝트 파일의 오버레이 그림), `print` · `controller`.

## lightcreator
`app` (주 상태, `EditorHost`와 `MenuHost` 구현) · `canvas` (편집기의 훅으로 구현된 베드 표시, 구역, 실시간 위치, 미리 보기), `shape_ops`, `overlay` (도면 아래의 공유 그림이 레이저 쪽에서 필요로 하는 것: 장치와 함께 보관되는 모서리, USB 카메라) · `menu` (동작 목록), `icons` (각 동작의 아이콘), `native_menu`, `panels`, `i18n` (텍스트), `guide`, `help_ui`, `update_ui`, `prefs`, `window_ui` · `laser` (GRBL과 Marlin용 시리얼 / TCP 작업자) · `device_ui`, `ser2net_ui`, `stream_ui` (장치, 카메라) · `clipart_ui` (프로그램과 함께 제공되는 그림), `materials_ui`, `preview_ui`, `print_ui`, `check_ui` · `panes_glue` (작업 영역의 탭), `control_glue` (제어 탭과 작업 영역 위의 바), `workspaces` (열린 도면) · `screenshots` (헤드리스 렌더링).

`build.rs`는 클립아트를 내장합니다. 인터페이스 아이콘은 baseCreator가 내장하며(`assets/ui-icons`), 테스트가 동작이 요구하는 모든 아이콘이 존재하는지 확인합니다.

## 규칙
* 좌표는 **밀리미터**이며, 원점은 작업 영역의 **왼쪽 위**입니다. 출력 시 원점이 앞 왼쪽에 있는 기계에서는 Y가 뒤집힙니다(**기계 → 장치 설정…**(Machine → Device settings…)).
* 저장되는 형상은 항상 mm이며, 표시 단위는 장치별로 정합니다.

## 테스트와 CI
* `cargo test --workspace`는 형상, 불리언, 오프셋, 래스터, G-code, 내보내기, 번역 로직을 다룹니다.
* 스크린숏은 실제 UI에서 헤드리스로 렌더링됩니다: `cargo test -p lightcreator --release render_screenshots -- --ignored`.
* GRBL 스트리밍은 [GRBL 시뮬레이터](GRBL-Simulator-KO)에서 처음부터 끝까지 테스트됩니다.
* GitHub 워크플로: `tests.yml`, `release.yml` (태그 `v` + `Cargo.toml` 버전으로 Windows / macOS / Linux 패키지를 빌드), `codeql.yml`.

## 기여
이슈와 풀 리퀘스트를 환영합니다. 도움이 되는 기여: 버그 보고, 실제 기계에서의 테스트, 번역, 클립아트, 재료. [지원](Support-KO)을 참고하십시오.
