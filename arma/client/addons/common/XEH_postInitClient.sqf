#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

GVAR(brodskyChat) = radioChannelCreate [
    [1.0, 0.84, 0.19, 0.8],
    "Brodsky",
    "Brodsky",
    [player]
];
GVAR(brodskyChat) enableChannel false;

[QGVAR(brodskySay), {
    params ["_text"];
    player customChat [GVAR(brodskyChat), _text];
}] call CBA_fnc_addEventHandler;
