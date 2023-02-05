#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

[
    QGVAR(enabled),
    "CHECKBOX",
    ["Enabled", "Enable Persistent Garage"],
    "Crate - Persistent Garage",
    false,
    true,
    {},
    true
] call CBA_fnc_addSetting;
