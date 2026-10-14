# Plugins

Plugins add things to LightCreator without changing the program: **clipart, machine presets, translations, importers of other file formats, generators of shapes and G-code postprocessors.** They are shared by LightCreator, MillCreator and PenCreator: a plugin lives in the folder `plugins` inside the shared data folder (`baseCreator`) and each program uses the parts that fit it.

**Settings → Plugins…** opens the window: install a plugin from a `.zip`, switch plugins on and off, remove them, open the folder, and choose the **postprocessor** for the next G-code export.

* **Importers:** **File → Import → Import with a plugin…**, drag and drop, or Open on a file whose extension a plugin reads. The shapes are added to the drawing (selected, one undo).
* **Generators:** **File → Import → Generate shape…** shows the form of the generator (numbers, switches, texts, lists) and adds the shapes it makes.
* **Postprocessors:** when one is chosen in the Plugins window, **Export G-code…** passes the program through it. It can only change the lines of G-code; lines that are controller commands (`$…`, `~`, `!`, `?`) or contain control characters are refused and nothing is saved. Sending a job straight to the machine does not go through a postprocessor.
* Presets and data files (`mill-tools`) of plugins are not used by LightCreator; it takes clipart, importers, generators and postprocessors.

**Safety.** Plugin scripts (Rhai) run in a sandbox without access to files, the network, the system or the machine, with a time and memory limit. Even so a plugin is somebody's code that decides what your machine is told to do: install only plugins you trust, and look at the G-code of a new postprocessor before running it on a machine.

How to write a plugin (manifest, form, script functions, examples): [PLUGINS.md in baseCreator](https://github.com/michalsarna/baseCreator/blob/main/docs/PLUGINS.md).
