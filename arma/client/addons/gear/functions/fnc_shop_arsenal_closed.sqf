#include "script_component.hpp"

params ["_display"];

if !(GVAR(enabled)) exitWith {};

if (ace_player isNotEqualTo player) exitWith {};
if !(player getVariable [QGVAR(shop_open), false]) exitWith {};

[GVAR(shop_balanceHandle)] call CBA_fnc_removePerFrameHandler;
[GVAR(shop_box), false] call ace_arsenal_fnc_removeBox;

if (GVAR(readOnly) || EGVAR(campaigns,loadouts)) exitWith {
    player setVariable [QGVAR(shop_open), false, true];
};

private _loadout = [player] call CBA_fnc_getLoadout;
private _items = [_loadout] call FUNC(loadout_items);
private _items_new = [+_items] call FUNC(shop_items_removeOwned);
private _cost = [_items_new, 2] call FUNC(shop_items_cost);

if (_cost == 0) then {
    [QGVAR(shop_leave), [player, _loadout, _items]] call CBA_fnc_serverEvent;
} else {
    systemChat "You do not own all items, reverting changes.";
    [player, GVAR(shop_preLoadout), false] call CBA_fnc_setLoadout;
    private _items = [GVAR(shop_preLoadout)] call FUNC(loadout_items);
    [QGVAR(shop_leave), [player, GVAR(shop_preLoadout), _items]] call CBA_fnc_serverEvent;
};
