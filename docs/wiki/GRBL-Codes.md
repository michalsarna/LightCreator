🌐 **English** · [Polski](Kody-GRBL)

# GRBL Codes

What LightCreator sends to a GRBL controller and what the controller answers. Facts about GRBL are those of **GRBL 1.1** (the firmware most diode lasers run); other versions and forks (grblHAL, FluidNC, vendor builds) can differ. Nothing here was tried on a real machine; the program's own streaming was tested against the [GRBL simulator](GRBL-Simulator).

## Real-time characters

Single bytes that GRBL acts on at once, even in the middle of a job. They are not lines and get no `ok`.

| Byte | Meaning | Used by LightCreator |
|---|---|---|
| `?` | Status report | Asked for four times a second; the reply fills the state and the position |
| `!` | Feed hold | **Pause** |
| `~` | Cycle start / resume | **Resume** |
| Ctrl+X (0x18) | Soft reset | **STOP**, and once after connecting so that the controller is in a known state |

After a soft reset in motion, real GRBL 1.1 raises `ALARM:3` (the position is lost) and needs `$X` or homing.

## System commands (`$`)

| Command | Meaning | Used by LightCreator |
|---|---|---|
| `$H` | Run the homing cycle | **Home**, "Home the machine" after connecting, **Live laser Position** switched on at rest |
| `$HX`, `$HY` | Home one axis (only when the firmware reports it) | The *Home X* / *Home Y* buttons |
| `$X` | Kill the alarm lock | **Unlock**, "Clear an alarm" after connecting. It skips homing: use it only when the head stands in a safe place |
| `$$` | Print the settings | The **Firmware ($$)** page, and once after connecting to check laser mode (`$32`) and `$30` |
| `$N=value` | Write a setting | The same page, one value at a time, after confirmation |
| `$J=...` | Jog (a motion that can be cancelled at once and never runs while a job does) | The jog arrows, for example `$J=G91 G21 X10 Y0 F3000` |
| `$G`, `$#`, `$I`, `$C`, `$N` | Parser state, coordinate offsets, build info, check mode, start-up lines | Not used |

## G-code the program writes

| Code | Meaning |
|---|---|
| `G21` | Millimetres |
| `G90` | Absolute coordinates |
| `G17` | XY plane |
| `G0 X.. Y..` | Rapid move (laser off) |
| `G1 X.. Y.. S.. F..` | Cutting move with power `S` and feed `F` |
| `G2` / `G3` | Clockwise / counter-clockwise arc, when the device writes arcs |
| `G91` | Relative coordinates, only inside a jog |

