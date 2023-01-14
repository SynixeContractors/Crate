#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

[QGVAR(brodskySay), {
    params ["_text"];
    player customChat [GVAR(brodskyChat), _text];
}] call CBA_fnc_addEventHandler;
