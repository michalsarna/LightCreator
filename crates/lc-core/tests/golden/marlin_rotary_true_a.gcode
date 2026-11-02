; LightCreator job — Golden
G21 ; mm
G90 ; absolute
G17
M3 I S0
; job: 1 layer(s) to burn, 1 object(s)
; layer C00 (Line) speed 20 mm/s power 60%
; settings: number 1 of 1 to burn, 1 object(s), 1 pass(es), air assist off
; object 1 of 1 start: rectangle
; path 1 of 1 start: closed, 60.0 mm
G0 X30.000 A343.775
G1 X30.000 A366.693 S600 F1200
G1 X50.000 A366.693
G1 X50.000 A343.775
G1 X30.000 A343.775
; path 1 end
; object 1 end
; end of layer C00
; end of the job
M5 I ; laser off
G0 X0.000 Y0.000
