#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

[
    QGVAR(enabled),
    "CHECKBOX",
    ["Enabled", "Shops are enabled"],
    ["Synixe Crate - Persistent Gear", "Shops"],
    true,
    true
] call CBA_fnc_addSetting;
