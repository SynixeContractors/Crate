#include "script_component.hpp"

params ["_steam"];

// TODO replace with server side hash map cache?

private _player = objNull;
{
    if ((getPlayerUID _x) isEqualTo _steam) exitWith {_player = _x;}
} forEach allPlayers;

_player
