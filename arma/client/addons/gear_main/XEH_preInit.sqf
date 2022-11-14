#include "script_component.hpp"

[
    QGVAR(enabled),
    "CHECKBOX",
    ["Enabled", "Enable Persistent Gear"],
    "Synixe Crate - Persistent Gear",
    false,
    true,
    {},
    true
] call CBA_fnc_addSetting;

[
    QGVAR(read_only),
    "CHECKBOX",
    ["Read Only", "Load gear when the mission starts, but do not save changes"],
    "Synixe Crate - Persistent Gear",
    false,
    true,
    {},
    true
] call CBA_fnc_addSetting;
