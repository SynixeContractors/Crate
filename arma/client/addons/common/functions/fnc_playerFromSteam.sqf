#include "..\script_component.hpp"

params ["_steam"];

private _player = objNull;
{
    if ((getPlayerUID _x) isEqualTo _steam) exitWith {_player = _x;}
} forEach ((allUnits + allDead) select {isPlayer _x});

_player
