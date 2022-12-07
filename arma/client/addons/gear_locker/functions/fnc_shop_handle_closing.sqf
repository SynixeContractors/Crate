#include "script_component.hpp"

private _me = [getPlayerUID player] call core_discord_fnc_findMemberFromSteam;

[QGVAR(take), [_me#1, _me#4, [[player] call CBA_fnc_getLoadout] call FUNC(loadout_items)]] call CBA_fnc_serverEvent;
