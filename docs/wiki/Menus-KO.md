🌐 [English](Menus) · [Polski](Menu) · [简体中文](Menus-ZH) · **한국어**

> ⚠️ 이 페이지는 영어판을 기계 번역한 것으로, 부정확한 부분이 있을 수 있습니다. 의심스러우면 [영어판](Menus)을 기준으로 하십시오. 수정 제안을 환영합니다.

# 메뉴

모든 메뉴의 모든 항목을 프로그램에 표시되는 순서대로, 단축키와 기능에 대한 짧은 설명과 함께 정리했습니다. 링크는 자세히 설명하는 페이지로 이어집니다. LightCreator, MillCreator, PenCreator는 하나의 메뉴 구성을 공유합니다.

**파일(File) · 편집(Edit) · 삽입(Insert) · 배치(Arrange) · 보기(View) · 기계(Machine) · 레이저(Laser) · 설정(Settings) · 도움말(Help)**

macOS에서는 **Ctrl**을 **Cmd**로 읽으십시오. 단축키는 [키보드 단축키](Keyboard-Shortcuts-KO)에 한곳에 정리되어 있습니다.

**플랫폼별 차이**

* **Windows와 Linux:** 메뉴 바가 창 안에 그려집니다. **종료**(Quit)는 **파일**의 마지막 항목이고 **정보**(About)는 **도움말**의 마지막 항목입니다.
* **macOS:** 메뉴가 시스템 메뉴 바에 있습니다. **파일** 왼쪽에 **LightCreator** 메뉴가 있으며, 여기에 **LightCreator 정보**(About LightCreator), 서비스(Services), 가리기(Hide), 기타 가리기(Hide Others), 모두 보기(Show All), **종료**(Quit) (Cmd+Q)가 있습니다. **파일**에는 종료가 없고 **도움말**에는 정보가 없습니다. **도움말** 다음에는 최소화(Minimize)와 최대화(Maximize)가 있는 **윈도우**(Window) 메뉴가 옵니다. 시스템 메뉴는 **새로 만들기**(New), **열기…**(Open…), **저장**(Save), **다른 이름으로 저장…**(Save as…), **SVG 가져오기…**(Import SVG…) 옆에만 단축키를 표시하지만, 아래 표의 모든 단축키는 macOS에서도(Cmd와 함께) 동작합니다.
* **Linux 전용:** **기계 → 네트워크로 공유 (ser2net)…**(Machine → Share over network (ser2net)…).
* **시작 창에서는** 다음 항목만 동작합니다: **새로 만들기**, **열기…**, **최근 파일 열기**(Open recent), **G-code 파일 보기…**(View G-code file…), **종료**, **정보**, **언어**(Language), **색 구성표**(Colour scheme), **백업에서 복원…**(Restore from backup…), **업데이트 확인…**(Check for updates…). 나머지는 프로젝트가 열려야 사용할 수 있습니다.

현재 사용할 수 없는 항목은 회색으로 표시됩니다(예: 되돌릴 것이 없을 때의 **실행 취소**(Undo), 연결이 없을 때의 **프레임**(Frame)). 스위치(**격자**(Grid), **툴패스 미리 보기**(Toolpath preview)…)는 켜져 있는 동안 체크 표시가 나타납니다.

## 파일 (File)

