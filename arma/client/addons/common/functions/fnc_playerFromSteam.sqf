#include "script_component.hpp"

params ["_steam"];

private _player = objNull;
{
    if ((getPlayerUID _x) isEqualTo _steam) exitWith {_player = _x;}
} forEach allPlayers;

_player
