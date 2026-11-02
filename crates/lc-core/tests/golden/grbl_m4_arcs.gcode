; LightCreator job — Golden
G21 ; mm
G90 ; absolute
G17
M4 S0
; job: 1 layer(s) to burn, 2 object(s)
; layer C00 (Line) speed 20 mm/s power 60%
; settings: number 1 of 1 to burn, 2 object(s), 1 pass(es), air assist off
; object 1 of 2 start: ellipse
; path 1 of 2 start: closed, 125.7 mm
G0 X104.540 Y87.312
G2 X107.312 Y84.540 I15.460 J12.688 S600 F1200
G2 X104.540 Y87.312 I12.688 J15.460
; path 1 end
; object 1 end
; object 2 of 2 start: rectangle
; path 2 of 2 start: closed, 60.0 mm
G0 X50.000 Y150.000
G1 X30.000 Y150.000
G1 X30.000 Y160.000
G1 X50.000 Y160.000
G1 X50.000 Y150.000
; path 2 end
; object 2 end
; end of layer C00
; end of the job
M5 ; laser off
G0 X0.000 Y0.000
M2
