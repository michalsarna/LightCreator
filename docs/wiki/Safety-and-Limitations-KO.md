🌐 [English](Safety-and-Limitations) · [Polski](Bezpieczenstwo-i-ograniczenia) · [简体中文](Safety-and-Limitations-ZH) · **한국어**

> ⚠️ 이 페이지는 영어판을 기계 번역한 것으로, 부정확한 부분이 있을 수 있습니다. 의심스러우면 [영어판](Safety-and-Limitations)을 기준으로 하십시오. 수정 제안을 환영합니다.

# 안전과 제한 사항

> LightCreator가 동작하는 것으로 확인된 기계와 사용 중인 기계를 추가하는 방법: [TESTS_ON_DEVICES.md](https://github.com/michalsarna/LightCreator/blob/main/TESTS_ON_DEVICES.md).

> ⚠️ **레이저는 위험합니다.** G-code는 낮은 출력으로 테스트하고, 보안경을 착용하며, 동작 중인 기계를 절대 방치하지 말고, 기계의 `$30` (S 최대) 및 `$32` (레이저 모드) 설정을 확인하십시오.

## 검증된 것
모든 형상, 불리언, 오프셋, 래스터, G-code, 내보내기, 번역 로직은 단위 테스트로 검증되며, 스크린숏은 실제 UI에서 렌더링됩니다. GRBL 스트리밍은 [시뮬레이터](GRBL-Simulator-KO)에서 실제 GRBL 1.1h 펌웨어를 상대로 처음부터 끝까지 시험됩니다.

## 검증되지 않은 것
**LightCreator는 Sculpfun S9 (GRBL 기반 다이오드 레이저)에서 테스트되었으며 정상적으로 동작합니다.** 실제 기계에서 아직 테스트되지 않은 것: Marlin 스트리밍, Z축과 로터리 부착 장치(척과 롤러의 모델은 가정임), 훅과 매크로, 금지 구역, 여러 레이저 헤드(`T<n>`과 S 환산), 초점 빔, 포인터 정렬과 시험 실행, 단순 스트리밍 모드와 멈춤 보고, USB 바인딩과 재연결, 연결 후 원점 복귀 / 잠금 해제, 펌웨어 ($$) 페이지(기록, 백업, 복원, 공장 초기화; 단일 축 원점 복귀 옵션은 `$I`에서 감지), Smoothieware 방언, 호(G2 / G3), 사용자 시작 / 끝 G-code, 레이어별 출력 모드, 네 모서리 원점, 축 반전과 회전된 패널, 소프트 리밋과 작업 검사, 연결 후 경고($32 / $30), Ruida / Trocen 경로, 종이 프린터 인쇄(`lp`, Windows 인쇄 동작), 실시간 카메라(여러 카메라, 화면 설정, 렌즈 보정, 기준점 정렬), 유지 보수 카운터, 장치에서 설정 읽기, Illustrator 가져오기(PDF 호환 파일만). 먼저 자투리 재료로 테스트하십시오.

## 웹 서버와 API
켜면(프로그램 옵션(Program options)) 웹 서버와 API는 접근 문구(기본값 `LightCreator`, 바꾸십시오)가 필요하지만 **암호화는 없습니다**. 기본적으로 이 컴퓨터에서만 수신하며, *네트워크의 다른 컴퓨터에서 접근 가능*(Reachable from other computers on the network)을 켜면 문구를 아는 네트워크의 누구나 기계를 보고 정지시키거나 움직일 수 있습니다. 신뢰하는 네트워크에서만 사용하십시오.

## 아직 구현되지 않은 것
Ruida / Trocen 직접 바이너리 프로토콜, 텍스트의 커닝과 복잡한 문자의 글자 모양 조합(데바나가리, 아랍 문자), 카메라의 자동 렌즈 보정(보정 값은 직접 입력).

## 보안
취약점은 [GitHub security advisories](https://github.com/michalsarna/LightCreator/security/advisories/new)를 통해 비공개로 보고하십시오. 최신 릴리스만 지원됩니다.
