#include "script_component.hpp"

[QEGVAR(shop_arsenal,loaderRegister), QUOTE(ADDON)] call CBA_fnc_localEvent;

private _me = [getPlayerUID player] call core_discord_fnc_findMemberFromSteam;

GVAR(preLoadout) = [player] call CBA_fnc_getLoadout;

[QGVAR(store), [_me#1, _me#4, [GVAR(preLoadout)] call FUNC(loadout_items)]] call CBA_fnc_serverEvent;