| Code | Meaning |
|---|---|
| `M3 S0` | Constant power: the laser stays at `S` while the program runs (focus beam, jobs of a layer with constant power) |
| `M4 S0` | Dynamic power: the power follows the speed, so corners do not burn deeper (needs `$32=1`, laser mode) |
| `M5` | Laser off |
| `M8` / `M9` | Air assist on / off (or the device's own commands) |
| `M2` | End of the program |

`S` runs from 0 to the **S-value max** of the head (`$30`). A job also starts with the device's own start lines, hooks and macros when you gave some.

## Settings (`$n`)

The list below is the one the Firmware page shows; hover a row there for the explanation. Values marked **critical** can fire the laser or move the machine the wrong way, so the page asks for a confirmation before it writes them.

| `$` | Name | Unit |
|---|---|---|
| 0 | Step pulse time | µs |
| 1 | Step idle delay | ms |
| 2 / 3 | Step pulse / direction invert | mask |
| 4, 5, 6 | Invert step enable / limit pins / probe pin | 0 / 1 |
| 10 | Status report options | mask |
| 11 | Junction deviation | mm |
| 12 | Arc tolerance | mm |
| 13 | Report in inches | 0 / 1 |
| 20 | Soft limits | 0 / 1 |
| 21 | Hard limits | 0 / 1 |
| 22 | Homing cycle | 0 / 1 |
| 23 | Homing direction invert | mask |
| 24 / 25 | Homing feed / seek | mm/min |
| 26 | Homing debounce | ms |
| 27 | Homing pull-off | mm |
| 30 | Maximum spindle speed (S for 100 %), **critical** | S |
| 31 | Minimum spindle speed, **critical** | S |
| 32 | Laser mode, **critical** | 0 / 1 |
| 100–102 | X, Y, Z steps per mm | steps/mm |
| 110–112 | X, Y, Z maximum rate | mm/min |
| 120–122 | X, Y, Z acceleration | mm/s² |
| 130–132 | X, Y, Z maximum travel, **critical** | mm |

For a laser: `$32=1` (laser mode) and `$30` equal to the S-value max of the profile (1000 on most boards, 255 on some). LightCreator warns after connecting when laser mode is off or `$30` differs from the profile.

## States in the status report

`<Idle|MPos:10.000,20.000,0.000|FS:0,0>`: the first word is the state.

| State | Meaning |
|---|---|
| `Idle` | Waiting for work |
| `Run` | Moving (a job, a frame, a jog) |
| `Hold:0` / `Hold:1` | Feed hold: 0 = finished stopping, 1 = still slowing down |
| `Jog` | A jog motion |
| `Home` | Homing |
| `Alarm` | Locked: unlock with `$X` or home |
| `Door:0`–`Door:3` | Safety door states |
| `Check` | Check mode (`$C`): moves are only checked |
| `Sleep` | Sleeping |

## Replies

* `ok`: the line was understood and put in the planner. LightCreator counts the bytes of the lines that wait for their `ok` and keeps the controller's 128-byte receive buffer full but not over (or sends one line at a time in *Simple* streaming).
* `error:N`: the line was refused. The job goes on (the line counts as answered) and the console shows the error.
* `ALARM:N`: the machine stopped and is locked.
* `[MSG:...]`, `[GC:...]`, `Grbl 1.1h ['$' for help]`: information, never counted as answers. The banner after a reset tells the program that GRBL is ready.

### `error:N`

| N | Meaning |
|---|---|
| 1 | A G-code word is a letter without a number or the letter is missing |
| 2 | A number has a wrong format or is missing |
| 3 | The `$` command is not known or not supported |
| 4 | A negative value where a positive one is needed |
| 5 | Homing is not enabled (`$22`) |
| 6 | Step pulse time below 3 µs |
| 7 | Reading the EEPROM failed; defaults were restored (often seen once after flashing) |
| 8 | The `$` command works only when GRBL is idle |
| 9 | G-code is locked out during an alarm or a jog |
| 10 | Soft limits need homing enabled |
| 11 | The line is longer than GRBL accepts and was dropped |
| 12 | A setting exceeds the step rate GRBL supports |
| 13 | The safety door is open |
| 14 | A start-up line or build info is too long for the EEPROM |
| 15 | The jog target is beyond the machine's travel |
| 16 | A jog command without `=` or with forbidden G-code |
| 17 | Laser mode needs a PWM spindle output |
| 20 | Unsupported or invalid G-code command |
| 21 | Two commands of one modal group in a line |
| 22 | No feed rate set yet |
| 23 | The command needs an integer value |
| 24 | Two commands that both need axis words in one line |
| 25 | A G-code word repeated in the line |
| 26 | The command needs axis words and there are none |
| 27 | The `N` line number is outside 1 – 9 999 999 |
| 28 | A required `P` or `L` value is missing |
| 29 | Only the work coordinate systems G54–G59 exist (G59.1–G59.3 do not) |
| 30 | G53 needs G0 or G1 motion mode |
| 31 | Unused axis words with motion mode cancel (G80) active |
| 32 | An arc without axis words in the selected plane |
| 33 | The motion target is invalid (an arc that cannot exist) |
| 34 | The arc radius gave a mathematical error |
| 35 | An arc with offsets lacks the I, J or K word of its plane |
| 36 | Leftover G-code words that no command uses |
| 37 | G43.1 can offset only its configured axis |
| 38 | Tool number above the supported maximum |

### `ALARM:N`

| N | Meaning | What to do |
|---|---|---|
| 1 | Hard limit triggered; the position is probably lost | Check the switch, home again |
| 2 | The G-code target exceeds the travel (soft limit); the position is kept | `$X` is enough; check the work against the work area |
| 3 | Reset while moving; the position is not guaranteed | Home again |
| 4 | Probe failed: not in its start state | Check the probe (not used by LightCreator) |
| 5 | Probe failed: no contact within the travel | As above |
| 6 | Homing failed: reset during the cycle | Home again |
| 7 | Homing failed: the safety door opened | Close it, home again |
| 8 | Homing failed: the switch did not clear on pull-off | Raise `$27` or check the wiring |
| 9 | Homing failed: the switch was not found | Check the switch, the direction (`$23`) and the wiring |

The *Unlock* button sends `$X`. It does not move the machine: if the head moved while the machine was locked, the position in the program no longer matches reality until you home.

## Where to read more

* The GRBL 1.1 wiki: [Commands](https://github.com/gnea/grbl/wiki/Grbl-v1.1-Commands), [Configuration](https://github.com/gnea/grbl/wiki/Grbl-v1.1-Configuration), [Laser mode](https://github.com/gnea/grbl/wiki/Grbl-v1.1-Laser-Mode) and [Jogging](https://github.com/gnea/grbl/wiki/Grbl-v1.1-Jogging).
* [Devices and Controllers](Devices-and-Controllers) and [Safety and Limitations](Safety-and-Limitations) in this wiki.
