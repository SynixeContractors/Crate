#include "script_component.hpp"

params ["_display"];

if (!(GVAR(enabled) && !(GVAR(readOnly))) || EGVAR(campaigns,loadouts)) exitWith {
    [_display] call ace_arsenal_fnc_buttonHide;
};

createDialog QGVAR(RscCheckout);
