Designing a UI framework for CAD & Tooling applications, **Territory Tabs**. The design will be based on my own experiences with all varieties of these apps, and watching others succeed and fail when using them. I'll also be developing a CAD program with this framework!

##Design Goals##

**Easy for Novices, Open to Experts**
 - Tightly constrained UI design environment for a streamlined, consistent, easy-to-learn user experience.
 - ...but with a completely exposed configuration for the experts.
 - Ultra flat and wide feature heap with a central, easily-accessed fuzzy search for navigation.
 - Frequently used features can be turned into Tabs, which sit inside non-overlapping Territories.
 - Layout and linked layout features emulate workflow tabs.
 - Multi-window workflow support.
 - Works with keyboard only, mouse only, and touchpad only. Keybinds & shortcuts, naturally.

**Consistent UI Rules**
 - The only opaque overlays are context menus and toasts. Nothing else overlaps!
 - Features are never hidden or removed, and can only be disabled (greyed-out).
 - All disabled features will have, on-hover, a reason for why it is disabled, and a meaning for further explanation.
 - RuleSets (exposed Systems) govern feature enabling/disabling, among many other domains.
 - Modern quality of life standards, like toggling UI elements not moving the toggle source.

**Lessons Learned**
 - 30 years of CAD & Tooling design errors have informed the priorities of Territory Tabs.
 - Version-controlled, SHA1 Repository compatible, deterministic Operations for a single source of truth.
 - Taskpool for predicting & parallelizing large Operations.
 - All features and actions exposed out of the gate for scripting and integrations with other apps.
 - All ECS, all the time.

## License

This work is dual-licensed under Apache 2.0 and MIT.
You can choose between one of them if you use this work.
