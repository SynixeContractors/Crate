#include "script_component.hpp"

if !(hasInterface) exitWith {};
if !(isMultiplayer) exitWith {};

[QGVAR(brodskySay), {
    params ["_text"];
    player customChat [GVAR(brodskyChat), _text];
}] call CBA_fnc_addEventHandler;

GVAR(disableScore) = {
    player addEventHandler ["HandleScore", {
        false
    }];
};

player addEventHandler ["Respawn", {
    params ["_unit"];
    _unit call GVAR(disableScore);
}];
player call GVAR(disableScore);

[player] call FUNC(noDefaultFace);
