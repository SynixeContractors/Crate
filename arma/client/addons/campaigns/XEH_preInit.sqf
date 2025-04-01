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
    QGVAR(loadouts),
    "CHECKBOX",
    ["Loadouts", "Use Campaign Loadouts"],
    "Crate - Persistent Campaign",
    false,
    true,
    {
        if (!isServer) exitWith {};
        diag_log format ["Campaign Loadout: %1", _this];
        if _this then {
            EXTCALL("gear:loadout:campaign",[GVAR(key)]);
            EGVAR(gear,readOnly) = true;
        } else {
            EXTCALL("gear:loadout:reset",[]);
        };
    },
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
