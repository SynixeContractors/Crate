#include "script_component.hpp"

[
    QGVAR(enabled),
    "CHECKBOX",
    ["Enabled", "Shops are enabled"],
    ["Synixe Crate - Persistent Gear", "Shops"],
    true,
    true,
] call CBA_fnc_addSetting;
