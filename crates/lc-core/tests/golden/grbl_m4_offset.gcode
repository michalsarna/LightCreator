; LightCreator job — Golden
G21 ; mm
G90 ; absolute
G17
M4 S0
; job: 1 layer(s) to burn, 1 object(s)
; layer C00 (Offset fill) speed 20 mm/s power 60%
; settings: number 1 of 1 to burn, 1 object(s), 1 pass(es), air assist off
; offset fill start: 5 rings, 1 mm apart
; ring 1 of 5 start: 28.0 mm
G0 X34.000 Y154.000
G1 X34.000 Y156.000 S600 F1200
G1 X46.000 Y156.000
G1 X46.000 Y154.000
G1 X34.000 Y154.000
; ring 1 end
; ring 2 of 5 start: 36.0 mm
G0 X33.000 Y153.000
G1 X33.000 Y157.000
G1 X47.000 Y157.000
G1 X47.000 Y153.000
G1 X33.000 Y153.000
; ring 2 end
; ring 3 of 5 start: 44.0 mm
G0 X32.000 Y152.000
G1 X32.000 Y158.000
G1 X48.000 Y158.000
G1 X48.000 Y152.000
G1 X32.000 Y152.000
; ring 3 end
; ring 4 of 5 start: 52.0 mm
G0 X31.000 Y151.000
G1 X31.000 Y159.000
G1 X49.000 Y159.000
G1 X49.000 Y151.000
G1 X31.000 Y151.000
; ring 4 end
; ring 5 of 5 start: 60.0 mm
G0 X30.000 Y150.000
G1 X30.000 Y160.000
G1 X50.000 Y160.000
G1 X50.000 Y150.000
G1 X30.000 Y150.000
; ring 5 end
; offset fill end
; end of layer C00
; end of the job
M5 ; laser off
G0 X0.000 Y0.000
M2
