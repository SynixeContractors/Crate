#include "script_component.hpp"

if !(hasInterface) exitWith {};
if !(isMultiplayer) exitWith {};

[QGVAR(member), [getPlayerUID player, profileName]] call CBA_fnc_serverEvent;

player addEventHandler ["Respawn", {
    params ["", "_corpse"];
    private _discord = player getVariable [QGVAR(id), ""];
    if (_discord isEqualTo "") exitWith {
        systemChat "player discord is empty, can't transfer to corpse";
    };
    _corpse setVariable [QGVAR(id), _discord, true];
    _corpse setVariable [QGVAR(steam), getPlayerUID player, true];
}];
