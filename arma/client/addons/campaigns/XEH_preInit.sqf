#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

[
    QGVAR(enabled),
    "CHECKBOX",
    ["Enabled", "Enable Persistent Campaigns"],
    "Crate - Persistent Campaign",
    false,
    true,
    {},
    true
] call CBA_fnc_addSetting;

[
    QGVAR(key),
    "EDITBOX",
    ["Key", "Campaign Key"],
    "Crate - Persistent Campaign",
    "",
    true,
    {},
    true
] call CBA_fnc_addSetting;
