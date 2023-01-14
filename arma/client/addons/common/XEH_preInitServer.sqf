#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate") exitWith {};
    switch (_func) do {
        case "global_message": {
            GVAR(brodskyChat) radioChannelAdd allPlayers;
            [QGVAR(brodskySay), _data] call CBA_fnc_globalEvent;
        };
    };
}];

GVAR(brodskyChat) = radioChannelCreate [
    [1.0, 0.84, 0.19, 0.8],
    "Brodsky",
    "Brodsky",
    [player]
];
GVAR(brodskyChat) enableChannel false;
publicVariable QGVAR(brodskyChat);
