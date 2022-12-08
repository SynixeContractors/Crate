#include "script_component.hpp"

params ["_display"];

if !(GVAR(enabled) && {!(GVAR(readOnly))}) exitWith {
    [_display] call ace_arsenal_fnc_buttonHide;
};

createDialog QGVAR(RscCheckout);
