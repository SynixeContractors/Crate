#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate") exitWith {};
    switch (_func) do {
        case "global_message": {
            [QGVAR(brodskySay), _data] call CBA_fnc_globalEvent;
        };
    };
}];
