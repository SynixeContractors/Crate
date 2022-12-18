#include "script_component.hpp"

params ["_box"];

GVAR(shop_blurHandle) = ppEffectCreate ["DynamicBlur", 800];
GVAR(shop_blurHandle) ppEffectEnable true;
GVAR(shop_blurHandle) ppEffectAdjust [8];
GVAR(shop_blurHandle) ppEffectCommit 0.25;

GVAR(shop_box) = _box;

player enableSimulation false;
player setVariable [QGVAR(shop_open), true, true];

private _loadout = [player] call CBA_fnc_getLoadout;
GVAR(shop_preLoadout) = _loadout;
private _items = [_loadout] call FUNC(loadout_items);

if (GVAR(readOnly)) then {
    [[], 0] call FUNC(shop_enter);
} else {
    [QGVAR(shop_enter), [player, _items]] call CBA_fnc_serverEvent;
};