| 항목 | 단축키 | 기능 |
|---|---|---|
| **새로 만들기** (New) | Ctrl+N | 새 탭에 비어 있는 새 도면을 만듭니다(최대 10개 탭). 시작 창에서는 선택한 장치(장치가 없으면 Dummy)로 프로젝트를 시작합니다. [설정과 파일](Settings-and-Files-KO)을 참고하십시오. |
| **열기…** (Open…) | Ctrl+O | 프로젝트(`.cre`)를 별도의 탭에서 엽니다(빈 탭은 그대로 사용하고, 이미 열려 있는 프로젝트는 표시만 합니다). 여기서 선택한 SVG나 DXF 파일은 열려 있는 도면에 추가됩니다. |
| **최근 파일 열기** (Open recent) ▸ | | 마지막으로 열거나 저장한 프로젝트와 파일 10개입니다. **목록 지우기**(Clear list)로 비웁니다. 더 이상 존재하지 않는 파일은 목록에서 빠집니다. |
| **저장** (Save) | Ctrl+S | 프로젝트를 저장합니다. 파일 이름이 없는 프로젝트는 이름을 묻습니다. |
| **다른 이름으로 저장…** (Save as…) | Ctrl+Shift+S | 프로젝트를 새 이름으로 저장합니다. |
| **프로젝트 정보…** (Project information…) | | 파일(이름, 폴더, 크기, 생성 및 변경 시각), 프로젝트가 만들어진 시각, 작성자, 기여자, 편집 기록을 보여 줍니다. 직접 저장할 때마다 **설정 → 프로그램 옵션 → 작성자**(Settings → Program options → Author)에 설정된 사람(이름, 이메일, 기타; 모든 프로그램에서 동일), 시각, 프로그램과 그 버전을 최대 1분에 한 번 기록합니다. 만든 사람은 세 가지를 모두, 기여자는 이름과 이메일만(이메일이 만든 사람과 같으면 이름만) 저장합니다. |
| **가져오기** (Import) ▸ **이미지 가져오기…** (Import image…) | | 비트맵(PNG, JPEG, BMP, GIF, WebP)을 조정 대화 상자와 함께 페이지에 놓습니다. [디자인 도구](Design-Tools-KO#가져오기)를 참고하십시오. |
| **가져오기** ▸ **SVG 가져오기…** (Import SVG…) | Ctrl+I | SVG 파일의 도형을 도면에 추가합니다. 색상은 레이어에 대응됩니다. 파일 안의 그림도 활성 레이어에 함께 들어옵니다. |
| **가져오기** ▸ **Adobe Illustrator / PDF / EPS / VectorCraft 가져오기…** (Import Adobe Illustrator / PDF / EPS / VectorCraft…) | | PDF 또는 PDF 호환 `.ai` 파일의 첫 페이지, 오래된 PostScript 방식의 `.ai`, EPS의 벡터 경로를 가져옵니다. VectorCraft 도면(`.vectorcraft`, 또는 VectorCraft 0.8 이상에서 작성한 EPS)은 그림과 함께 가져옵니다. |
| **가져오기** ▸ **DXF 가져오기…** (Import DXF…) | | 현재 방식 또는 R12(*legacy*) ASCII DXF의 선, 호, 원, 폴리라인, 스플라인, 블록을 가져옵니다. |
| **가져오기** ▸ **LightBurn / MillMage 프로젝트 가져오기…** (Import LightBurn / MillMage project…) | | LightBurn의 `.lbrn2` / `.lbrn` 파일 또는 MillMage의 `.mage` 파일에서 도형과 레이어 설정을 가져옵니다. |
| **가져오기** ▸ **UGS 디자인 가져오기…** (Import UGS design…) | | UGS Designer 파일(`.ugsd`)의 도형을 가져옵니다. |
| **가져오기** ▸ **플러그인으로 가져오기…** (Import with a plugin…) | | 플러그인의 가져오기 기능으로 파일을 읽습니다. [플러그인](Plugins-KO)을 참고하십시오. |
| **가져오기** ▸ **도형 생성…** (Generate shape…) | | 플러그인의 도형 생성기 양식입니다(같은 항목이 **삽입**에도 있습니다). |
| **내보내기** (Export) ▸ **G-code 내보내기…** (Export G-code…) | | 작업을 검사하고, 켜 둔 포스트프로세서를 거쳐 G-code를 파일에 기록합니다. **G-code 저장**(Save G-code) 버튼 및 **레이저 → G-code 내보내기…**(Laser → Export G-code…)와 같습니다. |
| **내보내기** ▸ **SVG 내보내기…** (Export SVG…) | | 도면을 작업 영역 크기의 SVG로 곡선과 그림을 포함해 기록합니다. |
| **내보내기** ▸ **PLT / DXF 내보내기…** (Export PLT / DXF…) | | 작업을 검사하고 Ruida 및 Trocen 컨트롤러 소프트웨어용 HPGL(`.plt`) 또는 DXF를 기록합니다. [장치와 컨트롤러](Devices-and-Controllers-KO#컨트롤러)를 참고하십시오. |
| **내보내기** ▸ **DXF 내보내기…** (Export DXF…) | | 도면 전체를 현재 방식(AutoCAD 2007)의 DXF로 기록합니다. 곡선은 0.01 mm로 평탄화됩니다. |
| **내보내기** ▸ **DXF R12 내보내기 (legacy)…** (Export DXF R12 (legacy)…) | | 같은 내용을 오래된 프로그램과 커터를 위한 DXF R12(*legacy*라고도 함)로 기록합니다. |
| **내보내기** ▸ **Adobe Illustrator 내보내기…** (Export Adobe Illustrator…) | | 윤곽을 곡선을 포함한 PostScript 방식의 Illustrator 파일로 기록하며, 프로젝트의 레이어마다 Illustrator 레이어를 하나씩 만듭니다. 그림은 제외됩니다. |
| **내보내기** ▸ **LightBurn 프로젝트 내보내기…** (Export LightBurn project…) | | LightBurn 프로젝트(`.lbrn2`, 또는 파일 이름을 그렇게 지으면 `.lbrn`)로, 곡선을 포함한 도형과 그림을 기록하며 프로젝트의 레이어마다 LightBurn 레이어를 하나씩 만듭니다. |
| **내보내기** ▸ **MillMage 프로젝트 내보내기…** (Export MillMage project…) | | 같은 내용을 MillMage 프로젝트(`.mage`)로 기록하며, 재료는 작업 영역 크기입니다. |
| **내보내기** ▸ **선택 항목을 SVG로 내보내기…** (Export selection as SVG…) / **선택 항목을 DXF로 내보내기…** (Export selection as DXF…) | | 선택한 도형을 기록합니다(아무것도 선택하지 않으면 전부). |
| **G-code 파일 보기…** (View G-code file…) | | G-code 파일을 G-code 분석(툴패스가 표시되는 탭)으로 엽니다. |
| **인쇄…** (Print…) | Ctrl+P | 디자인을 종이 프린터로 인쇄하거나 PDF로 저장합니다. [설정과 파일](Settings-and-Files-KO#인쇄)을 참고하십시오. |
| **종료** (Quit) | Ctrl+Q | 프로그램을 닫습니다. 저장하지 않은 작업은 탭마다 묻습니다. Windows와 Linux 전용입니다(macOS에서는 LightCreator 메뉴에 있습니다). |

## 편집 (Edit)

| 항목 | 단축키 | 기능 |
|---|---|---|
| **실행 취소** (Undo) | Ctrl+Z | 마지막 변경을 되돌립니다. |
| **다시 실행** (Redo) | Ctrl+Y (Ctrl+Shift+Z도 가능) | 되돌린 작업을 다시 실행합니다. |
| **복사** (Copy) | Ctrl+C | 선택 항목을 복사합니다. |
| **붙여넣기** (Paste) | Ctrl+V | 복사한 내용을 붙여 넣습니다. |
| **복제** (Duplicate) | Ctrl+D | 선택 항목의 사본을 즉시 만듭니다. |
| **삭제** (Delete) | Delete 또는 Backspace | 선택 항목을 삭제합니다(노드 편집 중에는 선택한 노드). 잠긴 객체는 삭제되지 않습니다. |
| **모두 선택** (Select all) | Ctrl+A | 모든 객체를 선택합니다. |

## 삽입 (Insert)

왼쪽 팔레트에 있는, 페이지에 새 객체를 놓는 도구들입니다. 사용 중인 도구에는 체크 표시가 됩니다. 하나를 선택하는 것은 해당 버튼을 클릭하거나 단축키를 누르는 것과 같습니다. [디자인 도구](Design-Tools-KO#그리기)를 참고하십시오.

| 항목 | 단축키 | 기능 |
|---|---|---|
| **사각형 (R)** (Rectangle (R)) | R | 드래그하여 사각형을 그립니다(Shift: 정사각형). |
| **타원 (E)** (Ellipse (E)) | E | 드래그하여 타원을 그립니다(Shift: 원). |
| **삼각형 (Y)** (Triangle (Y)) | Y | 드래그하여 삼각형을 그립니다. |
| **별 (S)** (Star (S)) | S | 드래그하여 오각별을 그립니다. |
| **다각형 (G)** (Polygon (G)) | G | 드래그하여 정다각형을 그립니다. 작은 창에서 변의 개수를 설정합니다. |
| **하트 (K)** (Heart (K)) | K | 드래그하여 하트를 그립니다. |
| **나선 (I)** (Spiral (I)) | I | 드래그하여 나선을 그립니다. 작은 창에서 회전 수를 설정합니다. |
| **선 (L)** (Line (L)) | L | 드래그하여 직선을 그립니다. |
| **폴리라인 / 펜 (P)** (Polyline / pen (P)) | P | 점을 하나씩 클릭합니다. 오른쪽 클릭, **Enter** 또는 더블 클릭으로 선을 끝냅니다. |
| **텍스트 (T)** (Text (T)) | T | 클릭하여 텍스트를 놓습니다. |
| **도형 생성…** (Generate shape…) | | 플러그인의 도형 생성기 양식입니다. [플러그인](Plugins-KO)을 참고하십시오. |
| **온라인 클립아트…** (Online clipart…) | | 온라인 아이콘 모음 검색이 있는 **클립아트**(Clipart) 탭을 엽니다. [클립아트 갤러리](Clipart-Gallery-KO)를 참고하십시오. |

## 배치 (Arrange)

명령은 선택 항목에 적용됩니다. 대부분은 작업 영역의 오른쪽 클릭 메뉴와 그 옆의 배치 바에도 있습니다. [디자인 도구](Design-Tools-KO)를 참고하십시오.

| 항목 | 단축키 | 기능 |
|---|---|---|
| **왼쪽 정렬** (Align left), **가운데 정렬 (H)** (Align centre (H)), **오른쪽 정렬** (Align right), **위쪽 정렬** (Align top), **가운데 정렬 (V)** (Align centre (V)), **아래쪽 정렬** (Align bottom) | | 선택한 객체를 한쪽 가장자리 또는 가운데에 맞춰 정렬합니다. |
| **베드 가운데에 놓기** (Centre on bed) | | 선택 항목을 작업 영역 가운데로 옮깁니다. |
| **서로 중심 맞추기** (Centre on each other) | | 선택한 객체들의 중심을 한 점에 맞춥니다(그룹은 하나의 객체로 셉니다). 두 개 이상이 필요합니다. |
| **좌우 뒤집기** (Flip horizontal) / **상하 뒤집기** (Flip vertical) | | 선택 항목을 대칭 반전합니다. |
| **시계 방향 90° 회전** (Rotate 90° CW) / **반시계 방향 90° 회전** (Rotate 90° CCW) | | 선택 항목을 4분의 1 바퀴 돌립니다. |
| **그룹** (Group) | Ctrl+G | 선택 항목을 하나의 그룹으로 묶습니다. |
| **그룹 해제** (Ungroup) | Ctrl+Shift+G | 그룹을 풉니다. |
| **잠금** (Lock) | Ctrl+L | 잠긴 객체는 이동, 편집, 삭제할 수 없지만 선택과 복사는 할 수 있습니다. |
| **잠금 해제** (Unlock) | Ctrl+Shift+L | 선택 항목의 잠금을 해제합니다. |
| **맨 앞으로 가져오기** (Bring to front) / **맨 뒤로 보내기** (Send to back) | | 객체의 순서를 바꿉니다. |
| **노드 편집** (Edit nodes) | N | 노드 도구로 전환합니다. [디자인 도구](Design-Tools-KO#노드-편집-n)를 참고하십시오. |
| **경로로 변환** (Convert to path) | | 선택한 도형을 직선 구간으로 된 단순 경로로 바꿉니다(회전과 크기가 윤곽에 반영됩니다). |
| **합집합** (Union), **교집합** (Intersection), **빼기** (Subtract), **배타적 논리합** (Exclusive or) | | 선택한 도형에 대한 불리언 연산입니다. |
| **도형 오프셋…** (Offset shape…) | | 윤곽을 키우거나 줄입니다. |
| **모서리 둥글리기…** (Round corners…) | | 뾰족한 모서리를 반지름으로 둥글게 합니다. |
| **곡선으로 변환** (Convert to curves) | | 선택한 도형(텍스트 포함, 그림 제외)을 편집 가능한 베지어 곡선으로 바꿉니다. |
| **경로 자동 연결…** (Auto-join paths…) | | 끝점이 허용 오차보다 가까운 열린 경로들을 연결합니다. |
| **허용 오차로 경로 닫기…** (Close path with tolerance…) | | 두 끝점이 그만큼 가까운 경로를 닫습니다. |
| **도그본 / T-본 모서리…** (Dogbone / T-bone corners…) | | 모든 뾰족한 모서리에 홈을 냅니다(밀링용). |
| **격자 배열…** (Grid array…) | | 선택 항목의 사본을 행과 열로 만듭니다. |
| **원형 배열…** (Circular array…) | | 선택 항목의 중심을 기준으로 둘레에 사본을 만듭니다. |
| **경로를 따라 복사…** (Copy along path…) | | 경로를 따라 사본을 만듭니다(경로를 마지막에 선택). |
| **경로 위의 텍스트…** (Text on path…) | | 텍스트의 윤곽을 경로를 따라 구부립니다(경로를 마지막에 선택). |
| **도형으로 자르기…** (Trim by shape…) | | 마지막에 선택한 닫힌 도형의 안쪽(또는 바깥쪽)에 있는 선 부분만 남깁니다. |
| **중복 제거** (Remove duplicates) | | 같은 레이어에서 윤곽이 같은 도형을 삭제합니다. |
| **분해** (Break apart) | | 윤곽마다 하나의 도형으로 나눕니다. |
| **단순화…** (Simplify…) | | 선에서 허용 오차 이내에 있는 점을 제거합니다. |
| **일련번호 텍스트…** (Serial text…) | | 숫자를 하나씩 늘려 가며 텍스트 사본을 만들거나, CSV 파일의 행마다 사본을 하나씩 만듭니다. |
| **바코드 / QR 코드…** (Barcode / QR code…) | | QR 코드 또는 Code 39 바코드를 윤곽으로 만듭니다. |
| **도형 네스팅…** (Nest shapes…) | | 선택한 도형을 영역 안에 빽빽하게 배치합니다. |
| **측정…** (Measure…) | | 선택 항목의 크기, 위치, 면적, 길이와 두 도형 사이의 거리를 보여 줍니다. |
| **이미지 조정…** (Adjust image…) | | 선택한 그림의 밝기, 대비, 감마 등을 조정합니다. |
| **이미지 트레이스…** (Trace image…) | | 선택한 그림을 윤곽으로 바꿉니다. |
| **이미지 편집…** (Edit image…) | | 선택한 그림의 사진 편집기입니다: 회색조의 브러시, 지우개, 복제 도장, 채우기. |
| **이미지로 변환…** (Convert to image…) | | 선택한 도형(그림 제외)을 해상도와 크기를 지정해 하나의 그림으로 만듭니다. |
| **스타일 트레이스…** (Style trace…) | | 선택한 그림을 21가지 스타일 중 하나의 선으로 다시 그립니다. [스타일 트레이스](Style-Trace-KO)를 참고하십시오. |
| **기하학적 다각형…** (Geometric polygons…) | | 선택한 그림을 다각형 사이와 윤곽 둘레에 간격이 있는 기하학적(로우 폴리) 다각형으로 만듭니다. 해당 스타일로 열린 스타일 트레이스 창입니다. [스타일 트레이스](Style-Trace-KO)를 참고하십시오. |
| **금지 구역으로 추가** (Add as no-go zone) | | 잠기지 않은 선택 도형이 모두 장치의 금지 구역(둥근 도형이면 원, 그 외에는 도형을 둘러싼 사각형)이 되고 도면에서 빠집니다. 오른쪽 클릭 메뉴에도 있습니다. |

## 보기 (View)

| 항목 | 단축키 | 기능 |
|---|---|---|
| **격자** (Grid) | | 격자를 표시하거나 숨깁니다. |
| **격자에 맞춤** (Snap to grid) | | 이동과 그리는 점이 격자에 맞춰집니다. |
| **객체에 맞춤** (Snap to objects) | | 이동하는 선택 항목이 주변 객체의 가장자리와 중심에 맞춰 정렬됩니다. |
| **확대** (Zoom in) / **축소** (Zoom out) | Ctrl++ / Ctrl+- | 보기의 중심을 기준으로 확대·축소합니다. |
| **베드를 창에 맞춤** (Fit bed to window) | Ctrl+Shift+0 | 작업 영역을 창에 맞추고, 직접 확대·축소할 때까지 맞춘 상태를 유지합니다. |
| **모든 객체에 맞춤** (Fit all objects) | Ctrl+Shift+9 | 모든 객체가 보이도록 확대·축소합니다. |
| **작업 영역을 창에 자동 맞춤** (Auto-fit work area to window) | | 스위치: 창 크기가 바뀌는 동안 작업 영역을 맞춘 상태로 유지합니다. |
| **분할 보기: 나란히** (Split view: side by side) | | 작업 영역을 나란히 놓인 창 영역(최대 세 개)으로 나눕니다. 창 영역이 하나뿐이고 탭이 두 개 이상일 때 두 번째 창 영역이 만들어집니다. [디자인 도구](Design-Tools-KO#보기와-탐색)를 참고하십시오. |
| **분할 보기: 위아래로** (Split view: one above the other) | | 같은 기능이며, 창 영역이 위아래로 쌓입니다. |
| **단일 보기** (Single view) | | 모든 탭을 하나의 창 영역으로 되돌립니다. |
| **툴패스 미리 보기** (Toolpath preview) | | 도면 위에 툴패스를 그립니다. **툴패스**(Toolpaths) 버튼과 같습니다. |
| **카메라 보기** (Camera view) | | 장치 카메라의 실시간 화면을 보여 줍니다. 장치 설정에 카메라 주소가 있어야 합니다. [장치와 컨트롤러](Devices-and-Controllers-KO#카메라)를 참고하십시오. |
| **카메라 오버레이 표시** (Show camera overlay) | | 디자인 아래의 오버레이 그림을 표시하거나 숨깁니다. 오버레이가 없으면 회색으로 표시됩니다. |
| **오버레이 가져오기…** (Overlay from…) | | **카메라 오버레이**(Camera overlay) 창을 엽니다: 그림을 불러오거나 카메라를 시작한 다음 정렬합니다. |
| **금지 구역 표시** (Show no-go zones) | | 장치의 금지 구역을 빨간색으로 그립니다. 장치에 금지 구역이 없으면 회색으로 표시됩니다. |
| **미리 보기에 치수 표시** (Dimensions in the preview) | | 미리 보기 위에 작업 크기를 표시하고(미리 보기의 **치수**(Dimensions) 스위치와 같음) 미리 보기를 엽니다. |
| **미리 보기의 원통 (로터리)** (Cylinder in the preview (rotary)) | | 로터리 부착 장치가 있을 때: 미리 보기에서 작업을 공작물 위에 3D로 보여 주거나, (꺼져 있으면) 평면으로 보여 줍니다. 미리 보기의 **원통 (3D)**(Cylinder (3D)) / **평면**(Flat)과 같습니다. |
| **미리 보기** (Preview) | | **미리 보기…**(Preview…) 버튼처럼 미리 보기를 표시하거나 숨깁니다. |
| **G-code 미리 보기** (Preview G-code) | | **G-code 미리 보기** 버튼처럼 작업의 G-code를 탭에 표시합니다. |

## 기계 (Machine)

| 항목 | 단축키 | 기능 |
|---|---|---|
| **장치 전환…** (Switch device…) | | 다른 장치를 선택하기 위해 시작 창으로 돌아갑니다. |
| **장치 설정…** (Device settings…) | | 모든 페이지가 있는 장치 창입니다. [장치와 컨트롤러](Devices-and-Controllers-KO)를 참고하십시오. |
| **펌웨어 설정 ($$)…** (Firmware settings ($$)…) | | 장치 창의 펌웨어 페이지입니다(GRBL 및 Marlin). |
| **보정…** (Calibration…) | | 측정한 이동 거리로 축의 밀리미터당 스텝 수를 계산하고, 측정한 두 대각선으로 축의 직각도를 계산합니다. 새 값을 컨트롤러에 기록할 수 있습니다. 연결이 필요합니다. [장치와 컨트롤러](Devices-and-Controllers-KO#보정)를 참고하십시오. |
| **금지 구역…** (No-go zones…) | | **금지 구역**(No-go zones) 페이지가 열린 장치 창입니다. |
| **제어 장치 ▸** (Control devices ▸) | | 제어 장치 설정…(Control device settings…)은 키보드와 패드, 그리고 그 키의 창을 엽니다. 이 기계의 제어 장치…(Control device of this machine…)는 열린 기계의 제어 장치를 선택합니다. [제어 장치](Control-Devices-KO)를 참고하십시오. |
| **진단 및 통계…** (Diagnostics and statistics…) | | 컨트롤러의 입력, 기계 통계, 유지 보수 카운터, 저장할 수 있는 zip입니다. |
| **네트워크로 공유 (ser2net)…** (Share over network (ser2net)…) | | Linux 전용: ser2net으로 로컬 시리얼 포트를 TCP로 공유합니다. [장치와 컨트롤러](Devices-and-Controllers-KO#tcp와-ser2net)를 참고하십시오. |
| **프레임** (Frame) | | 레이저가 프레임 출력으로 선택한 도형(아무것도 선택하지 않으면 전부)을 둘러싼 상자를 따라갑니다. **제어**(Control) 탭의 **윤곽 프레임**(Outline frame)과 같습니다. 연결이 필요하고 실행 중인 작업이 없어야 합니다. |
| **작업 시작** (Start job) | | **START**와 같습니다: 작업을 검사하고 레이저로 보냅니다(Ruida와 Trocen에서는 대신 PLT / DXF 파일을 내보냅니다). |
| **줄부터 시작…** (Start from line…) | | 선택한 프로그램의 줄부터 작업을 시작합니다. 기계는 먼저 그 줄에서 프로그램이 갖는 상태로 맞춰집니다. 연결이 필요합니다. |
| **프로그램 검사** (Check the program) | | GRBL이 검사 모드(`$C`)로 프로그램을 훑어봅니다: 아무것도 움직이지 않고, 받아들이지 않는 줄을 보고합니다. 연결된 기계가 필요합니다. |

## 레이저 (Laser)

| 항목 | 단축키 | 기능 |
|---|---|---|
| **재료…** (Materials…) | | 장치의 재료 창입니다: 재료, 재료의 레이어 설정, 내보내기와 가져오기. [절단과 레이어](Cuts-and-Layers-KO#재료)를 참고하십시오. |
| **레이어 프리셋…** (Layer presets…) | | 레이어 프리셋 창입니다: 활성 레이어의 설정을 이름을 붙여 저장하거나 프리셋을 레이어에 적용합니다. |
| **G-code 미리 보기** (Preview G-code) | | 작업의 G-code를 저장하지 않고 탭에 텍스트로 표시합니다(**G-code 미리 보기** 버튼과 같음). |
| **미리 보기 창…** (Preview window…) | Ctrl+Shift+P | 작업 미리 보기를 작업 영역의 탭으로 엽니다. [절단과 레이어](Cuts-and-Layers-KO#미리-보기)를 참고하십시오. |
| **G-code 내보내기…** (Export G-code…) | Ctrl+E | **파일 → 내보내기 → G-code 내보내기…** 및 **G-code 저장** 버튼과 같습니다. |

## 설정 (Settings)

| 항목 | 단축키 | 기능 |
|---|---|---|
| **보기 옵션…** (View options…) | | 격자, 작업 영역과 그 주변의 색, 선 두께, 레이어 이름, 변수 이름, 저장 후 G-code 미리 보기. [디자인 도구](Design-Tools-KO#보기와-탐색)를 참고하십시오. **G-code 콘솔**(G-code console)의 글꼴 크기와 색. |
| **프로그램 옵션…** (Program options…) | | 자동 저장, 시작 시 업데이트 확인, 단위, 미리 보기의 **렌더링 품질**(Rendering quality) (1 빠름 … 5 선명함), 웹 서버와 API. [설정과 파일](Settings-and-Files-KO)과 [웹 서버와 API](Web-Server-and-API-KO)를 참고하십시오. |
| **플러그인…** (Plugins…) | | 플러그인을 설치, 전환, 제거합니다. 공식 목록도 있습니다. [플러그인](Plugins-KO)을 참고하십시오. |
| **언어** (Language) ▸ | | 인터페이스의 38개 언어 중 하나를 선택합니다. |
| **색 구성표** (Colour scheme) ▸ | | 색 구성표 중 하나를 선택합니다. |

## 도움말 (Help)

| 항목 | 단축키 | 기능 |
|---|---|---|
| **빠른 안내서** (Quick guide) | F1 | 검색 기능이 있는 프로그램 내 안내서입니다. |
| **GitHub의 프로젝트 페이지** (Project page on GitHub) | | 브라우저에서 프로젝트 페이지를 엽니다. |
| **백업 다운로드…** (Download backup…) | | 설정과 클립아트가 담긴 zip입니다. [설정과 파일](Settings-and-Files-KO)을 참고하십시오. |
| **백업에서 복원…** (Restore from backup…) | | 그런 zip을 다시 읽어 들입니다. |
| **진단 정보 내보내기 (zip)…** (Export diagnostics (zip)…) | | 버그 보고서에 첨부할 zip입니다. |
| **업데이트 확인…** (Check for updates…) | | 지금 새 릴리스를 찾습니다. |
| **정보** (About) | | 프로그램과 baseCreator의 버전, 링크, 라이선스입니다. Windows와 Linux 전용입니다(macOS에서는 **LightCreator → LightCreator 정보**). |
