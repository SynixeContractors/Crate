#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate:global") exitWith {};
    switch (_func) do {
        case "brodsky_say": {
            GVAR(brodskyChat) radioChannelAdd allPlayers;
            [{
                [QGVAR(brodskySay), parseSimpleArray _this] call CBA_fnc_globalEvent;
            }, _data, 1] call CBA_fnc_waitAndExecute;
        };
    };
}];

GVAR(brodskyChat) = radioChannelCreate [
    [1.0, 0.84, 0.19, 0.8],
    "Brodsky",
    "Brodsky",
    [player]
];
publicVariable QGVAR(brodskyChat);
