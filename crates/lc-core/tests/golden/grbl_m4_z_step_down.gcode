; LightCreator job — Golden
G21 ; mm
G90 ; absolute
G17
M4 S0
; job: 1 layer(s) to burn, 1 object(s)
; layer C00 (Line) speed 20 mm/s power 60%
; settings: number 1 of 1 to burn, 1 object(s), 3 pass(es), air assist off, Z step -2.5 mm
; pass 1/3
G0 Z12.000 ; Z for pass 1
; object 1 of 1 start: rectangle
; path 1 of 1 start: closed, 60.0 mm
G0 X30.000 Y150.000
G1 X30.000 Y160.000 S600 F1200
G1 X50.000 Y160.000
G1 X50.000 Y150.000
G1 X30.000 Y150.000
; path 1 end
; object 1 end
; pass 2/3
G0 Z9.500 ; Z for pass 2
; object 1 of 1 start: rectangle
; path 1 of 1 start: closed, 60.0 mm
G1 X30.000 Y160.000
G1 X50.000 Y160.000
G1 X50.000 Y150.000
G1 X30.000 Y150.000
; path 1 end
; object 1 end
; pass 3/3
G0 Z7.000 ; Z for pass 3
; object 1 of 1 start: rectangle
; path 1 of 1 start: closed, 60.0 mm
G1 X30.000 Y160.000
G1 X50.000 Y160.000
G1 X50.000 Y150.000
G1 X30.000 Y150.000
; path 1 end
; object 1 end
; end of layer C00
; end of the job
M5 ; laser off
G0 X0.000 Y0.000
M2
