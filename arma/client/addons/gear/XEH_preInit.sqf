#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"

[
    QGVAR(enabled),
    "CHECKBOX",
    ["Enabled", "Enable Persistent Gear"],
    "Crate - Persistent Gear",
    false,
    true,
    {},
    true
] call CBA_fnc_addSetting;

[
    QGVAR(readOnly),
    "CHECKBOX",
    ["Read Only", "Load gear when the mission starts, but do not save changes"],
    "Crate - Persistent Gear",
    false,
    true,
    {},
    true
] call CBA_fnc_addSetting;

ADDON = true;
