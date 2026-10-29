# Plugins

Plugins add things to LightCreator without changing the program: **clipart, machine presets, translations, importers of other file formats, generators of shapes and G-code postprocessors.** They are shared by LightCreator, MillCreator and PenCreator: a plugin lives in the folder `plugins` inside the shared data folder (`baseCreator`) and each program uses the parts that fit it.

**Settings → Plugins…** opens the window: install a plugin from a `.zip`, switch plugins on and off, remove them, open the folder, and set the plugins that have settings: every plugin with something to choose (a **postprocessor**) has a tab of its own in the right-hand panel, where it is chosen for the next G-code export.

* **Importers:** **File → Import → Import with a plugin…**, drag and drop, or Open on a file whose extension a plugin reads. The shapes are added to the drawing (selected, one undo).
* **Generators:** **File → Import → Generate shape…** shows the form of the generator (numbers, switches, texts, lists) and adds the shapes it makes.
* **Postprocessors:** when one is chosen in the tab of its plugin, **Export G-code…** passes the program through it. It can only change the lines of G-code; lines that are controller commands (`$…`, `~`, `!`, `?`) or contain control characters are refused and nothing is saved. Sending a job straight to the machine does not go through a postprocessor.
* Presets and data files (`mill-tools`) of plugins are not used by LightCreator; it takes clipart, importers, generators and postprocessors.

**Using the G-code tools (and any postprocessor), step by step.** 1. *Settings → Plugins…* → *Official plugins* → install **G-code tools** (or *Install plugin from zip…*). 2. A tab named **G-code tools** appears in the right-hand panel, after the other tabs (when the panel is collapsed, on the narrow strip). 3. Open it and choose **Number the lines (N10, N20, …)** or **Remove comments**; *None* switches it off again. 4. **Export G-code…** (Laser / Milling / Plot menu, or the *Save G-code* button) now writes the processed file; the status bar says which postprocessor was used. A job started with **Start** is sent unchanged, and the G-code viewer shows the file as it was written.

**Safety.** Plugin scripts (Rhai) run in a sandbox without access to files, the network, the system or the machine, with a time and memory limit. Even so a plugin is somebody's code that decides what your machine is told to do: install only plugins you trust, and look at the G-code of a new postprocessor before running it on a machine.

## Official plugins

**Settings → Plugins… → Official plugins** shows the official list of plugins, kept in the repository [baseCreator-plugins](https://github.com/michalsarna/baseCreator-plugins). Press **Check for updates** to load it: it lists *all* plugins, also the ones for another program (marked *For: …*). **Install** downloads a plugin from the official site (its checksum is checked) into the plugins folder shared by all programs; **Update** appears when the list has a newer version, with the changelog on hover. A plugin that needs a newer program than yours is shown but not offered. Looking for updates when the program starts is **off** until you tick *Check for plugin updates at start*; nothing is installed without your click.

Writing your own plugin and sending it to the list: the [guide for authors](https://github.com/michalsarna/baseCreator-plugins/blob/main/docs/GUIDE.md) in the plugins repository, with templates; problems and ideas go to the issues of that repository.

How to write a plugin (manifest, form, script functions, examples): [PLUGINS.md in baseCreator](https://github.com/michalsarna/baseCreator/blob/main/docs/PLUGINS.md).
